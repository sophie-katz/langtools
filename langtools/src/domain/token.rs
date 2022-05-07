use super::source_location::{Column, Line, Offset, SourceLocation};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Token<TKind> {
    pub location: SourceLocation,
    pub text: String,
    pub kind: TKind,
}

impl<TKind> Token<TKind> {
    pub fn new(
        path: PathBuf,
        offset: Offset,
        line: Line,
        column: Column,
        text: String,
        kind: TKind,
    ) -> Self {
        Self::new_from_location(SourceLocation::new(path, offset, line, column), text, kind)
    }

    pub fn new_from_location(location: SourceLocation, text: String, kind: TKind) -> Self {
        Self {
            location,
            text,
            kind,
        }
    }

    pub fn to_kindless(&self) -> Token<()> {
        Token::new_from_location(self.location.clone(), self.text.clone(), ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_to_kindless() {
        let token_with_kind =
            Token::<&str>::new(PathBuf::from("--"), 0, 1, 1, String::from("hi"), "bye");
        let token_without_kind: Token<()> = token_with_kind.to_kindless();

        assert_eq!(token_without_kind.location, token_with_kind.location);
        assert_eq!(token_without_kind.text, token_with_kind.text);
    }
}
