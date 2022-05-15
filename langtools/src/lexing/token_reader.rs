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

use crate::domain::token::{Token, TokenKind};

pub struct TokenReader<'iter, TTokenKind: TokenKind> {
    tokens: Vec<&'iter Token<TTokenKind>>,
    offset: usize,
    iter: &'iter mut dyn Iterator<Item = &'iter Token<TTokenKind>>,
}

impl<'iter, TTokenKind: TokenKind> TokenReader<'iter, TTokenKind> {
    pub fn new(iter: &'iter mut dyn Iterator<Item = &'iter Token<TTokenKind>>) -> Self {
        Self {
            tokens: Vec::new(),
            offset: 0,
            iter,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn seek(&mut self, offset: usize) {
        assert!(offset <= self.offset, "cannot seek forward");
        self.offset = offset;
    }

    pub fn has_more(&mut self) -> bool {
        self.ensure_buffer_is_filled();

        self.offset < self.tokens.len()
    }

    pub fn peek_next(&mut self) -> Option<&Token<TTokenKind>> {
        self.ensure_buffer_is_filled();

        if self.offset < self.tokens.len() {
            Some(self.tokens[self.offset])
        } else {
            None
        }
    }

    pub fn eat_next(&mut self) -> Option<&Token<TTokenKind>> {
        self.ensure_buffer_is_filled();

        if self.offset < self.tokens.len() {
            let result = Some(self.tokens[self.offset]);
            self.offset += 1;
            result
        } else {
            None
        }
    }

    fn ensure_buffer_is_filled(&mut self) {
        if self.offset >= self.tokens.len() {
            if let Some(token) = self.iter.next() {
                self.tokens.push(token);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
    enum TokenKindTest {
        A,
        B,
    }

    impl TokenKind for TokenKindTest {}

    #[test]
    fn test_token_reader_empty() {
        let tokens = Vec::<Token<TokenKindTest>>::new();
        let mut tokens_iter = tokens.iter();
        let mut token_reader = TokenReader::<TokenKindTest>::new(&mut tokens_iter);

        assert_eq!(token_reader.offset(), 0);
        assert!(!token_reader.has_more());
        assert_eq!(token_reader.peek_next(), None);
        assert_eq!(token_reader.eat_next(), None);
    }

    #[test]
    fn test_token_reader_one() {
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

        assert_eq!(token_reader.offset(), 0);
        assert!(token_reader.has_more());
        assert_eq!(
            token_reader.peek_next().map(|token| token.to_owned()),
            Some(Token::<TokenKindTest>::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("a"),
                TokenKindTest::A
            ))
        );
        assert_eq!(
            token_reader.eat_next().map(|token| token.to_owned()),
            Some(Token::<TokenKindTest>::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("a"),
                TokenKindTest::A
            ))
        );

        assert_eq!(token_reader.offset(), 1);
        assert!(!token_reader.has_more());
        assert_eq!(token_reader.peek_next(), None);
        assert_eq!(token_reader.eat_next(), None);
    }

    #[test]
    fn test_token_reader_two() {
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

        assert_eq!(token_reader.offset(), 0);
        assert!(token_reader.has_more());
        assert_eq!(
            token_reader.peek_next().map(|token| token.to_owned()),
            Some(Token::<TokenKindTest>::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("a"),
                TokenKindTest::A
            ))
        );
        assert_eq!(
            token_reader.eat_next().map(|token| token.to_owned()),
            Some(Token::<TokenKindTest>::new(
                PathBuf::from("--"),
                0,
                1,
                1,
                String::from("a"),
                TokenKindTest::A
            ))
        );

        assert_eq!(token_reader.offset(), 1);
        assert!(token_reader.has_more());
        assert_eq!(
            token_reader.peek_next().map(|token| token.to_owned()),
            Some(Token::<TokenKindTest>::new(
                PathBuf::from("--"),
                1,
                2,
                2,
                String::from("b"),
                TokenKindTest::B
            ))
        );
        assert_eq!(
            token_reader.eat_next().map(|token| token.to_owned()),
            Some(Token::<TokenKindTest>::new(
                PathBuf::from("--"),
                1,
                2,
                2,
                String::from("b"),
                TokenKindTest::B
            ))
        );

        assert_eq!(token_reader.offset(), 2);
        assert!(!token_reader.has_more());
        assert_eq!(token_reader.peek_next(), None);
        assert_eq!(token_reader.eat_next(), None);
    }
}
