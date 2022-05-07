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
