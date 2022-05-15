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
use crate::{
    domain::token::{Token, TokenKind},
    tree::Tree,
};
use std::{collections::HashSet, mem};

pub type ParserSequentialAction<TTree> = fn(Token<()>, Vec<TTree>) -> TTree;

pub struct ParserSequential<TTokenKind: TokenKind, TTree: Tree> {
    sequence: Vec<Box<dyn Parse<TTokenKind, TTree>>>,
    action: ParserSequentialAction<TTree>,
}

pub struct ParserSequentialBuilder<TTokenKind: TokenKind, TTree: Tree> {
    sequence: Vec<Box<dyn Parse<TTokenKind, TTree>>>,
    action: Option<ParserSequentialAction<TTree>>,
}

impl<TTokenKind: TokenKind, TTree: Tree> ParserSequentialBuilder<TTokenKind, TTree> {
    pub fn sequence(&mut self, value: Box<dyn Parse<TTokenKind, TTree>>) -> &mut Self {
        self.sequence.push(value);
        self
    }

    pub fn action(&mut self, value: ParserSequentialAction<TTree>) -> &mut Self {
        self.action = Some(value);
        self
    }

    pub fn build(
        &mut self,
    ) -> Result<ParserSequential<TTokenKind, TTree>, ParsingError<TTokenKind>> {
        if self.sequence.is_empty() {
            return Err(ParsingError::RequiredParserFieldMissing("sequence"));
        }

        Ok(ParserSequential {
            sequence: mem::take(&mut self.sequence),
            action: self
                .action
                .ok_or(ParsingError::RequiredParserFieldMissing("action"))?,
        })
    }
}

impl<TTokenKind: TokenKind, TTree: Tree> Default for ParserSequentialBuilder<TTokenKind, TTree> {
    fn default() -> Self {
        Self {
            sequence: Vec::new(),
            action: None,
        }
    }
}

impl<TTokenKind: TokenKind, TTree: Tree> Parse<TTokenKind, TTree>
    for ParserSequential<TTokenKind, TTree>
{
    fn parse(
        &self,
        token_reader: &mut crate::lexing::token_reader::TokenReader<TTokenKind>,
    ) -> Result<TTree, ParsingError<TTokenKind>> {
        let mut children = Vec::<TTree>::new();

        let mut token: Option<Token<()>> = None;

        for child_parser in self.sequence.iter() {
            match child_parser.parse(token_reader) {
                Ok(child) => {
                    if token.is_none() {
                        token = Some(child.token().clone());
                    }

                    children.push(child);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok((self.action)(
            token.ok_or(ParsingError::RequiredParserFieldMissing("sequence"))?,
            children,
        ))
    }

    fn expected_tokens_unsafe(&self) -> Result<HashSet<TTokenKind>, ParsingError<TTokenKind>> {
        self.sequence
            .first()
            .ok_or(ParsingError::RequiredParserFieldMissing("sequence"))?
            .expected_tokens()
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
    }

    impl TokenKind for TokenKindTest {}

    #[derive(Debug, PartialEq, Eq, Clone)]
    enum TreeTest {
        A(Token<()>),
        B(Token<()>),
        AB(Token<()>, Box<TreeTest>, Box<TreeTest>),
    }

    impl Tree for TreeTest {
        fn token(&self) -> &Token<()> {
            match &self {
                TreeTest::A(token) => token,
                TreeTest::B(token) => token,
                TreeTest::AB(token, _, _) => token,
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

        let parser = ParserSequentialBuilder::<TokenKindTest, TreeTest>::default()
            .sequence(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::A)
                    .action(|token| TreeTest::A(token))
                    .build()
                    .unwrap(),
            ))
            .sequence(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::B)
                    .action(|token| TreeTest::B(token))
                    .build()
                    .unwrap(),
            ))
            .action(|token, children| {
                let mut iter = children.into_iter();

                TreeTest::AB(
                    token,
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                )
            })
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
    fn test_parser_sequential_unexpected_first_token() {
        let tokens = Vec::from([
            Token::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("b"),
                TokenKindTest::B,
            ),
            Token::new(
                PathBuf::from("--"),
                1,
                2,
                2,
                String::from("b"),
                TokenKindTest::B,
            ),
        ]);
        let mut tokens_iter = tokens.iter();
        let mut token_reader = TokenReader::<TokenKindTest>::new(&mut tokens_iter);

        let parser = ParserSequentialBuilder::<TokenKindTest, TreeTest>::default()
            .sequence(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::A)
                    .action(|token| TreeTest::A(token))
                    .build()
                    .unwrap(),
            ))
            .sequence(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::B)
                    .action(|token| TreeTest::B(token))
                    .build()
                    .unwrap(),
            ))
            .action(|token, children| {
                let mut iter = children.into_iter();

                TreeTest::AB(
                    token,
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                )
            })
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Err(ParsingError::UnexpectedToken {
                expected_token_kinds: HashSet::from([TokenKindTest::A]),
                actual_token: Token::new(
                    PathBuf::from("--"),
                    0,
                    1,
                    1,
                    String::from("b"),
                    TokenKindTest::B
                )
            })
        );
    }

    #[test]
    fn test_parser_sequential_unexpected_second_token() {
        let tokens = Vec::from([
            Token::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("a"),
                TokenKindTest::A,
            ),
            Token::new(
                PathBuf::from("--"),
                1,
                2,
                2,
                String::from("a"),
                TokenKindTest::A,
            ),
        ]);
        let mut tokens_iter = tokens.iter();
        let mut token_reader = TokenReader::<TokenKindTest>::new(&mut tokens_iter);

        let parser = ParserSequentialBuilder::<TokenKindTest, TreeTest>::default()
            .sequence(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::A)
                    .action(|token| TreeTest::A(token))
                    .build()
                    .unwrap(),
            ))
            .sequence(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::B)
                    .action(|token| TreeTest::B(token))
                    .build()
                    .unwrap(),
            ))
            .action(|token, children| {
                let mut iter = children.into_iter();

                TreeTest::AB(
                    token,
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                )
            })
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Err(ParsingError::UnexpectedToken {
                expected_token_kinds: HashSet::from([TokenKindTest::B]),
                actual_token: Token::new(
                    PathBuf::from("--"),
                    1,
                    2,
                    2,
                    String::from("a"),
                    TokenKindTest::A
                )
            })
        );
    }

    #[test]
    fn test_parser_sequential_success() {
        let tokens = Vec::from([
            Token::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("a"),
                TokenKindTest::A,
            ),
            Token::new(
                PathBuf::from("--"),
                1,
                2,
                2,
                String::from("b"),
                TokenKindTest::B,
            ),
        ]);
        let mut tokens_iter = tokens.iter();
        let mut token_reader = TokenReader::<TokenKindTest>::new(&mut tokens_iter);

        let parser = ParserSequentialBuilder::<TokenKindTest, TreeTest>::default()
            .sequence(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::A)
                    .action(|token| TreeTest::A(token))
                    .build()
                    .unwrap(),
            ))
            .sequence(Box::new(
                ParserTokenBuilder::<TokenKindTest, TreeTest>::default()
                    .token_kind(TokenKindTest::B)
                    .action(|token| TreeTest::B(token))
                    .build()
                    .unwrap(),
            ))
            .action(|token, children| {
                let mut iter = children.into_iter();

                TreeTest::AB(
                    token,
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                )
            })
            .build()
            .unwrap();

        assert_eq!(
            parser.parse(&mut token_reader),
            Ok(TreeTest::AB(
                Token::new(PathBuf::from("--"), 0, 1, 1, String::from("a"), ()),
                Box::new(TreeTest::A(Token::new(
                    PathBuf::from("--"),
                    0,
                    1,
                    1,
                    String::from("a"),
                    ()
                ))),
                Box::new(TreeTest::B(Token::new(
                    PathBuf::from("--"),
                    1,
                    2,
                    2,
                    String::from("b"),
                    ()
                )))
            ))
        );
    }
}
