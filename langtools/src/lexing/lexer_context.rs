// MIT License
//
// Copyright (c) 2022 Sophie Katz
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::{
    domain::token::{Token, TokenKind},
    messaging::{
        message::{Message, Severity},
        message_context::MessageContext,
    },
    sourcing::{
        read_source::ReadSource, source_reader::SourceReader, sourcing_error::SourcingError,
    },
};

use super::{
    dfsa_executor::DFSAExecutor,
    lexer::Lexer,
    lexer_trigger_action::LexerTriggerAction,
    lexing_error::{LexingError, Result},
};

pub struct LexerContext<'lexer, TTokenKind: TokenKind> {
    lexer: &'lexer Lexer<TTokenKind>,
    source_reader: &'lexer mut SourceReader<'lexer>,
    message_context: &'lexer mut MessageContext,
}

impl<'lexer, TTokenKind: TokenKind> LexerContext<'lexer, TTokenKind> {
    pub fn new(
        lexer: &'lexer Lexer<TTokenKind>,
        source_reader: &'lexer mut SourceReader<'lexer>,
        message_context: &'lexer mut MessageContext,
    ) -> Self {
        Self {
            lexer,
            source_reader,
            message_context,
        }
    }

    pub fn lex_next(&mut self) -> Result<Token<TTokenKind>> {
        // Check preconditions
        assert!(self.source_reader.is_buffering_enabled());
        assert!(self.source_reader.is_buffer_empty());

        // Save location of first character of token
        let location_first = self.source_reader.location.clone();

        // Lex next trigger or return error
        let trigger_action = self.lex_next_trigger_action()?;

        // Run the trigger action
        if let Some(token_kind) = (trigger_action.callback)(self.source_reader) {
            // If a token kind of specified, emit the token
            Ok(Token::new_from_location(
                location_first,
                self.source_reader.pop_buffer()?,
                token_kind,
            ))
        } else {
            // If no token kind is specified, skip the token and try to lex another
            self.source_reader.clear_buffer()?;
            self.lex_next()
        }
    }

    fn lex_next_trigger_action(&mut self) -> Result<&LexerTriggerAction<TTokenKind>> {
        let mut trigger_dfsa_executor: DFSAExecutor<char, LexerTriggerAction<TTokenKind>> =
            DFSAExecutor::new(&self.lexer.trigger_dfsa)?;

        // We need to greedily eat the next trigger using the trigger DFSA. Let's say we have a string we're going
        // to lex "abbc" and two triggers in our DFSA: one for "ab" and one for "abb". Thus our DFSA would look like
        // this:
        //
        // State 0 (start):
        //   'a' -> state 1
        // State 1:
        //   'b' -> state 2
        // State 2 (trigger 0):
        //   'b' -> state 3
        // State 3 (trigger 1)
        //
        // The reading of the trigger would then look like this:
        //
        // Step 1:
        //   At state 0.
        //   Peek 'a'.
        //   Has transition to state 1.
        //   Eat 'a'.
        //   Go to state 1.
        // Step 2:
        //   At state 1.
        //   Peek 'b'.
        //   Has transition to state 2.
        //   Eat 'b'.
        //   Go to state 2.
        // Step 3:
        //   At state 2.
        //   Has trigger 0.
        //   Peek 'b'.
        //   Has transition to state 3.
        //   Eat 'b'.
        //   Go to state 3.
        // Step 4:
        //   At state 3.
        //   Has trigger 1.
        //   Peek 'c'.
        //   No transition available.
        //   Return trigger 1 with buffer "abb".
        //
        // The algorithm is then:
        //
        // While true:
        //   If the current state has a trigger, save it as the best trigger so far.
        //   Peek the next character if there are more characters to be read.
        //   If there is a transition on the next character:
        //     Eat the next character and append it to the buffer.
        //     Go to the state denoted by the transition.
        //   Else:
        //     Break.
        //
        // If a best trigger has been saved, return it with the buffer so far,
        // Else go into error recovery.

        let mut last_trigger: Option<&LexerTriggerAction<TTokenKind>> = None;
        let first_char = self.source_reader.peek_next().ok();

        loop {
            let current_trigger = trigger_dfsa_executor.current_action();
            if current_trigger.is_some() {
                last_trigger = current_trigger;
            }

            if let Ok(next_char) = self.source_reader.peek_next() {
                if trigger_dfsa_executor.step(next_char).is_ok() {
                    self.source_reader.eat_next()?;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if let Some(last_trigger) = last_trigger {
            Ok(last_trigger)
        } else if let Some(first_char) = first_char {
            if trigger_dfsa_executor.is_at_start() {
                Err(LexingError::UnexpectedCharacter(first_char))
            } else {
                Err(LexingError::UnexpectedEndOfSource)
            }
        } else {
            Err(LexingError::UnexpectedEndOfSource)
        }
    }
}

impl<'lexer, TTokenKind: TokenKind> Iterator for LexerContext<'lexer, TTokenKind> {
    type Item = Token<TTokenKind>;

    fn next(&mut self) -> Option<Self::Item> {
        let location_first = self.source_reader.location.clone();

        match self.lex_next() {
            Ok(token) => Some(token),
            Err(LexingError::UnexpectedEndOfSource)
            | Err(LexingError::SourcingError(SourcingError::NoMoreChars)) => None,
            Err(LexingError::UnexpectedCharacter(chr)) => {
                self.message_context.emit(Message::new_location(
                    location_first,
                    Severity::Error,
                    format!("unexpected character {chr:?}"),
                ));

                (self.lexer.get_error_handler())(self.source_reader);
                self.source_reader.clear_buffer().ok()?;

                self.next()
            }
            Err(LexingError::SourcingError(err)) => {
                self.message_context.emit(Message::new_location(
                    location_first,
                    Severity::InternalError,
                    format!("sourcing error: {err}"),
                ));

                None
            }
            Err(LexingError::FSAError(err)) => {
                self.message_context.emit(Message::new_location(
                    location_first,
                    Severity::InternalError,
                    format!("FSA error: {err}"),
                ));

                None
            }
            Err(LexingError::DuplicateTrigger(_)) => {
                panic!("this error type should not be emitted during lexing")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{domain::source_info::SourceInfo, sourcing::source_string::SourceString};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
    enum TokenKindTest {
        A,
        AB,
        AC,
        ABC,
    }

    impl TokenKind for TokenKindTest {}

    #[test]
    fn test_lexer_context_simple() {
        let mut lexer = Lexer::<TokenKindTest>::new();

        assert_eq!(lexer.add_trigger("ab", |_| Some(TokenKindTest::AB)), Ok(()));
        assert_eq!(lexer.add_trigger("ac", |_| Some(TokenKindTest::AC)), Ok(()));
        assert_eq!(
            lexer.add_trigger("abc", |_| Some(TokenKindTest::ABC)),
            Ok(())
        );

        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "abacabcac");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.enable_buffering(), Ok(()));

        let mut message_context = MessageContext::new();

        let mut token_source = lexer.lex(&mut source_reader, &mut message_context);

        assert_eq!(
            token_source.lex_next(),
            Ok(Token::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("ab"),
                TokenKindTest::AB
            ))
        );

        assert_eq!(
            token_source.lex_next(),
            Ok(Token::new(
                PathBuf::from("--"),
                2,
                1,
                3,
                String::from("ac"),
                TokenKindTest::AC
            ))
        );

        assert_eq!(
            token_source.lex_next(),
            Ok(Token::new(
                PathBuf::from("--"),
                4,
                1,
                5,
                String::from("abc"),
                TokenKindTest::ABC
            ))
        );

        assert_eq!(
            token_source.lex_next(),
            Ok(Token::new(
                PathBuf::from("--"),
                7,
                1,
                8,
                String::from("ac"),
                TokenKindTest::AC
            ))
        );

        assert_eq!(
            token_source.lex_next(),
            Err(LexingError::UnexpectedEndOfSource)
        );
    }

    #[test]
    fn test_lexer_context_unexpected_char() {
        let mut lexer = Lexer::<TokenKindTest>::new();

        assert_eq!(lexer.add_trigger("ab", |_| Some(TokenKindTest::AB)), Ok(()));
        assert_eq!(lexer.add_trigger("ac", |_| Some(TokenKindTest::AC)), Ok(()));
        assert_eq!(
            lexer.add_trigger("abc", |_| Some(TokenKindTest::ABC)),
            Ok(())
        );

        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "d");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.enable_buffering(), Ok(()));

        let mut message_context = MessageContext::new();

        let mut token_source = lexer.lex(&mut source_reader, &mut message_context);

        assert_eq!(
            token_source.lex_next(),
            Err(LexingError::UnexpectedCharacter('d'))
        );
    }

    #[test]
    fn test_lexer_context_unexpected_end() {
        let mut lexer = Lexer::<TokenKindTest>::new();

        assert_eq!(lexer.add_trigger("ab", |_| Some(TokenKindTest::AB)), Ok(()));
        assert_eq!(lexer.add_trigger("ac", |_| Some(TokenKindTest::AC)), Ok(()));
        assert_eq!(
            lexer.add_trigger("abc", |_| Some(TokenKindTest::ABC)),
            Ok(())
        );

        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "a");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.enable_buffering(), Ok(()));

        let mut message_context = MessageContext::new();

        let mut token_source = lexer.lex(&mut source_reader, &mut message_context);

        assert_eq!(
            token_source.lex_next(),
            Err(LexingError::UnexpectedEndOfSource)
        );
    }

    #[test]
    fn test_lexer_context_trigger_action() {
        let mut lexer = Lexer::<TokenKindTest>::new();

        assert_eq!(
            lexer.add_trigger("ab", |source_reader| {
                while source_reader.peek_next() == Ok('b') {
                    let _ = source_reader.eat_next();
                }

                Some(TokenKindTest::AB)
            }),
            Ok(())
        );
        assert_eq!(lexer.add_trigger("ac", |_| Some(TokenKindTest::AC)), Ok(()));
        assert_eq!(
            lexer.add_trigger("abc", |_| Some(TokenKindTest::ABC)),
            Ok(())
        );

        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "abbbacabcac");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.enable_buffering(), Ok(()));

        let mut message_context = MessageContext::new();

        let mut token_source = lexer.lex(&mut source_reader, &mut message_context);

        assert_eq!(
            token_source.lex_next(),
            Ok(Token::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("abbb"),
                TokenKindTest::AB
            ))
        );

        assert_eq!(
            token_source.lex_next(),
            Ok(Token::new(
                PathBuf::from("--"),
                4,
                1,
                5,
                String::from("ac"),
                TokenKindTest::AC
            ))
        );

        assert_eq!(
            token_source.lex_next(),
            Ok(Token::new(
                PathBuf::from("--"),
                6,
                1,
                7,
                String::from("abc"),
                TokenKindTest::ABC
            ))
        );

        assert_eq!(
            token_source.lex_next(),
            Ok(Token::new(
                PathBuf::from("--"),
                9,
                1,
                10,
                String::from("ac"),
                TokenKindTest::AC
            ))
        );

        assert_eq!(
            token_source.lex_next(),
            Err(LexingError::UnexpectedEndOfSource)
        );
    }

    #[test]
    fn test_lexer_context_error_recovery_default() {
        let mut lexer = Lexer::<TokenKindTest>::new();

        assert_eq!(lexer.add_trigger("a", |_| Some(TokenKindTest::A)), Ok(()));

        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "aba");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.enable_buffering(), Ok(()));

        let mut message_context = MessageContext::new();

        let mut token_source = lexer.lex(&mut source_reader, &mut message_context);

        assert_eq!(
            token_source.next(),
            Some(Token::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("a"),
                TokenKindTest::A
            ))
        );

        assert_eq!(
            token_source.next(),
            Some(Token::new(
                PathBuf::from("--"),
                2,
                1,
                3,
                String::from("a"),
                TokenKindTest::A
            ))
        );

        assert_eq!(token_source.next(), None);

        assert!((&message_context.messages)
            .iter()
            .any(|i| (*i.description).contains("unexpected")));
    }

    #[test]
    fn test_lexer_context_error_recovery_simple() {
        let mut lexer = Lexer::<TokenKindTest>::new();

        assert_eq!(lexer.add_trigger("a", |_| Some(TokenKindTest::A)), Ok(()));

        lexer.set_error_handler(|source| {
            let _ = source.eat_next();
            let _ = source.eat_next();
        });

        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "abaa");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.enable_buffering(), Ok(()));

        let mut message_context = MessageContext::new();

        let mut token_source = lexer.lex(&mut source_reader, &mut message_context);

        assert_eq!(
            token_source.next(),
            Some(Token::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("a"),
                TokenKindTest::A
            ))
        );

        assert_eq!(
            token_source.next(),
            Some(Token::new(
                PathBuf::from("--"),
                3,
                1,
                4,
                String::from("a"),
                TokenKindTest::A
            ))
        );

        assert_eq!(token_source.next(), None);

        assert!((&message_context.messages)
            .iter()
            .any(|i| (*i.description).contains("unexpected")));
    }
}
