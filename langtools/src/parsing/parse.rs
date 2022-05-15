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

use super::parsing_error::ParsingError;
use crate::{domain::token::TokenKind, lexing::token_reader::TokenReader, tree::Tree};

pub trait Parse<TTokenKind: TokenKind, TTree: Tree> {
    fn parse(
        &self,
        token_reader: &mut TokenReader<TTokenKind>,
    ) -> Result<TTree, ParsingError<TTokenKind>>;

    fn expected_tokens_unsafe(&self) -> Result<HashSet<TTokenKind>, ParsingError<TTokenKind>>;

    fn expected_tokens(&self) -> Result<HashSet<TTokenKind>, ParsingError<TTokenKind>> {
        let result = self.expected_tokens_unsafe()?;

        if result.is_empty() {
            Err(ParsingError::NoExpectedTokensProvided)
        } else {
            Ok(result)
        }
    }
}
