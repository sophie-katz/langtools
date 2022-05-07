use super::sourcing_error::Result;
use crate::domain::source_info::SourceInfo;

pub trait ReadSource {
    fn info(&self) -> &SourceInfo;
    fn offset(&self) -> usize;
    fn has_more(&mut self) -> bool;
    fn peek_next(&mut self) -> Result<char>;
    fn eat_next(&mut self) -> Result<char>;
}
