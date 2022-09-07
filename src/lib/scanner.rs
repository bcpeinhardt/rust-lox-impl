
use std::collections::HashMap;

use crate::{lox::Lox, util::strip_quotes};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    GreaterEqual,
    Greater,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String(String),
    Number(f64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

fn keywords() -> HashMap<String, TokenType> { 
    let mut map = HashMap::new();
    map.insert("and".to_owned(), TokenType::And);
    map.insert("class".to_owned(), TokenType::Class);
    map.insert("else".to_owned(), TokenType::Else);
    map.insert("false".to_owned(), TokenType::False);
    map.insert("for".to_owned(), TokenType::For);
    map.insert("fun".to_owned(), TokenType::Fun);
    map.insert("if".to_owned(), TokenType::If);
    map.insert("nil".to_owned(), TokenType::Nil);
    map.insert("or".to_owned(), TokenType::Or);
    map.insert("print".to_owned(), TokenType::Print);
    map.insert("return".to_owned(), TokenType::Return);
    map.insert("super".to_owned(), TokenType::Super);
    map.insert("this".to_owned(), TokenType::This);
    map.insert("true".to_owned(), TokenType::True);
    map.insert("var".to_owned(), TokenType::Var);
    map.insert("while".to_owned(), TokenType::While);

    map
}


#[derive(Debug, Clone, PartialEq)]
pub struct Token {

    // This is our first real departure from the book. Rust doesn't have an "Object" type. I think the simplest solution is to refactor
    // the Token class to not have a field called "literal". Instead, the TokenType variants String and Number can simply have their
    // underlying types associated with them. 
    token_type: TokenType,
    lexeme: String,
    line: usize
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Token { 
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} {}", self.token_type, self.lexeme)
    }
} 

pub struct Scanner<'a> {

    // The scanner gets a mutable reference to the Lox class to allow for calling the
    // error handling methods.
    lox: &'a mut Lox,

    // The original source code as a string
    source: String,

    // Used to collect the tokens as the source code is lexed
    tokens: Vec<Token>,

    // Used to keep track of our spot in the source code
    start: usize,
    current: usize,

    // The line is really only kept for error handling. Incremented every time a \n
    // is found in source.
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: String, lox: &'a mut Lox) -> Self {
        Self {

            lox,

            source,
            tokens: vec![],

            start: 1,
            current: 0,
            line: 1
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(TokenType::Eof, "".to_owned(), self.line));
        self.tokens.clone()
    }

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
                if self.match_next('=') {
                    self.add_token(TokenType::BangEqual);
                } else {
                    self.add_token(TokenType::Bang);
                }
            },
            '=' => {
                if self.match_next('=') {
                    self.add_token(TokenType::EqualEqual);
                } else {
                    self.add_token(TokenType::Equal);
                }
            },
            '<' => {
                if self.match_next('=') {
                    self.add_token(TokenType::LessEqual);
                } else {
                    self.add_token(TokenType::Less);
                }
            },
            '>' => {
                if self.match_next('=') {
                    self.add_token(TokenType::GreaterEqual);
                } else {
                    self.add_token(TokenType::Greater);
                }
            },
            '/' => { 
                if self.match_next('/') {
                    // Read to the end of a comment line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            },
            ' ' | '\r' | '\t' => {
            },
            '\n' => { self.line += 1; },
            '"' => {
                self.string();
            },
            n if Self::is_digit(n) => {
                self.number();
            },
            l if Self::is_alpha(l) => {
                self.identifier();
            }
            _ => {
                self.lox.error(self.line, "Unexpected Character.".to_owned());
            }

        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.lox.error(self.line, "Unterminated String".to_owned());
        } else {
            self.advance(); // Closing "
            let value = strip_quotes(self.get_current_lexeme());
            self.add_token(TokenType::String(value));
        }
    }

    fn number(&mut self) {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            self.advance();
            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }
        let num = self.get_current_lexeme().parse::<f64>().expect("Could not parse f64 from number");
        self.add_token(TokenType::Number(num));
    }

    fn identifier(&mut self) {
        while Self::is_alpha_numeric(self.peek()) { 
            self.advance();
        }

        let text: String = self.get_current_lexeme();
        match keywords().get(&text) {
            Some(keyword_token) => {
                self.add_token(keyword_token.clone());
            },
            None => {
                self.add_token(TokenType::Identifier);
            },
        }
    }

    fn is_digit(c: char) -> bool { 
        c >= '0' && c <= '9'
    }

    fn is_alpha(l: char) -> bool { 
        (l >= 'a' && l <= 'z') || (l >= 'A' && l <= 'Z') || l == '_'
    }

    fn is_alpha_numeric(c: char) -> bool {
        Self::is_alpha(c) || Self::is_digit(c)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).expect(&format!("Could not find {}th char in source", self.current - 1))
    }

    fn match_next(&mut self, expected: char) -> bool { 
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current).expect("No char at expected index") != expected { return false };

        self.current += 1;
        return true;
    }

    fn peek(&self) -> char { 
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).expect("No char at expected index")
        }
    }

    fn peek_next(&mut self) -> char { 
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).expect("No char at end")
        }
    }

    /// Returns the current subslice of the source code in view by the Scanner
    fn get_current_lexeme(&self) -> String {
        self.source.chars().skip(self.start).take(self.current - self.start).collect()
    }

    /// Adds any token to the tokens list
    fn add_token(&mut self, token_type: TokenType) {
        self.tokens.push(Token::new(token_type, self.get_current_lexeme(), self.line));
    }
}
