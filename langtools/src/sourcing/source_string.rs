use crate::domain::source_info::SourceInfo;
use std::iter::Peekable;
use std::str::Chars;

use super::read_source::ReadSource;
use super::sourcing_error::{Result, SourcingError};

pub struct SourceString<'string> {
    info: SourceInfo,
    iter: Peekable<Chars<'string>>,
    offset: usize,
}

impl<'string> SourceString<'string> {
    pub fn new(info: SourceInfo, data: &'string str) -> Self {
        Self {
            info,
            iter: data.chars().peekable(),
            offset: 0,
        }
    }
}

impl<'string> ReadSource for SourceString<'string> {
    fn info(&self) -> &SourceInfo {
        &self.info
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn has_more(&mut self) -> bool {
        self.iter.peek().is_some()
    }

    fn peek_next(&mut self) -> Result<char> {
        self.iter.peek().ok_or(SourcingError::NoMoreChars).copied()
    }

    fn eat_next(&mut self) -> Result<char> {
        self.offset += 1;
        self.iter.next().ok_or(SourcingError::NoMoreChars)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_source_string_empty() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "");

        assert_eq!(source.info().path, PathBuf::from("--"));
        assert_eq!(source.offset(), 0);
        assert!(!source.has_more());
        assert_eq!(source.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_string_one() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "a");

        assert_eq!(source.info().path, PathBuf::from("--"));
        assert_eq!(source.offset(), 0);
        assert!(source.has_more());
        assert_eq!(source.peek_next(), Ok('a'));
        assert_eq!(source.eat_next(), Ok('a'));

        assert_eq!(source.info().path, PathBuf::from("--"));
        assert_eq!(source.offset(), 1);
        assert!(!source.has_more());
        assert_eq!(source.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source.eat_next(), Err(SourcingError::NoMoreChars));
    }

    #[test]
    fn test_source_string_two() {
        let mut source = SourceString::new(SourceInfo::new(PathBuf::from("--")), "ab");

        assert_eq!(source.info().path, PathBuf::from("--"));
        assert_eq!(source.offset(), 0);
        assert!(source.has_more());
        assert_eq!(source.peek_next(), Ok('a'));
        assert_eq!(source.eat_next(), Ok('a'));

        assert_eq!(source.info().path, PathBuf::from("--"));
        assert_eq!(source.offset(), 1);
        assert!(source.has_more());
        assert_eq!(source.peek_next(), Ok('b'));
        assert_eq!(source.eat_next(), Ok('b'));

        assert_eq!(source.info().path, PathBuf::from("--"));
        assert_eq!(source.offset(), 2);
        assert!(!source.has_more());
        assert_eq!(source.peek_next(), Err(SourcingError::NoMoreChars));
        assert_eq!(source.eat_next(), Err(SourcingError::NoMoreChars));
    }
}
