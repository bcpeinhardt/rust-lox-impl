use crate::{
    lox::Lox,
    scanner::{Token, TokenType}, expr::Expr, stmt::Stmt, error::{ErrorReporter, StaticResult, StaticError},
};

/// The parser is responsible for taking a list of tokens and turning them into an abstract syntax tree.
pub struct Parser {

    /// The list of tokens to parse into a syntax tree
    tokens: Vec<Token>,

    /// An index of where we are in the token list.
    current: usize,

    error_reporter: ErrorReporter
}

impl Parser {
    pub fn new(tokens: Vec<Token>, error_reporter: ErrorReporter) -> Self {
        Self {
            tokens,
            current: 0,
            error_reporter
        }
    }

    pub fn parse(mut self) -> (Vec<Stmt>, ErrorReporter) {
        let mut statements = vec![];

        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt)
            }
        }
        
        (statements, self.error_reporter)
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let res = if self.advance_on(TokenType::Var) { 
            self.var_declaration()
        } else {
            self.statement()
        };

        match res {
            Ok(stmt) => Some(stmt),
            Err(e) => {
                self.synchronize();
                None
            },
        }
    }

    fn var_declaration(&mut self) -> StaticResult<Stmt> {
        let name = self.advance_on_or_err(TokenType::Identifier, "Expected variable name.")?;
        let mut initializer = None;
        if self.advance_on(TokenType::Equal) { 
            initializer = Some(self.expression()?);
        }
        self.advance_on_or_err(TokenType::SemiColon, "Expected ';' after variable declaration.")?;
        Ok(Stmt::VarDecl(name, initializer))
    }

    fn statement(&mut self) -> StaticResult<Stmt> { 
        if self.advance_on(TokenType::Print) { 
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> StaticResult<Stmt> {
        let expr = self.expression()?;
        self.advance_on_or_err(TokenType::SemiColon, "Expected ';' after value")?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> StaticResult<Stmt> { 
        let expr = self.expression()?;
        self.advance_on_or_err(TokenType::SemiColon, "Expected ';' after expression")?;
        Ok(Stmt::Expression(expr))
    }

    /// expression -> equality
    fn expression(&mut self) -> StaticResult<Expr> {
        Ok(self.equality()?)
    }

    /// equality -> comparison (( != | ==) comparison )*
    fn equality(&mut self) -> StaticResult<Expr> {
        let mut expr = self.comparison()?;
        while self.advance_on_any_of(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous_token();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    /// comparison -> term (( > | >= | < | <= ) term)*
    fn comparison(&mut self) -> StaticResult<Expr> {
        let mut expr = self.term()?;
        while self.advance_on_any_of(vec![
            TokenType::GreaterEqual,
            TokenType::Greater,
            TokenType::LessEqual,
            TokenType::Less,
        ]) {
            let operator = self.previous_token();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    /// term -> factor ((+ | - ) factor)*
    fn term(&mut self) -> StaticResult<Expr> {
        let mut expr = self.factor()?;
        while self.advance_on_any_of(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous_token();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    /// factor -> unary (( / | * ) unary)*
    fn factor(&mut self) -> StaticResult<Expr> {
        let mut expr = self.unary()?;
        while self.advance_on_any_of(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous_token();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    /// unary -> ( ! | - ) unary
    ///        | primary ;
    fn unary(&mut self) -> StaticResult<Expr> {
        if self.advance_on_any_of(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous_token();
            let right = self.unary()?;
            Ok(Expr::Unary(operator, Box::new(right)))
        } else {
            Ok(self.primary()?)
        }
    }

    /// primary -> NUMBER | STRING | true | false | nil
    ///          | ( expression )
    fn primary(&mut self) -> StaticResult<Expr> {
        if self.advance_on(TokenType::Identifier) {
            Ok(Expr::Variable(self.previous_token()))
        } else if self.advance_on(TokenType::LeftParen) {
            // Handle a grouping
            let expr = self.expression()?;
            self.advance_on_or_err(TokenType::RightParen, "Expected ')' after expression.")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else if self.advance_on_any_of(vec![TokenType::True, TokenType::False, TokenType::Nil]) {
            // Handle bool or nil
            Ok(Expr::Literal(self.previous_token()))
        } else {
            // Handle String or Number
            if let TokenType::String(_) = self.current_token().token_type {
                Ok(Expr::Literal(self.advance()))
            } else if let TokenType::Number(_) = self.current_token().token_type {
                Ok(Expr::Literal(self.advance()))
            } else {
                // We've reached the bottom of the grammar and we don't know what expression this is.
                self.error(self.current_token(), "Expected expression");
                Err(())
            }
        }
    }

    /// Will advance the current token if it has the given token type, otherwise
    /// it will produce an error with the given message.
    fn advance_on_or_err(&mut self, tt: TokenType, msg: &str) -> StaticResult<Token> {
        if self.current_token_is_a(tt) {
            Ok(self.advance())
        } else {
            self.error(self.current_token(), msg);
            Err(())
        }
    }

    /// Reports the error to the calling lox instance, then returns the relevant ParseError
    /// for the Parser.
    fn error(&mut self, token: Token, msg: &str) -> StaticError {
        self.error_reporter.error_token(token, msg);
        ()
    }

    /// Tries to bring the parser to a statement boundary when an error is encountered.
    fn synchronize(&mut self) {
        self.advance();

        // Advance until the previoous token is a Semi-colon or
        // the current colon is a keywordused to start a statement.
        while !self.is_at_end() {
            if self.previous_token().token_type == TokenType::SemiColon
                || vec![
                    TokenType::Class,
                    TokenType::For,
                    TokenType::Fun,
                    TokenType::If,
                    TokenType::Print,
                    TokenType::Return,
                    TokenType::Var,
                    TokenType::While,
                ]
                .iter()
                .any(|tt| tt.clone() == self.current_token().token_type)
            {
                return;
            }
            self.advance();
        }
    }

    /// Will advance the current token if it has one of the given token types.
    /// Returns whether or not the current token was advanced.
    fn advance_on_any_of(&mut self, token_types: Vec<TokenType>) -> bool {
        for tt in token_types.into_iter() {
            if self.current_token_is_a(tt) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    /// Will advance the current token if it has the given token type.
    /// Returns whether or not the current token was advanced.
    fn advance_on(&mut self, token_type: TokenType) -> bool {
        self.advance_on_any_of(vec![token_type])
    }

    /// Checks whether the current token is a particular token type
    fn current_token_is_a(&mut self, tt: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.current_token().token_type == tt
        }
    }

    /// If not yet at the end of the tokens, advances the current token and returns
    /// the token we were originally on.
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous_token()
    }

    /// Returns whether the current token is an EOF token
    fn is_at_end(&self) -> bool {
        self.current_token().token_type == TokenType::Eof
    }

    /// Returns the current token in the list
    fn current_token(&self) -> Token {
        self.tokens
            .get(self.current)
            .expect("Called unwrap from Parser::peek fn on missing token")
            .clone()
    }

    /// Returns the previous token in the list
    fn previous_token(&mut self) -> Token {
        self.tokens
            .get(self.current - 1)
            .expect("Called unwrap from Parser::peek fn on missing token")
            .clone()
    }
}
