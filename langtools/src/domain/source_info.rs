use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SourceInfo {
    pub path: PathBuf,
}

impl SourceInfo {
    pub fn new(path: PathBuf) -> Self {
        SourceInfo { path }
    }
}
