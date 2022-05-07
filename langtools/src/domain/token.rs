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

use super::source_location::{Column, Line, Offset, SourceLocation};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Token<TKind> {
    pub location: SourceLocation,
    pub text: String,
    pub kind: TKind,
}

impl<TKind> Token<TKind> {
    pub fn new(
        path: PathBuf,
        offset: Offset,
        line: Line,
        column: Column,
        text: String,
        kind: TKind,
    ) -> Self {
        Self::new_from_location(SourceLocation::new(path, offset, line, column), text, kind)
    }

    pub fn new_from_location(location: SourceLocation, text: String, kind: TKind) -> Self {
        Self {
            location,
            text,
            kind,
        }
    }

    pub fn to_kindless(&self) -> Token<()> {
        Token::new_from_location(self.location.clone(), self.text.clone(), ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_to_kindless() {
        let token_with_kind =
            Token::<&str>::new(PathBuf::from("--"), 0, 1, 1, String::from("hi"), "bye");
        let token_without_kind: Token<()> = token_with_kind.to_kindless();

        assert_eq!(token_without_kind.location, token_with_kind.location);
        assert_eq!(token_without_kind.text, token_with_kind.text);
    }
}
