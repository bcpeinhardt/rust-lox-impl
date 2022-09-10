use thiserror::Error;

use crate::token::{Token, TokenType};

/// The error reporter is a very simple interface for reporting errors
/// vie stdout and recording whether or not an error occurred.
pub struct ErrorReporter {
    pub had_error: bool,
}

impl ErrorReporter {
    /// Basic constructor. Creates a new error reporter with had_error set to false.
    pub fn new() -> Self {
        Self { had_error: false }
    }

    /// Report any error that implements std::fmt::Display. The error
    /// will be print to the console and had_error will be set to true.
    pub fn error<T: std::fmt::Display>(&mut self, error: T) {
        eprintln!("{}", error);
        self.had_error = true;
    }
}

/// Enrichable object for tracking static errors through scanning and parsing
#[derive(Debug, Clone)]
pub struct StaticErrorReporter {
    pub had_error: bool,
}

impl StaticErrorReporter {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    /// Report a static error given a line and a msg (Used from Scanner)
    pub fn error(&mut self, line: usize, msg: String) {
        self._error(line, "".to_string(), msg);
    }

    /// Report a static error given a token and a Msg (called from Parser)
    pub fn error_token(&mut self, token: Token, msg: &str) {
        if token.token_type == TokenType::Eof {
            self._error(token.line, " at end".to_owned(), msg.to_owned());
        } else {
            self._error(
                token.line,
                format!(" at '{}'", token.lexeme),
                msg.to_owned(),
            );
        }
    }

    /// Internal method for reporting a static error
    fn _error(&mut self, line: usize, location: String, msg: String) {
        eprintln!("[line {}] Error{}: {}", line, location, msg);
        self.had_error = true;
    }
}

/// Create this error to represent an error which occurs at Runtime.
pub struct RuntimeError {
    pub token: Token,
    pub msg: String,
}

impl RuntimeError {
    /// Create this error to represent an error which occurs at Runtime.
    pub fn new(token: Token, msg: &str) -> Self {
        Self {
            token,
            msg: msg.to_owned(),
        }
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone)]
pub struct RuntimeErrorReporter {
    pub had_error: bool,
}

impl RuntimeErrorReporter {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    /// Report a runtime error (Called from the Interpreter)
    pub fn runtime_error(&mut self, error: RuntimeError) {
        eprintln!("{} [line {}]", error.msg, error.token.line);
        self.had_error = true;
    }
}
