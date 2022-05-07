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

use super::dfsa_types::DFSAId;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::result;

pub type Result<TValue> = result::Result<TValue, DFSAError>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DFSAError {
    NoStartId,
    OutOfRangeId(DFSAId),
    StateHasNoAction(DFSAId),
    TransitionAlreadyExists,
    NoSuchTransition,
}

impl Error for DFSAError {}

impl Display for DFSAError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DFSAError::NoStartId => write!(f, "no start id specified"),
            DFSAError::OutOfRangeId(id) => write!(f, "out of range id: {}", id),
            DFSAError::StateHasNoAction(id) => write!(f, "state {} has no action", id),
            DFSAError::TransitionAlreadyExists => write!(f, "transition already exists"),
            DFSAError::NoSuchTransition => write!(f, "no such transition exists on element"),
        }
    }
}
