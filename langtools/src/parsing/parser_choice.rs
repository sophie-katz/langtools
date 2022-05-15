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

use super::{parse::Parse, parsing_error::ParsingError};
use crate::{domain::token::TokenKind, tree::Tree};
use std::{collections::HashSet, mem};

pub struct ParserChoice<TTokenKind: TokenKind, TTree: Tree> {
    choices: Vec<Box<dyn Parse<TTokenKind, TTree>>>,
}

pub struct ParserChoiceBuilder<TTokenKind: TokenKind, TTree: Tree> {
    choices: Vec<Box<dyn Parse<TTokenKind, TTree>>>,
}

impl<TTokenKind: TokenKind, TTree: Tree> ParserChoiceBuilder<TTokenKind, TTree> {
    pub fn choices(&mut self, value: Box<dyn Parse<TTokenKind, TTree>>) -> &mut Self {
        self.choices.push(value);
        self
    }

    pub fn build(&mut self) -> Result<ParserChoice<TTokenKind, TTree>, ParsingError<TTokenKind>> {
        if self.choices.is_empty() {
            return Err(ParsingError::RequiredParserFieldMissing("choices"));
        }

        Ok(ParserChoice {
            choices: mem::take(&mut self.choices),
        })
    }
}

impl<TTokenKind: TokenKind, TTree: Tree> Default for ParserChoiceBuilder<TTokenKind, TTree> {
    fn default() -> Self {
        Self {
            choices: Vec::new(),
        }
    }
}

impl<TTokenKind: TokenKind, TTree: Tree> Parse<TTokenKind, TTree>
    for ParserChoice<TTokenKind, TTree>
{
    fn parse(
        &self,
        token_reader: &mut crate::lexing::token_reader::TokenReader<TTokenKind>,
    ) -> Result<TTree, ParsingError<TTokenKind>> {
        let offset = token_reader.offset();

        for child_parser in self.choices.iter() {
            if let Ok(child) = child_parser.parse(token_reader) {
                return Ok(child);
            } else {
                token_reader.seek(offset);
            }
        }

        if let Some(token) = token_reader.peek_next() {
            Err(ParsingError::UnexpectedToken {
                expected_token_kinds: self.expected_tokens()?,
                actual_token: token.clone(),
            })
        } else {
            Err(ParsingError::UnexpectedEndOfSource {
                expected_token_kinds: self.expected_tokens()?,
            })
        }
    }

    fn expected_tokens_unsafe(&self) -> Result<HashSet<TTokenKind>, ParsingError<TTokenKind>> {
        let mut result = HashSet::<TTokenKind>::new();

        for child_parser in self.choices.iter() {
            for token_kind in child_parser.expected_tokens()? {
                result.insert(token_kind);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, path::PathBuf};

    use crate::{
        domain::token::Token, lexing::token_reader::TokenReader,
        parsing::parser_token::ParserTokenBuilder, tree::visit::Visit,
    };

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
    enum TokenKindTest {
        A,
        B,
        C,
    }

    impl TokenKind for TokenKindTest {}

    #[derive(Debug, PartialEq, Eq, Clone)]
    enum TreeTest {
        A(Token<()>),
        B(Token<()>),
    }

    impl Tree for TreeTest {
        fn token(&self) -> &Token<()> {
            match &self {
                TreeTest::A(token) => token,
                TreeTest::B(token) => token,
            }
        }
    }

    impl Visit for TreeTest {
        fn visit(&self, _: crate::tree::visit::VisitCallback<Self>) {}

        fn visit_mut(&self, _: crate::tree::visit::VisitCallbackMut<Self>) {}
    }

    #[test]
    fn test_parser_sequential_end_of_source() {
        let tokens = Vec::new();
        let mut tokens_iter = tokens.iter();
        let mut token_reader = TokenReader::<TokenKindTest>::new(&mut tokens_iter);

        let parser = ParserChoiceBuilder::<TokenKindTest, TreeTest>::default()
            .choices(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::A)
                    .action(|token| TreeTest::A(token))
                    .build()
                    .unwrap(),
            ))
            .choices(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::B)
                    .action(|token| TreeTest::B(token))
                    .build()
                    .unwrap(),
            ))
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Err(ParsingError::UnexpectedEndOfSource {
                expected_token_kinds: HashSet::from([TokenKindTest::A, TokenKindTest::B]),
            })
        );
    }

    #[test]
    fn test_parser_sequential_unexpected_token() {
        let tokens = Vec::from([Token::new(
            PathBuf::from("--"),
            0,
            1,
            1,
            String::from("c"),
            TokenKindTest::C,
        )]);
        let mut tokens_iter = tokens.iter();
        let mut token_reader = TokenReader::<TokenKindTest>::new(&mut tokens_iter);

        let parser = ParserChoiceBuilder::<TokenKindTest, TreeTest>::default()
            .choices(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::A)
                    .action(|token| TreeTest::A(token))
                    .build()
                    .unwrap(),
            ))
            .choices(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::B)
                    .action(|token| TreeTest::B(token))
                    .build()
                    .unwrap(),
            ))
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Err(ParsingError::UnexpectedToken {
                expected_token_kinds: HashSet::from([TokenKindTest::A, TokenKindTest::B]),
                actual_token: Token::new(
                    PathBuf::from("--"),
                    0,
                    1,
                    1,
                    String::from("c"),
                    TokenKindTest::C
                )
            })
        );
    }

    #[test]
    fn test_parser_sequential_success_a() {
        let tokens = Vec::from([Token::new(
            PathBuf::from("--"),
            0,
            1,
            1,
            String::from("a"),
            TokenKindTest::A,
        )]);
        let mut tokens_iter = tokens.iter();
        let mut token_reader = TokenReader::<TokenKindTest>::new(&mut tokens_iter);

        let parser = ParserChoiceBuilder::<TokenKindTest, TreeTest>::default()
            .choices(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::A)
                    .action(|token| TreeTest::A(token))
                    .build()
                    .unwrap(),
            ))
            .choices(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::B)
                    .action(|token| TreeTest::B(token))
                    .build()
                    .unwrap(),
            ))
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Ok(TreeTest::A(Token::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("a"),
                ()
            )))
        );
    }

    #[test]
    fn test_parser_sequential_success_b() {
        let tokens = Vec::from([Token::new(
            PathBuf::from("--"),
            0,
            1,
            1,
            String::from("b"),
            TokenKindTest::B,
        )]);
        let mut tokens_iter = tokens.iter();
        let mut token_reader = TokenReader::<TokenKindTest>::new(&mut tokens_iter);

        let parser = ParserChoiceBuilder::<TokenKindTest, TreeTest>::default()
            .choices(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::A)
                    .action(|token| TreeTest::A(token))
                    .build()
                    .unwrap(),
            ))
            .choices(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::B)
                    .action(|token| TreeTest::B(token))
                    .build()
                    .unwrap(),
            ))
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Ok(TreeTest::B(Token::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("b"),
                ()
            )))
        );
    }
}
