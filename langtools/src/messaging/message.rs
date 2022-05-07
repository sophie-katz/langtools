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

use crate::domain::{source_info::SourceInfo, source_location::SourceLocation, token::Token};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Severity {
    Note,
    Info,
    Warning,
    Error,
    FatalError,
    InternalWarning,
    InternalError,
}

#[derive(Debug, Clone)]
pub enum MessageSource {
    Global,
    Source { source: SourceInfo },
    Location { location: SourceLocation },
    Token { token: Token<()> },
}

#[derive(Debug, Clone)]
pub struct Message {
    pub source: MessageSource,
    pub severity: Severity,
    pub description: String,
}

impl Message {
    pub fn new_global(severity: Severity, description: String) -> Self {
        Self {
            source: MessageSource::Global,
            severity,
            description,
        }
    }

    pub fn new_source(source: SourceInfo, severity: Severity, description: String) -> Self {
        Self {
            source: MessageSource::Source { source },
            severity,
            description,
        }
    }

    pub fn new_location(location: SourceLocation, severity: Severity, description: String) -> Self {
        Self {
            source: MessageSource::Location { location },
            severity,
            description,
        }
    }

    pub fn new_token(token: Token<()>, severity: Severity, description: String) -> Self {
        Self {
            source: MessageSource::Token { token },
            severity,
            description,
        }
    }
}
