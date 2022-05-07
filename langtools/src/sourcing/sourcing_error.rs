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

pub type Result<TValue> = result::Result<TValue, SourcingError>;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum SourcingError {
    NoMoreChars,
    BufferingAlreadyEnabled,
    BufferingAlreadyDisabled,
    BufferingNeedsToBeEnabled,
}

impl Error for SourcingError {}

impl Display for SourcingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SourcingError::NoMoreChars => write!(f, "no more characters to be read"),
            SourcingError::BufferingAlreadyEnabled => write!(f, "buffering already enabled"),
            SourcingError::BufferingAlreadyDisabled => write!(f, "buffering already disabled"),
            SourcingError::BufferingNeedsToBeEnabled => {
                write!(f, "buffering must be enabled for it to be accessed")
            }
        }
    }
}
