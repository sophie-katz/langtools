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

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::result;

use crate::sourcing::sourcing_error::SourcingError;

use super::dfsa_error::DFSAError;

pub type Result<T> = result::Result<T, LexingError>;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum LexingError {
    DFSAError(DFSAError),
    SourcingError(SourcingError),
    UnexpectedCharacter(char),
    UnexpectedEndOfSource,
    DuplicateTrigger(String),
}

impl Error for LexingError {}

impl Display for LexingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LexingError::DFSAError(err) => write!(f, "dfsa error: {err}"),
            LexingError::SourcingError(err) => write!(f, "sourcing error: {err}"),
            LexingError::UnexpectedCharacter(value) => {
                write!(f, "unexpected character: {:#?}", value)
            }
            LexingError::UnexpectedEndOfSource => write!(f, "unexpected end of source"),
            LexingError::DuplicateTrigger(prefix) => write!(f, "duplicate trigger {prefix:#?}"),
        }
    }
}

impl From<DFSAError> for LexingError {
    fn from(other: DFSAError) -> Self {
        Self::DFSAError(other)
    }
}

impl From<SourcingError> for LexingError {
    fn from(other: SourcingError) -> Self {
        Self::SourcingError(other)
    }
}
