use thiserror::Error;

use crate::token::Token;

/// An enum to represent all possible errors encountered while scanning/lexing
#[derive(Error, Debug, Clone)]
pub enum RuntimeError {
    /// For the moment, I don't want to try to enumerate all the errors so I'll just pass a message.
    #[error("{0}: {1}")]
    WithMsg(RuntimeErrorCtx, String),
}

impl RuntimeError {
    pub fn new(token: Token, msg: impl std::fmt::Display) -> Self {
        Self::WithMsg(RuntimeErrorCtx { token }, msg.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeErrorCtx {
    pub token: Token,
}

impl std::fmt::Display for RuntimeErrorCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[Line {}] Error at '{}'",
            self.token.line, self.token.lexeme
        )
    }
}
