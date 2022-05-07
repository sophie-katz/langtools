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
