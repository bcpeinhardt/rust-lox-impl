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
