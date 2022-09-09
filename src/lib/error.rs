use crate::{scanner::{Token, TokenType}};

pub type StaticError = ();
pub type StaticResult<T> = Result<T, StaticError>;

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
            msg: msg.to_owned()
        }
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// The error reporter is essentially an enrichable object which is passed through the scanner, parser, and interpreter 
/// for collecting errors that happen during these processes and responding to them at a time of our choosing.
/// 
#[derive(Debug, Clone)]
pub struct ErrorReporter {
    pub had_static_error: bool,
    pub had_runtime_error: bool,
}

impl ErrorReporter { 
    pub fn new() -> Self {
        Self {
            had_static_error: false, 
            had_runtime_error: false
        }
    }

    /// Report a static error given a line and a msg (Used from Scanner)
    pub fn error(&mut self, line: usize, msg: String) {
        self.static_error(line, "".to_string(), msg);
    }

    /// Report a static error given a token and a Msg (called from Parser)
    pub fn error_token(&mut self, token: Token, msg: &str) {
        if token.token_type == TokenType::Eof {
            self.static_error(token.line, " at end".to_owned(), msg.to_owned());
        } else {
            self.static_error(
                token.line,
                format!(" at '{}'", token.lexeme),
                msg.to_owned(),
            );
        }
    }

    /// Internal method for reporting a static error
    fn static_error(&mut self, line: usize, location: String, msg: String) {
        eprintln!("[line {}] Error{}: {}", line, location, msg);
        self.had_static_error = true;
    }

    /// Report a runtime error (Called from the Interpreter)
    pub fn runtime_error(&mut self, error: RuntimeError) {
        eprintln!("{} [line {}]", error.msg, error.token.line);
        self.had_runtime_error = true;
    }
}