use crate::{
    bubble_closure::ThenTry,
    error::{
        error_reporter::ErrorReporter,
        parse_error::{ParseError, ParseErrorCtx},
    },
    grammar::{
        AssignmentExpr, BinaryExpr, BlockStmt, CallExpr, Expr, ExpressionStmt,
        FunctionDeclarationStmt, GroupingExpr, IfStmt, LiteralExpr, ReturnStmt, Stmt, UnaryExpr,
        VariableDeclarationStmt, VariableExpr, WhileStmt,
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

    /// Used to keep track of how many local scopes deep we are (for variable resolving)
    depth: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>, error_reporter: ErrorReporter) -> Self {
        Self {
            tokens,
            current: 0,
            error_reporter,
            depth: 0
        }
    }

    /// Parses the provided list of Tokens into Lox Statements.
    /// Uses go style tuple error return so that multiple
    /// errors can be collected.
    pub fn parse(mut self) -> (Vec<Stmt>, ErrorReporter) {
        let mut statements = vec![];

        while !self.is_at_end() {
            if let Some(stmt) = self.parse_item() {
                statements.push(stmt)
            }
        }

        (statements, self.error_reporter)
    }

    /// Tries to parse a single statement, and returns the statement if succsessful
    /// or reports an error and syncronizes the parser.
    fn parse_item(&mut self) -> Option<Stmt> {
        self.declaration()
            .map_err(|e| {
                self.error_reporter.error(e);
                self.synchronize();
            })
            .ok()
    }

    /// A declaration is the top level parsable entity. Tries to parse
    /// a function declaration or a variable declaration, or defaults
    /// to some other kind of statement.
    fn declaration(&mut self) -> ParseResult<Stmt> {
        if self.advance_on(TokenType::Fun) {
            self.function_declaration()
                .map(|stmt| Stmt::FunctionDeclaration(stmt))
        } else if self.advance_on(TokenType::Var) {
            self.var_declaration()
                .map(|stmt| Stmt::VariableDeclaration(stmt))
        } else {
            self.statement()
        }
    }

    /// Parses a function declaration statement. Triggered when a `fun` token is
    /// encountered.
    fn function_declaration(&mut self) -> ParseResult<FunctionDeclarationStmt> {


        // Parse the function name and the opening parenthesis.
        let name = self.advance_on_or_err(TokenType::Identifier)?;
        let left_paren = self.advance_on_or_err(TokenType::LeftParen)?;

        // Parse the function parameters if any
        let mut params = vec![];
        if !self.current_token_is_a(TokenType::RightParen) {
            // Parse the first parameter.
            params.push(self.advance_on_or_err(TokenType::Identifier)?);

            // Parse the comma and the next parameter if there is one.
            while self.advance_on(TokenType::Comma) {
                params.push(self.advance_on_or_err(TokenType::Identifier)?);

                // We set a rule that functions can have no more than 255 parameters.
                if params.len() >= 255 {
                    self.error_reporter
                        .error(ParseError::TooManyFunctionArguments(
                            left_paren.clone().into(),
                        ))
                }
            }
        }

        // Consume the closing parenthesis and the body of the function as a block statement
        self.advance_on_or_err(TokenType::RightParen)?;
        self.advance_on_or_err(TokenType::LeftBrace)?;

        
        let body = self.block_statement()?.body;

        // Return the function declaration.
        Ok(FunctionDeclarationStmt { name, params, body })
    }

    /// Parses a variable declaration. Triggered when a `var` keyword is encountered.
    fn var_declaration(&mut self) -> ParseResult<VariableDeclarationStmt> {
        // Parse the variable name
        let name = self.advance_on_or_err(TokenType::Identifier)?;

        // If an `=` token is present, parse the expression after it as the initial value
        // for the variable.
        let initializer = self
            .advance_on(TokenType::Equal)
            .then_try(|| self.expression())?;

        // Consume the semi-colon to end the statement
        self.advance_on_or_err(TokenType::SemiColon)?;

        // Return the variable declaration.
        Ok(VariableDeclarationStmt { name, initializer })
    }

    /// Handles statements which are not declarations.
    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.advance_on(TokenType::If) {
            self.if_statement().map(|stmt| Stmt::If(stmt))
        } else if self.advance_on(TokenType::For) {
            // the for statement desugars to multiple wrapped
            // statements, which we handle in the function.
            self.for_statement()
        } else if self.advance_on(TokenType::While) {
            self.while_statement().map(|stmt| Stmt::While(stmt))
        } else if self.advance_on(TokenType::Return) {
            self.return_statement().map(|stmt| Stmt::Return(stmt))
        } else if self.advance_on(TokenType::LeftBrace) {
            self.block_statement().map(|stmt| Stmt::Block(stmt))
        } else {
            self.expression_statement()
        }
    }

    /// Parses a for loop, and creates a desugared while loop representation
    /// ```
    /// for (var i = 1; i <= 10; i = i + 1) {
    ///     print(i);
    /// }
    /// ```
    /// caramalizes to
    /// ```
    /// {
    ///     var i = 1;
    ///     while(i <= 10) {
    ///         print(i);
    ///         i = i + 1;
    ///     }
    /// }
    /// ```
    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.advance_on_or_err(TokenType::LeftParen)?;

        // Parse the initializer
        let initializer = if self.advance_on(TokenType::SemiColon) {
            None
        } else if self.advance_on(TokenType::Var) {
            Some(Stmt::VariableDeclaration(self.var_declaration()?))
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
                body: vec![body, Stmt::Expression(ExpressionStmt { expr: inc })],
            });
        }

        // If the condition is null, set it to a simple literal true value.
        if condition.is_none() {
            condition = Some(Expr::Literal(LiteralExpr {
                token: Token::new(TokenType::True, "true".to_owned(), 0),
            }))
        }

        // Make the body a while loop which executes itself based on the condition
        body = Stmt::While(WhileStmt {
            condition: condition.unwrap(),
            body: Box::new(body),
        });

        // Make the body a block stmt which includes the initializer and the while loop
        if let Some(init) = initializer {
            body = Stmt::Block(BlockStmt {
                body: vec![init, body],
            });
        }

        Ok(body)
    }

    /// Parses a while loop
    fn while_statement(&mut self) -> ParseResult<WhileStmt> {
        self.advance_on_or_err(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.advance_on_or_err(TokenType::RightParen)?;
        let body = self.statement()?;
        Ok(WhileStmt {
            condition,
            body: Box::new(body),
        })
    }

    /// Parse a return statement
    fn return_statement(&mut self) -> ParseResult<ReturnStmt> {
        let return_keyword = self.previous_token();
        let mut value = None;
        if !self.current_token_is_a(TokenType::SemiColon) {
            value = Some(self.expression()?);
        }
        self.advance_on_or_err(TokenType::SemiColon)?;
        Ok(ReturnStmt {
            return_keyword,
            value,
        })
    }

    /// Parses an if statement
    fn if_statement(&mut self) -> ParseResult<IfStmt> {
        self.advance_on_or_err(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.advance_on_or_err(TokenType::RightParen)?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = self
            .advance_on(TokenType::Else)
            .then_try(|| self.statement())?
            .map(|stmt| Box::new(stmt));
        Ok(IfStmt {
            condition,
            then_branch,
            else_branch,
        })
    }

    /// Parses a block stmt
    fn block_statement(&mut self) -> ParseResult<BlockStmt> {
        self.depth += 1;
        let mut statements = vec![];

        while !self.is_at_end() && !self.current_token_is_a(TokenType::RightBrace) {
            if let Some(stmt) = self.parse_item() {
                statements.push(stmt);
            }
        }

        self.advance_on_or_err(TokenType::RightBrace)?;
        self.depth -= 1;
        Ok(BlockStmt { body: statements })
    }

    /// Parses an expression statement
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
                .error(ParseError::InvalidAssignmentTarget(equals.into()))
        }

        Ok(expr)
    }

    /// Parses an or expression
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

    /// Parses an and expression
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

    /// Parses a function call expression
    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        // We loop to support multiple calls for functions that produce functions
        // ```
        // iProduceAFunc()();
        // ```
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
                        .error(ParseError::TooManyFunctionArguments(self.err_ctx()));
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
                // println!("{}: {}", self.previous_token(), self.previous_token().token_type);
                Err(ParseError::ExpectedExpression(self.err_ctx()))
            }
        }
    }

    /// Will advance the current token if it has the given token type, otherwise
    /// it will produce an error with the given message.
    fn advance_on_or_err(&mut self, tt: TokenType) -> ParseResult<Token> {
        self.current_token_is_a(tt.clone())
            .then(|| self.advance())
            .ok_or(ParseError::ExpectedDifferentToken(self.err_ctx(), tt))
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

    /// Convenience method for providing the context for a given ParseError.
    /// If we add to the ParseErrorCtx in the future, we can just update this method.
    fn err_ctx(&self) -> ParseErrorCtx {
        self.current_token().into()
    }

    /// Returns the previous token in the list
    fn previous_token(&mut self) -> Token {
        self.tokens
            .get(self.current - 1)
            .expect("Called unwrap from Parser::peek fn on missing token")
            .clone()
    }
}
