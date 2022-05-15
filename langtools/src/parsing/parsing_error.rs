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

use std::fmt::{self, Display, Formatter};
use std::{collections::HashSet, error::Error};

use crate::domain::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum ParsingError<TTokenKind: TokenKind> {
    UnexpectedEndOfSource {
        expected_token_kinds: HashSet<TTokenKind>,
    },
    UnexpectedToken {
        expected_token_kinds: HashSet<TTokenKind>,
        actual_token: Token<TTokenKind>,
    },
    RequiredParserFieldMissing(&'static str),
    NoExpectedTokensProvided,
}

impl<TTokenKind: TokenKind> PartialEq for ParsingError<TTokenKind> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::UnexpectedEndOfSource {
                    expected_token_kinds: expected_token_kinds_self,
                },
                Self::UnexpectedEndOfSource {
                    expected_token_kinds: expected_token_kinds_other,
                },
            ) => {
                expected_token_kinds_self.len() == expected_token_kinds_other.len()
                    && expected_token_kinds_self.is_subset(expected_token_kinds_other)
            }
            (
                Self::UnexpectedToken {
                    expected_token_kinds: expected_token_kinds_self,
                    actual_token: actual_token_self,
                },
                Self::UnexpectedToken {
                    expected_token_kinds: expected_token_kinds_other,
                    actual_token: actual_token_other,
                },
            ) => {
                expected_token_kinds_self.len() == expected_token_kinds_other.len()
                    && actual_token_self == actual_token_other
                    && expected_token_kinds_self.is_subset(expected_token_kinds_other)
            }
            (
                Self::RequiredParserFieldMissing(name_self),
                Self::RequiredParserFieldMissing(name_other),
            ) => name_self == name_other,
            (Self::NoExpectedTokensProvided, Self::NoExpectedTokensProvided) => true,
            _ => false,
        }
    }
}

impl<TTokenKind: TokenKind> Error for ParsingError<TTokenKind> {}

impl<TTokenKind: TokenKind> Display for ParsingError<TTokenKind> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParsingError::UnexpectedEndOfSource {
                expected_token_kinds,
            } => write!(
                f,
                "unexpected end of source, expected {expected_token_kinds:?}"
            ),
            ParsingError::UnexpectedToken {
                expected_token_kinds,
                actual_token,
            } => write!(f, "expected {expected_token_kinds:?}, not {actual_token:?}"),
            ParsingError::RequiredParserFieldMissing(name) => {
                write!(f, "required parser field {name:?} missing")
            }
            ParsingError::NoExpectedTokensProvided => {
                write!(f, "no expected tokens provided")
            }
        }
    }
}
