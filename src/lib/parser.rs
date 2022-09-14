use crate::{
    error::{
        error_reporter::ErrorReporter,
        parse_error::{ParseError, ParseErrorCtx},
    },
    grammar::{
        AssignmentExpr, BinaryExpr, BlockStmt, CallExpr, Expr, ExpressionStmt,
        FunctionDeclarationStmt, GroupingExpr, IfStmt, LiteralExpr, PrintStmt, ReturnStmt, Stmt,
        UnaryExpr, VariableDeclarationStmt, VariableExpr, WhileStmt,
    },
    token::{Token, TokenType},
};

pub type ParseResult<T> = Result<T, ParseError>;

/// The parser is responsible for taking a list of tokens and turning them into a syntax tree.
pub struct Parser {
    /// The list of tokens to parse into a syntax tree
    tokens: Vec<Token>,

    /// An index of where we are in the token list.
    current: usize,

    /// Enrichable object for tracking static errors through scanning and parsing
    error_reporter: ErrorReporter,
}

impl Parser {
    /// Takes a list of tokens to be parsed and a StaticErrorReporter object to be enriched
    /// with errors encountered in the parsing process.
    pub fn new(tokens: Vec<Token>, error_reporter: ErrorReporter) -> Self {
        Self {
            tokens,
            current: 0,
            error_reporter,
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
        let res = if self.advance_on(TokenType::Fun) {
            self.function()
        } else if self.advance_on(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match res {
            Ok(stmt) => Some(stmt),
            Err(e) => {
                self.error_reporter.error(e);
                self.synchronize();
                None
            }
        }
    }

    fn function(&mut self) -> ParseResult<Stmt> {
        let name = self.advance_on_or_err(TokenType::Identifier)?;
        let left_paren = self.advance_on_or_err(TokenType::LeftParen)?;
        let mut params = vec![];
        if !self.current_token_is_a(TokenType::RightParen) {
            params.push(self.advance_on_or_err(TokenType::Identifier)?);
            while self.advance_on(TokenType::Comma) {
                if params.len() >= 255 {
                    self.error_reporter
                        .error(ParseError::TooManyFunctionArguments(ParseErrorCtx {
                            token: left_paren.clone(),
                        }))
                }
                params.push(self.advance_on_or_err(TokenType::Identifier)?);
            }
        }
        self.advance_on_or_err(TokenType::RightParen)?;
        self.advance_on_or_err(TokenType::LeftBrace)?;
        let body = self.block_statement()?.body;
        Ok(Stmt::FunctionDeclaration(FunctionDeclarationStmt {
            name,
            params,
            body,
        }))
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.advance_on_or_err(TokenType::Identifier)?;
        let mut initializer = None;
        if self.advance_on(TokenType::Equal) {
            initializer = Some(self.expression()?);
        }
        self.advance_on_or_err(TokenType::SemiColon)?;
        Ok(Stmt::VariableDeclaration(VariableDeclarationStmt {
            name,
            initializer,
        }))
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.advance_on(TokenType::If) {
            self.if_statement()
        } else if self.advance_on(TokenType::For) { 
            self.for_statement()
        } else if self.advance_on(TokenType::Print) {
            self.print_statement()
        } else if self.advance_on(TokenType::While) {
            self.while_statement()
        } else if self.advance_on(TokenType::Return) {
            self.return_statement()
        } else if self.advance_on(TokenType::LeftBrace) {
            Ok(Stmt::Block(self.block_statement()?))
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.advance_on_or_err(TokenType::LeftParen)?;

        // Parse the initializer
        let initializer = if self.advance_on(TokenType::SemiColon) {
            None
        } else if self.advance_on(TokenType::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        // Parse the condition
        let mut condition = if !self.current_token_is_a(TokenType::SemiColon) { 
            Some(self.expression()?)
        } else {
            None
        };
        self.advance_on_or_err(TokenType::SemiColon)?;

        // Parse the increment
        let increment = if !self.current_token_is_a(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.advance_on_or_err(TokenType::RightParen)?;

        // Parse the body of the loop
        let mut body = self.statement()?;

        // Add the inc as a final statement to execute in the desugared while loop
        if let Some(inc) = increment { 
            body = Stmt::Block(BlockStmt {
                body: vec![body, Stmt::Expression(ExpressionStmt {
                    expr: inc
                })],
            });
        }

        // If the condition is null, set it to a simple literal true value.
        if condition.is_none() {
            condition = Some(Expr::Literal(LiteralExpr {
                token: Token::new(TokenType::True, "true".to_owned(), 0)
            }))
        }

        // Make the body a while loop which executes itself based on the condition
        body = Stmt::While(WhileStmt {
            condition: condition.unwrap(), body: Box::new(body)
        });

        if let Some(init) = initializer {
            body = Stmt::Block(BlockStmt {
                body: vec![init, body]
            });
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.advance_on_or_err(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.advance_on_or_err(TokenType::RightParen)?;
        let body = self.statement()?;
        Ok(Stmt::While(WhileStmt {
            condition, body: Box::new(body)
        }))
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let return_keyword = self.previous_token();
        let mut value = None;
        if !self.current_token_is_a(TokenType::SemiColon) {
            value = Some(self.expression()?);
        }
        self.advance_on_or_err(TokenType::SemiColon)?;
        Ok(Stmt::Return(ReturnStmt {
            return_keyword,
            value,
        }))
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.advance_on_or_err(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.advance_on_or_err(TokenType::RightParen)?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.advance_on(TokenType::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If(IfStmt {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn block_statement(&mut self) -> ParseResult<BlockStmt> {
        let mut statements = vec![];

        while !self.is_at_end() && !self.current_token_is_a(TokenType::RightBrace) {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }

        self.advance_on_or_err(TokenType::RightBrace)?;
        Ok(BlockStmt { body: statements })
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.advance_on_or_err(TokenType::SemiColon)?;
        Ok(Stmt::Print(PrintStmt { expr }))
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.advance_on_or_err(TokenType::SemiColon)?;
        Ok(Stmt::Expression(ExpressionStmt { expr }))
    }

    /// expression -> assignment
    fn expression(&mut self) -> ParseResult<Expr> {
        Ok(self.assignment()?)
    }

    /// assignment -> some_var = assignment
    ///             | equality
    fn assignment(&mut self) -> ParseResult<Expr> {
        // If we're looking as an assignment, this will trickle down to an Expr::Variable
        let expr = self.or()?;

        if self.advance_on(TokenType::Equal) {
            let equals = self.previous_token();
            let value = self.assignment()?;

            if let Expr::Variable(VariableExpr { name }) = expr {
                return Ok(Expr::Assignment(AssignmentExpr {
                    variable: name,
                    expr: Box::new(value),
                }));
            }

            self.error_reporter
                .error(ParseError::InvalidAssignmentTarget(ParseErrorCtx {
                    token: equals,
                }))
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;

        while self.advance_on(TokenType::Or) {
            let operator = self.previous_token();
            let right = self.and()?;
            expr = Expr::Logical(BinaryExpr {
                lhs: Box::new(expr),
                operator,
                rhs: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;

        while self.advance_on(TokenType::And) {
            let operator = self.previous_token();
            let right = self.equality()?;
            expr = Expr::Logical(BinaryExpr {
                lhs: Box::new(expr),
                operator,
                rhs: Box::new(right),
            });
        }

        Ok(expr)
    }

    /// equality -> comparison (( != | ==) comparison )*
    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;
        while self.advance_on_any_of(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous_token();
            let right = self.comparison()?;
            expr = Expr::Binary(BinaryExpr {
                lhs: Box::new(expr),
                operator,
                rhs: Box::new(right),
            });
        }
        Ok(expr)
    }

    /// comparison -> term (( > | >= | < | <= ) term)*
    fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;
        while self.advance_on_any_of(vec![
            TokenType::GreaterEqual,
            TokenType::Greater,
            TokenType::LessEqual,
            TokenType::Less,
        ]) {
            let operator = self.previous_token();
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {
                lhs: Box::new(expr),
                operator,
                rhs: Box::new(right),
            });
        }
        Ok(expr)
    }

    /// term -> factor ((+ | - ) factor)*
    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;
        while self.advance_on_any_of(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous_token();
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr {
                lhs: Box::new(expr),
                operator,
                rhs: Box::new(right),
            });
        }
        Ok(expr)
    }

    /// factor -> unary (( / | * ) unary)*
    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;
        while self.advance_on_any_of(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous_token();
            let right = self.unary()?;
            expr = Expr::Binary(BinaryExpr {
                lhs: Box::new(expr),
                operator,
                rhs: Box::new(right),
            });
        }
        Ok(expr)
    }

    /// unary -> ( ! | - ) unary
    ///        | primary ;
    fn unary(&mut self) -> ParseResult<Expr> {
        if self.advance_on_any_of(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous_token();
            let right = self.unary()?;
            Ok(Expr::Unary(UnaryExpr {
                operator,
                rhs: Box::new(right),
            }))
        } else {
            Ok(self.call()?)
        }
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.advance_on(TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut args = vec![];
        if !self.current_token_is_a(TokenType::RightParen) {
            args.push(self.expression()?);
            while self.advance_on(TokenType::Comma) {
                if args.len() >= 255 {
                    // We report an error but we dont throw it because we dont need to synchronize.
                    self.error_reporter
                        .error(ParseError::TooManyFunctionArguments(ParseErrorCtx {
                            token: self.current_token(),
                        }));
                }
                args.push(self.expression()?);
            }
        }
        let closing_paren = self.advance_on_or_err(TokenType::RightParen)?;
        Ok(Expr::Call(CallExpr {
            callee: Box::new(callee),
            closing_paren,
            args,
        }))
    }

    /// primary -> NUMBER | STRING | true | false | nil
    ///          | ( expression )
    fn primary(&mut self) -> ParseResult<Expr> {
        if self.advance_on(TokenType::Identifier) {
            Ok(Expr::Variable(VariableExpr {
                name: self.previous_token(),
            }))
        } else if self.advance_on(TokenType::LeftParen) {
            // Handle a grouping
            let expr = self.expression()?;
            self.advance_on_or_err(TokenType::RightParen)?;
            Ok(Expr::Grouping(GroupingExpr {
                expr: Box::new(expr),
            }))
        } else if self.advance_on_any_of(vec![TokenType::True, TokenType::False, TokenType::Nil]) {
            // Handle bool or nil
            Ok(Expr::Literal(LiteralExpr {
                token: self.previous_token(),
            }))
        } else {
            // Handle String or Number
            if let TokenType::String(_) = self.current_token().token_type {
                Ok(Expr::Literal(LiteralExpr {
                    token: self.advance(),
                }))
            } else if let TokenType::Number(_) = self.current_token().token_type {
                Ok(Expr::Literal(LiteralExpr {
                    token: self.advance(),
                }))
            } else {
                // We've reached the bottom of the grammar and we don't know what expression this is.
                Err(ParseError::ExpectedExpression(ParseErrorCtx {
                    token: self.current_token(),
                }))
            }
        }
    }

    /// Will advance the current token if it has the given token type, otherwise
    /// it will produce an error with the given message.
    fn advance_on_or_err(&mut self, tt: TokenType) -> ParseResult<Token> {
        if self.current_token_is_a(tt.clone()) {
            Ok(self.advance())
        } else {
            Err(ParseError::ExpectedDifferentToken(
                ParseErrorCtx {
                    token: self.current_token(),
                },
                tt,
            ))
        }
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
