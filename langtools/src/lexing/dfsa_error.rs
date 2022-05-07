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
