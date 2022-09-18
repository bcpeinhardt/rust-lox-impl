use thiserror::Error;

/// An enum to represent all possible errors encountered while scanning/lexing
#[derive(Error, Debug)]
pub enum ScanError {
    #[error("{0}: Unexpected Character")]
    UnexpectedCharacter(ScanErrorCtx),

    #[error("{0}: Unterminated String")]
    UnterminatedString(ScanErrorCtx),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScanErrorCtx {
    pub line: usize,
}

impl From<usize> for ScanErrorCtx {
    fn from(line: usize) -> Self {
        Self { line }
    }
}

impl std::fmt::Display for ScanErrorCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[Line {}]", self.line)
    }
}
