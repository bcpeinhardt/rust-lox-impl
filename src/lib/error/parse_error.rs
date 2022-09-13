use thiserror::Error;

use crate::token::{Token, TokenType};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}: Invalid Assignment Target")]
    InvalidAssignmentTarget(ParseErrorCtx),

    #[error("{0}: Expected Expression")]
    ExpectedExpression(ParseErrorCtx),

    #[error("{0}: Expected '{1}'")]
    ExpectedDifferentToken(ParseErrorCtx, TokenType),

    #[error("{0}: Cannot have more that 255 arguments for a function (Seriously chill)")]
    TooManyFunctionArguments(ParseErrorCtx),
}

#[derive(Debug)]
pub struct ParseErrorCtx {
    pub token: Token,
}

impl std::fmt::Display for ParseErrorCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[Line {}] Error at '{}'",
            self.token.line, self.token.lexeme
        )
    }
}
