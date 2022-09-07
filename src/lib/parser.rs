use crate::{scanner::{Token, TokenType}, lox::Lox};

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Token),
    Unary(Token, Box<Expr>)
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Binary(lhs, token, rhs) => {
                write!(f, "({} {} {})", token.lexeme, lhs, rhs)
            },
            Expr::Grouping(expr) => {
                write!(f, "(group {})", expr)
            },
            Expr::Literal(token) => {
                write!(f, "{}", token.lexeme)
            },
            Expr::Unary(token, expr) => {
                write!(f, "({} {})", token.lexeme, expr)
            },
        }
    }
}

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    lox: &'a mut Lox
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, lox: &'a mut Lox) -> Self {
        Self {
            tokens,
            current: 0,
            lox
        }
    }

    pub fn parse(&mut self) -> Expr {
        self.expression()
    }

    /// expression -> equality
    fn expression(&mut self) -> Expr {
        self.equality()
    }

    /// equality -> comparison (( != | ==) comparison )*
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.match_tokens(vec![TokenType::BangEqual, TokenType::EqualEqual]) { 
            let operator = self.previous();
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        expr
    }

    /// comparison -> term (( > | >= | < | <= ) term)*
    fn comparison(&mut self) -> Expr { 
        let mut expr = self.term();
        while self.match_tokens(vec![TokenType::GreaterEqual, TokenType::Greater, TokenType::LessEqual, TokenType::Less]) {
            let operator = self.previous();
            let right = self.term();
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        expr
    }

    /// term -> factor ((+ | - ) factor)*
    fn term(&mut self) -> Expr { 
        let mut expr = self.factor();
        while self.match_tokens(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.factor();
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        expr
    }

    /// factor -> unary (( / | * ) unary)*
    fn factor(&mut self) -> Expr { 
        let mut expr = self.unary();
        while self.match_tokens(vec![TokenType::Slash, TokenType::Star]) { 
            let operator = self.previous();
            let right = self.unary();
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        expr
    }

    /// unary -> ( ! | - ) unary
    ///        | primary ;
    fn unary(&mut self) -> Expr { 
        if self.match_tokens(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary();
            Expr::Unary(operator, Box::new(right))
        } else {
            self.primary()
        }
    }

    /// primary -> NUMBER | STRING | true | false | nil
    ///          | ( expression )
    fn primary(&mut self) -> Expr { 
        if self.match_token(TokenType::LeftParen) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expected ')' after expression.");
            Expr::Grouping(Box::new(expr))
        } else if self.match_tokens(vec![TokenType::True, TokenType::False, TokenType::Nil]) {

            // This is pretty different from the book. We are actually storing the tokens. If this 
            // gets too weird, we can do some refactoring in Scanner and here in the parser to find 
            // a suitable replacement for Java's Object type.
            Expr::Literal(self.previous())
        } else {
            // Handle destructuring string and Number literals
            if let TokenType::String(ref s) = self.peek().token_type {
                Expr::Literal(self.advance())
            } else if let TokenType::Number(ref s) = self.peek().token_type { 
                Expr::Literal(self.advance())
            } else {
                self.error(self.peek(), "Expected expression");
                Expr::Literal(self.peek())
            }
        }
    }

    fn consume(&mut self, tt: TokenType, msg: &str) -> Option<Token> {
        if self.check(tt) {
            Some(self.advance())
        } else {
            self.error(self.peek(), msg);
            None
        }
    }

    fn error(&mut self, token: Token, msg: &str) -> String {
        self.lox.error_token(token, msg);
        "Error".to_owned()
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SemiColon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class | TokenType::For | TokenType::Fun | TokenType::If | TokenType::Print | TokenType::Return | TokenType::Var | TokenType::While => {
                    
                },
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn match_tokens(&mut self, token_types: Vec<TokenType>) -> bool {
        for tt in token_types.into_iter() {
            if self.check(tt) { 
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        self.match_tokens(vec![token_type])
    }

    fn check(&mut self, tt: TokenType) -> bool { 
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == tt
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool { 
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> Token { 
        self.tokens.get(self.current).expect("Called unwrap from Parser::peek fn on missing token").clone()
    }

    fn previous(&mut self) -> Token { 
        self.tokens.get(self.current - 1).expect("Called unwrap from Parser::peek fn on missing token").clone()
    }
}