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

use super::{
    read_source::ReadSource,
    sourcing_error::{self, SourcingError},
};
use crate::domain::{
    source_info::SourceInfo,
    source_location::{SourceLocation, COLUMN_INITIAL, LINE_INITIAL, OFFSET_INITIAL},
};

#[readonly::make]
pub struct SourceReader<'source> {
    source: &'source mut dyn ReadSource,
    pub location: SourceLocation,
    buffer: Option<String>,
}

impl<'source> SourceReader<'source> {
    pub fn new(source: &'source mut dyn ReadSource) -> Self {
        let info = source.info().clone();

        Self::new_with_location(
            source,
            SourceLocation::new_from_info(info, OFFSET_INITIAL, LINE_INITIAL, COLUMN_INITIAL),
        )
    }

    pub fn new_with_location(
        source: &'source mut dyn ReadSource,
        location: SourceLocation,
    ) -> Self {
        Self {
            source,
            location,
            buffer: None,
        }
    }

    pub fn is_buffering_enabled(&self) -> bool {
        self.buffer.is_some()
    }

    pub fn enable_buffering(&mut self) -> sourcing_error::Result<()> {
        if self.buffer.is_some() {
            Err(SourcingError::BufferingAlreadyEnabled)
        } else {
            self.buffer = Some(String::new());
            Ok(())
        }
    }

    pub fn disable_buffering(&mut self) -> sourcing_error::Result<()> {
        if self.buffer.is_none() {
            Err(SourcingError::BufferingAlreadyDisabled)
        } else {
            self.buffer = None;
            Ok(())
        }
    }

    pub fn is_buffer_empty(&self) -> bool {
        self.buffer
            .as_ref()
            .map(|buffer| buffer.is_empty())
            .unwrap_or(true)
    }

    pub fn clear_buffer(&mut self) -> sourcing_error::Result<()> {
        self.buffer
            .as_mut()
            .ok_or(SourcingError::BufferingNeedsToBeEnabled)?
            .clear();

        Ok(())
    }

    pub fn pop_buffer(&mut self) -> sourcing_error::Result<String> {
        self.buffer
            .replace(String::new())
            .ok_or(SourcingError::BufferingNeedsToBeEnabled)
    }

    fn eat_next_helper_fold_newlines(&mut self) -> sourcing_error::Result<char> {
        let result = self.source.eat_next()?;

        if result == '\r' {
            match self.source.peek_next() {
                Ok('\r') | Ok('\n') => {
                    self.source.eat_next()?;
                    Ok('\n')
                }
                Ok(_) => Ok('\n'),
                error => error,
            }
        } else {
            Ok(result)
        }
    }

    fn eat_next_helper_update_location(&mut self) -> sourcing_error::Result<char> {
        let result = self.eat_next_helper_fold_newlines()?;

        assert!(result != '\r');

        if result == '\n' {
            self.location.line += 1;
            self.location.column = COLUMN_INITIAL;
        } else {
            self.location.column += 1;
        }

        self.location.offset = self.source.offset();

        Ok(result)
    }

    fn eat_next_helper_push_buffer(&mut self) -> sourcing_error::Result<char> {
        let result = self.eat_next_helper_update_location()?;

        if let Some(ref mut buffer) = self.buffer {
            buffer.push(result);
        }

        Ok(result)
    }
}

impl<'source> ReadSource for SourceReader<'source> {
    fn info(&self) -> &SourceInfo {
        self.source.info()
    }

    fn offset(&self) -> usize {
        self.source.offset()
    }

    fn has_more(&mut self) -> bool {
        self.source.has_more()
    }

    fn peek_next(&mut self) -> sourcing_error::Result<char> {
        let result = self.source.peek_next();
        match result {
            Ok('\r') => Ok('\n'),
            _ => result,
        }
    }

    fn eat_next(&mut self) -> sourcing_error::Result<char> {
        self.eat_next_helper_push_buffer()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        super::{source_string::SourceString, sourcing_error::SourcingError},
        *,
    };

    #[test]
    fn test_source_reader_empty() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_one_line() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "ab");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('a'));
        assert_eq!(source_reader.eat_next(), Ok('a'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 1);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 1, 1, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('b'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 1, 3)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_one_newline_unix() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "\n");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 1);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 1, 2, 1)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_one_newline_dos() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "\r\n");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 2, 1)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_one_newline_mac() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "\r\r");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 2, 1)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_one_newline_unix_then_line() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "\nab");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 1);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 1, 2, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('a'));
        assert_eq!(source_reader.eat_next(), Ok('a'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 2, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('b'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 3);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 3, 2, 3)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_one_newline_dos_then_line() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "\r\nab");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 2, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('a'));
        assert_eq!(source_reader.eat_next(), Ok('a'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 3);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 3, 2, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('b'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 4);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 4, 2, 3)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_one_newline_mac_then_line() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "\r\rab");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 2, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('a'));
        assert_eq!(source_reader.eat_next(), Ok('a'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 3);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 3, 2, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('b'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 4);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 4, 2, 3)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_line_then_newline_unix_then_line() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "ab\ncd");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('a'));
        assert_eq!(source_reader.eat_next(), Ok('a'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 1);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 1, 1, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('b'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 1, 3)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 3);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 3, 2, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('c'));
        assert_eq!(source_reader.eat_next(), Ok('c'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 4);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 4, 2, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('d'));
        assert_eq!(source_reader.eat_next(), Ok('d'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 5);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 5, 2, 3)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_line_then_newline_dos_then_line() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "ab\r\ncd");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('a'));
        assert_eq!(source_reader.eat_next(), Ok('a'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 1);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 1, 1, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('b'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 1, 3)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 4);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 4, 2, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('c'));
        assert_eq!(source_reader.eat_next(), Ok('c'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 5);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 5, 2, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('d'));
        assert_eq!(source_reader.eat_next(), Ok('d'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 6);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 6, 2, 3)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_line_then_newline_mac_then_line() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "ab\r\rcd");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('a'));
        assert_eq!(source_reader.eat_next(), Ok('a'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 1);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 1, 1, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('b'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 1, 3)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 4);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 4, 2, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('c'));
        assert_eq!(source_reader.eat_next(), Ok('c'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 5);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 5, 2, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('d'));
        assert_eq!(source_reader.eat_next(), Ok('d'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 6);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 6, 2, 3)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_mixed_newlines() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "a\nb\r\nc\r\rd");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 0);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 0, 1, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('a'));
        assert_eq!(source_reader.eat_next(), Ok('a'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 1);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 1, 1, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 2);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 2, 2, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('b'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 3);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 3, 2, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 5);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 5, 3, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('c'));
        assert_eq!(source_reader.eat_next(), Ok('c'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 6);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 6, 3, 2)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('\n'));
        assert_eq!(source_reader.eat_next(), Ok('\n'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 8);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 8, 4, 1)
        );
        assert!(source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Ok('d'));
        assert_eq!(source_reader.eat_next(), Ok('d'));

        assert_eq!(source_reader.info().path, PathBuf::from("--"));
        assert_eq!(source_reader.offset(), 9);
        assert_eq!(
            source_reader.location,
            SourceLocation::new(PathBuf::from("--"), 9, 4, 2)
        );
        assert!(!source_reader.has_more());
        assert_eq!(source_reader.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source_reader.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_reader_buffer_disabled() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "abc");
        let mut source_reader = SourceReader::new(&mut source);

        assert!(!source_reader.is_buffering_enabled());
        assert!(source_reader.is_buffer_empty());
        assert_eq!(
            source_reader.disable_buffering(),
            Err(SourcingError::BufferingAlreadyDisabled)
        );
        assert_eq!(
            source_reader.clear_buffer(),
            Err(SourcingError::BufferingNeedsToBeEnabled)
        );
        assert_eq!(
            source_reader.pop_buffer(),
            Err(SourcingError::BufferingNeedsToBeEnabled)
        );
    }

    #[test]
    fn test_source_reader_buffer_enabled() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "abc");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.enable_buffering(), Ok(()));
        assert_eq!(
            source_reader.enable_buffering(),
            Err(SourcingError::BufferingAlreadyEnabled)
        );

        assert!(source_reader.is_buffering_enabled());
        assert!(source_reader.is_buffer_empty());

        assert_eq!(source_reader.disable_buffering(), Ok(()));
        assert_eq!(
            source_reader.disable_buffering(),
            Err(SourcingError::BufferingAlreadyDisabled)
        );
    }

    #[test]
    fn test_source_reader_buffer_pop() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "abc");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.enable_buffering(), Ok(()));

        assert_eq!(source_reader.eat_next(), Ok('a'));
        assert_eq!(source_reader.eat_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('c'));

        assert_eq!(source_reader.pop_buffer(), Ok(String::from("abc")));
        assert_eq!(source_reader.pop_buffer(), Ok(String::new()));
    }

    #[test]
    fn test_source_reader_buffer_clear_then_pop() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "abc");
        let mut source_reader = SourceReader::new(&mut source);

        assert_eq!(source_reader.enable_buffering(), Ok(()));

        assert_eq!(source_reader.eat_next(), Ok('a'));

        assert_eq!(source_reader.clear_buffer(), Ok(()));

        assert_eq!(source_reader.eat_next(), Ok('b'));
        assert_eq!(source_reader.eat_next(), Ok('c'));

        assert_eq!(source_reader.pop_buffer(), Ok(String::from("bc")));
    }
}
