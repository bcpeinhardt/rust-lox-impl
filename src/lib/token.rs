/// Represents every valid Lox token.
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

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TokenType::LeftParen => "(".to_owned(),
            TokenType::RightParen => ")".to_owned(),
            TokenType::LeftBrace => "{".to_owned(),
            TokenType::RightBrace => "}".to_owned(),
            TokenType::Comma => ".to_owned(),".to_owned(),
            TokenType::Dot => ".".to_owned(),
            TokenType::Minus => "-".to_owned(),
            TokenType::Plus => "+".to_owned(),
            TokenType::SemiColon => ";".to_owned(),
            TokenType::Slash => "/".to_owned(),
            TokenType::Star => "*".to_owned(),
            TokenType::Bang => "!".to_owned(),
            TokenType::BangEqual => "!=".to_owned(),
            TokenType::Equal => "=".to_owned(),
            TokenType::EqualEqual => "==".to_owned(),
            TokenType::GreaterEqual => ">=".to_owned(),
            TokenType::Greater => ">".to_owned(),
            TokenType::Less => "<".to_owned(),
            TokenType::LessEqual => "<=".to_owned(),
            TokenType::Identifier => "identifier".to_owned(),
            TokenType::String(s) => s.clone(),
            TokenType::Number(n) => {
                format!("{}", n)
            }
            TokenType::And => "&&".to_owned(),
            TokenType::Class => "class".to_owned(),
            TokenType::Else => "else".to_owned(),
            TokenType::False => "false".to_owned(),
            TokenType::Fun => "fun".to_owned(),
            TokenType::For => "for".to_owned(),
            TokenType::If => "if".to_owned(),
            TokenType::Nil => "nil".to_owned(),
            TokenType::Or => "||".to_owned(),
            TokenType::Print => "print".to_owned(),
            TokenType::Return => "return".to_owned(),
            TokenType::Super => "super".to_owned(),
            TokenType::This => "this".to_owned(),
            TokenType::True => "true".to_owned(),
            TokenType::Var => "var".to_owned(),
            TokenType::While => "while".to_owned(),
            TokenType::Eof => "eof".to_owned(),
        };

        write!(f, "{}", s)
    }
}

/// Represents a valid Lox token.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Represents the type of the token. Contains the associated literal for Strings and Numbers.
    pub token_type: TokenType,

    /// What the token looks like. The Lexemme for a Plus would be "+".to_string().
    pub lexeme: String,

    /// The line the particular token was found on.
    pub line: usize,
}

impl Token {
    /// Standard constructor
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
        write!(f, "{}", self.lexeme)
    }
}
