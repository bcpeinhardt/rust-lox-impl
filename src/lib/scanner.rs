use crate::{
    error::{
        error_reporter::ErrorReporter,
        scan_error::{ScanError, ScanErrorCtx},
    },
    token::{Token, TokenType},
    util::{is_alpha, is_alpha_numeric, is_digit, keywords, strip_quotes},
};

/// The scanner class is used to take raw source code as a string and produce a Vector of tokens, as well
/// as to report any errors encountered in the process.
pub struct Scanner {
    /// The original source code as a String
    source: String,

    /// Used to collect the tokens as the source code is lexed
    tokens: Vec<Token>,

    /// Used to keep track of our spot in the source code
    start: usize,

    /// Used to keep track of our spot in the source code
    current: usize,

    /// The line is really only kept for error handling. Incremented every time a \n
    /// is found in source.
    line: usize,

    /// Enrichable object for tracking static errors through scanning and parsing
    error_reporter: ErrorReporter,
}

impl Scanner {
    /// Generates a new scanner from the source code and a reference to the Lox class (for reporting errors that outlive the Scanner)
    pub fn new(source: String, error_reporter: ErrorReporter) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            error_reporter,
        }
    }

    fn err_ctx(&self) -> ScanErrorCtx {
        self.line.into()
    }

    /// Scans the source code and produces a Vector of Tokens.
    pub fn scan_tokens(mut self) -> (Vec<Token>, ErrorReporter) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        // Add an automatic EOF token when the end of the source code is reached.
        self.tokens
            .push(Token::new(TokenType::Eof, "".to_owned(), self.line));

        (self.tokens, self.error_reporter)
    }

    /// Handles scanning in any single token.
    fn scan_token(&mut self) {
        let next_char = self.advance();

        match next_char {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::SemiColon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                if self.advance_on('=') {
                    self.add_token(TokenType::BangEqual);
                } else {
                    self.add_token(TokenType::Bang);
                }
            }
            '=' => {
                if self.advance_on('=') {
                    self.add_token(TokenType::EqualEqual);
                } else {
                    self.add_token(TokenType::Equal);
                }
            }
            '<' => {
                if self.advance_on('=') {
                    self.add_token(TokenType::LessEqual);
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            '>' => {
                if self.advance_on('=') {
                    self.add_token(TokenType::GreaterEqual);
                } else {
                    self.add_token(TokenType::Greater);
                }
            }
            '/' => {
                if self.advance_on('/') {
                    // Read to the end of a comment line
                    while self.current_char() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {
                // Ignore whitespace
            }
            '\n' => {
                self.line += 1;
            }
            '"' => {
                self.string();
            }
            n if is_digit(n) => {
                self.number();
            }
            l if is_alpha(l) => {
                self.identifier_or_keyword();
            }
            _ => {
                self.error_reporter
                    .error(ScanError::UnexpectedCharacter(self.err_ctx()));
            }
        }
    }

    /// Handles scanning in string values
    fn string(&mut self) {
        // Scan to the ending quotation mark
        while self.current_char() != '"' && !self.is_at_end() {
            // Support multi line strings
            if self.current_char() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        // If we reach the ending quotation before the end of the file, consume it then add the String token. Otherwise, report the error.
        if self.is_at_end() {
            self.error_reporter
                .error(ScanError::UnterminatedString(self.err_ctx()));
        } else {
            self.advance(); // Closing "
            self.add_token(TokenType::String(strip_quotes(self.get_current_lexeme())));
        }
    }

    /// Handles scanning number values
    fn number(&mut self) {
        // Scan in all digits
        while is_digit(self.current_char()) {
            self.advance();
        }

        // If there is a DOT character followed by more digits, scan in the DOT and the rest of the digits
        if self.current_char() == '.' && is_digit(self.next_char()) {
            self.advance();
            while is_digit(self.current_char()) {
                self.advance();
            }
        }

        // Parse the number as an f64 and add the token for the number literal.
        let num = self
            .get_current_lexeme()
            .parse::<f64>()
            .expect("Could not parse f64 from number");
        self.add_token(TokenType::Number(num));
    }

    /// Handles scanning in Keywords and Identifiers.
    fn identifier_or_keyword(&mut self) {
        // Encompass the rest of the alpha_numeric characters of the identifier.
        while is_alpha_numeric(self.current_char()) {
            self.advance();
        }

        // Check to see if the current lexeme is one of Lox's keywords. If it is,
        // add the appropriate keyword token, otherwise add it as an identifier.
        match keywords().get(&self.get_current_lexeme()) {
            Some(keyword_token) => {
                self.add_token(keyword_token.clone());
            }
            None => {
                self.add_token(TokenType::Identifier);
            }
        }
    }

    /// Lets us know if we've made it to the end of the source code.
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// Advance current to encompass another character and return the previous character for evaluation.
    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).expect(&format!(
            "Could not find {}th char in source",
            self.current - 1
        ))
    }

    /// Only advances current if the next char is the one we're looking for. Returns
    /// boolean indicating whether the character was found.
    fn advance_on(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.current_char() != expected {
            return false;
        } else {
            self.advance();
            return true;
        }
    }

    /// Returns the current character in source.
    fn current_char(&self) -> char {
        self.peek_n_characters(0)
    }

    /// Returns the next character in source.
    fn next_char(&self) -> char {
        self.peek_n_characters(1)
    }

    /// Method for peeking at what the next characters are.
    fn peek_n_characters(&self, n: usize) -> char {
        if self.current + n >= self.source.len() {
            '\0'
        } else {
            self.source
                .chars()
                .nth(self.current + n)
                .expect(&format!("No char {} characters ahead", n))
        }
    }

    /// Returns the current subslice of the source code in view by the Scanner
    fn get_current_lexeme(&self) -> String {
        self.source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect()
    }

    /// Adds any token to the tokens list
    fn add_token(&mut self, token_type: TokenType) {
        self.tokens
            .push(Token::new(token_type, self.get_current_lexeme(), self.line));
    }
}
