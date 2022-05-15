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

use std::collections::HashSet;

use super::{parse::Parse, parsing_error::ParsingError};
use crate::{
    domain::token::{Token, TokenKind},
    tree::Tree,
};

pub type ParserTokenAction<TTree> = fn(Token<()>) -> TTree;

pub struct ParserToken<TTokenKind: TokenKind, TTree: Tree> {
    token_kind: TTokenKind,
    action: ParserTokenAction<TTree>,
}

pub struct ParserTokenBuilder<TTokenKind: TokenKind, TTree: Tree> {
    token_kind: Option<TTokenKind>,
    action: Option<ParserTokenAction<TTree>>,
}

impl<TTokenKind: TokenKind, TTree: Tree> ParserTokenBuilder<TTokenKind, TTree> {
    pub fn token_kind(&mut self, value: TTokenKind) -> &mut Self {
        self.token_kind = Some(value);
        self
    }

    pub fn action(&mut self, value: ParserTokenAction<TTree>) -> &mut Self {
        self.action = Some(value);
        self
    }

    pub fn build(&mut self) -> Result<ParserToken<TTokenKind, TTree>, ParsingError<TTokenKind>> {
        Ok(ParserToken {
            token_kind: self
                .token_kind
                .ok_or(ParsingError::RequiredParserFieldMissing("token_kind"))?,
            action: self
                .action
                .ok_or(ParsingError::RequiredParserFieldMissing("action"))?,
        })
    }
}

impl<TTokenKind: TokenKind, TTree: Tree> Default for ParserTokenBuilder<TTokenKind, TTree> {
    fn default() -> Self {
        Self {
            token_kind: None,
            action: None,
        }
    }
}

impl<TTokenKind: TokenKind, TTree: Tree> Parse<TTokenKind, TTree>
    for ParserToken<TTokenKind, TTree>
{
    fn parse(
        &self,
        token_reader: &mut crate::lexing::token_reader::TokenReader<TTokenKind>,
    ) -> Result<TTree, ParsingError<TTokenKind>> {
        if let Some(token) = token_reader.eat_next() {
            if token.kind == self.token_kind {
                Ok((self.action)(token.to_kindless()))
            } else {
                Err(ParsingError::UnexpectedToken {
                    expected_token_kinds: self.expected_tokens()?,
                    actual_token: token.clone(),
                })
            }
        } else {
            Err(ParsingError::UnexpectedEndOfSource {
                expected_token_kinds: self.expected_tokens()?,
            })
        }
    }

    fn expected_tokens_unsafe(&self) -> Result<HashSet<TTokenKind>, ParsingError<TTokenKind>> {
        Ok(HashSet::from([self.token_kind]))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{domain::token::Token, lexing::token_reader::TokenReader, tree::visit::Visit};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
    enum TokenKindTest {
        A,
        B,
    }

    impl TokenKind for TokenKindTest {}

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct TreeTest {
        token: Token<()>,
    }

    impl Tree for TreeTest {
        fn token(&self) -> &Token<()> {
            &self.token
        }
    }

    impl Visit for TreeTest {
        fn visit(&self, _: crate::tree::visit::VisitCallback<Self>) {}

        fn visit_mut(&self, _: crate::tree::visit::VisitCallbackMut<Self>) {}
    }

    #[test]
    fn test_parser_token_end_of_source() {
        let tokens = Vec::new();
        let mut tokens_iter = tokens.iter();
        let mut token_reader = TokenReader::<TokenKindTest>::new(&mut tokens_iter);

        let parser = ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
            .token_kind(TokenKindTest::A)
            .action(|token| TreeTest { token })
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Err(ParsingError::UnexpectedEndOfSource {
                expected_token_kinds: HashSet::from([TokenKindTest::A]),
            })
        );
    }

    #[test]
    fn test_parser_token_unexpected_token() {
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

        let parser = ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
            .token_kind(TokenKindTest::B)
            .action(|token| TreeTest { token })
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Err(ParsingError::UnexpectedToken {
                expected_token_kinds: HashSet::from([TokenKindTest::B]),
                actual_token: tokens[0].clone(),
            })
        );
    }

    #[test]
    fn test_parser_token_success() {
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

        let parser = ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
            .token_kind(TokenKindTest::A)
            .action(|token| TreeTest { token })
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Ok(TreeTest {
                token: Token::new(PathBuf::from("--"), 0, 1, 1, String::from("a"), ())
            })
        );
    }
}
