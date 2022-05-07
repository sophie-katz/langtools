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
