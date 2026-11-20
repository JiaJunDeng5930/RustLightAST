//! Declaration metadata derived from the RustLight model.

use std::fmt;

#[derive(Debug, Clone)]
pub struct SourceParseError(String);

impl SourceParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for SourceParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SourceParseError {}

impl From<syn::Error> for SourceParseError {
    fn from(error: syn::Error) -> Self {
        let start = error.span().start();
        Self(format!("{error} (line {}, column {})", start.line, start.column + 1))
    }
}
