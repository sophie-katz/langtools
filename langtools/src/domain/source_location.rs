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

use super::source_info::SourceInfo;
use std::path::PathBuf;

pub type Offset = usize;
pub type Line = u32;
pub type Column = Line;

pub const OFFSET_INITIAL: Offset = 0;
pub const LINE_INITIAL: Line = 1;
pub const COLUMN_INITIAL: Column = 1;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SourceLocation {
    pub info: SourceInfo,
    pub offset: Offset,
    pub line: Line,
    pub column: Column,
}

impl SourceLocation {
    pub fn new(path: PathBuf, offset: Offset, line: Line, column: Column) -> Self {
        Self::new_from_info(SourceInfo::new(path), offset, line, column)
    }

    pub fn new_from_info(info: SourceInfo, offset: Offset, line: Line, column: Column) -> Self {
        Self {
            info,
            offset,
            line,
            column,
        }
    }
}
