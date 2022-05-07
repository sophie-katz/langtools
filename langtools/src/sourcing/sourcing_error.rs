use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::result;

pub type Result<TValue> = result::Result<TValue, SourcingError>;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum SourcingError {
    NoMoreChars,
    BufferingAlreadyEnabled,
    BufferingAlreadyDisabled,
    BufferingNeedsToBeEnabled,
}

impl Error for SourcingError {}

impl Display for SourcingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SourcingError::NoMoreChars => write!(f, "no more characters to be read"),
            SourcingError::BufferingAlreadyEnabled => write!(f, "buffering already enabled"),
            SourcingError::BufferingAlreadyDisabled => write!(f, "buffering already disabled"),
            SourcingError::BufferingNeedsToBeEnabled => {
                write!(f, "buffering must be enabled for it to be accessed")
            }
        }
    }
}
