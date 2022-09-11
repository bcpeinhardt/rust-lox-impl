use crate::{
    environment::Environment,
    error::{
        error_reporter::ErrorReporter,
        runtime_error::{RuntimeError, RuntimeErrorCtx},
    },
    grammar::{Expr, Stmt},
    object::LoxObject,
    token::{Token, TokenType},
};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// The interpreter is responsible for "running" the program.
pub struct Interpreter {
    /// Used for the programs memory (storing and retrieving variables, etc)
    environment: Environment,

    pub error_reporter: ErrorReporter,
}

impl Interpreter {
    /// Constructs a new interpreter for running a Lox program.
    pub fn new() -> Self {
        Self {
            environment: Environment::new(None),
            error_reporter: ErrorReporter::new(),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts.into_iter() {
            self.execute(stmt);
        }
    }

    fn execute(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Expression(expr) => {
                if let Err(e) = self.expression_statement(expr) {
                    self.error_reporter.error(e);
                }
            }
            Stmt::Print(expr) => {
                if let Err(e) = self.print_statement(expr) {
                    self.error_reporter.error(e);
                }
            }
            Stmt::VarDecl(name, maybe_expr) => {
                if let Err(e) = self.variable_statement(name, maybe_expr) {
                    self.error_reporter.error(e);
                }
            }
            Stmt::Block(statements) => {
                self.execute_block(statements, Environment::new(Some(self.environment.clone())));
            }
            Stmt::If(condition, then_branch, else_branch) => {
                if let Err(e) = self.if_statement(condition, *then_branch, else_branch.map(|s| *s))
                {
                    self.error_reporter.error(e);
                }
            }
        }
    }

    fn if_statement(
        &mut self,
        condition: Expr,
        then_branch: Stmt,
        else_branch: Option<Stmt>,
    ) -> RuntimeResult<()> {
        if self.evaluate(condition)?.is_truthy() {
            self.execute(then_branch);
        } else if let Some(stmt) = else_branch {
            self.execute(stmt);
        }
        Ok(())
    }

    fn execute_block(&mut self, statements: Vec<Stmt>, environment: Environment) {
        let previous = self.environment.clone();
        self.execute_block_failable(statements, environment);
        self.environment = previous;
    }

    fn execute_block_failable(&mut self, statements: Vec<Stmt>, environment: Environment) {
        self.environment = environment.clone();
        for stmt in statements.into_iter() {
            self.execute(stmt);
        }
    }

    fn expression_statement(&mut self, expr: Expr) -> RuntimeResult<()> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn print_statement(&mut self, expr: Expr) -> RuntimeResult<()> {
        let res = self.evaluate(expr)?;
        println!("{}", res);
        Ok(())
    }

    fn variable_statement(&mut self, name: Token, initializer: Option<Expr>) -> RuntimeResult<()> {
        let mut value = LoxObject::Nil;
        if let Some(expr) = initializer {
            value = self.evaluate(expr)?;
        }
        self.environment.define(&name.lexeme, value);
        Ok(())
    }

    /// Top level function for evaluating an expression
    fn evaluate(&mut self, expr: Expr) -> RuntimeResult<LoxObject> {
        match expr {
            Expr::Binary(lhs, operator, rhs) => self.evaluate_binary(*lhs, operator, *rhs),
            Expr::Grouping(e) => {
                // For a grouping just recursively evaluate the inner expression
                self.evaluate(*e)
            }
            Expr::Literal(token) => self.evaluate_literal(token),
            Expr::Unary(operator, e) => self.evaluate_unary(operator, *e),
            Expr::Variable(token) => self.environment.get(token),
            Expr::Assignment(name, expr) => {
                let value = self.evaluate(*expr)?;
                self.environment.assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Logical(lhs, operator, rhs) => {
                self.evaluate_logical_expression(*lhs, operator, *rhs)
            }
            Expr::Call(callee, closing_paren, args) => {
                self.evaluate_call_expr(*callee, closing_paren, args)
            },
        }
    }

    fn evaluate_call_expr(&mut self, callee: Expr, closing_paren: Token, args: Vec<Expr>) -> RuntimeResult<LoxObject> { 
        let mut callee = self.evaluate(callee)?;

        let args_evaluated = args.into_iter().map(|arg| self.evaluate(arg)).collect();
    }

    fn evaluate_logical_expression(
        &mut self,
        lhs: Expr,
        operator: Token,
        rhs: Expr,
    ) -> RuntimeResult<LoxObject> {
        let left = self.evaluate(lhs)?;
        if (operator.token_type == TokenType::Or && left.is_truthy())
            || !left.is_truthy()
        {
            // Short circuit
            Ok(left)
        } else {
            // Doesn't short circuit, must evaluate rhs
            self.evaluate(rhs)
        }
    }

    /// Converts a unary expression into a LoxObject
    fn evaluate_unary(&mut self, operator: Token, expr: Expr) -> RuntimeResult<LoxObject> {
        // Evaluate the right hand side expression
        let right = self.evaluate(expr)?;

        match operator.token_type {
            TokenType::Bang => {
                // !some_var should return a boolean based on whether the object
                // conforms to Lox's conception of "truthiness"
                Ok(LoxObject::Boolean(!right.is_truthy()))
            }
            TokenType::Minus => {
                // The unary minus negates a number, but for anything else produces
                // a Runtime Error
                if let LoxObject::Number(n) = right {
                    Ok(LoxObject::Number(-n))
                } else {
                    Err(RuntimeError::new(
                        operator.clone(),
                        "Unary '-' can only be applied to numbers.",
                    ))
                }
            }
            _ => {
                // No other operators other than ! and - can be used in a unary way.
                Err(RuntimeError::new(
                    operator.clone(),
                    format!("token '{}' cannot be used as unary", operator.lexeme),
                ))
            }
        }
    }

    /// Converts a binary expression into a LoxObject
    fn evaluate_binary(
        &mut self,
        lhs: Expr,
        operator: Token,
        rhs: Expr,
    ) -> RuntimeResult<LoxObject> {
        // Evaluate tje left and right expressions.
        let left = self.evaluate(lhs)?;
        let right = self.evaluate(rhs)?;

        match operator.token_type {
            TokenType::Minus => {
                // Applies to numbers only
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    Ok(LoxObject::Number(l - r))
                } else {
                    Err(RuntimeError::new(
                        operator,
                        "Can only subtract number types",
                    ))
                }
            }
            TokenType::Plus => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left.clone(), right.clone())
                {
                    // Adds Numbers
                    Ok(LoxObject::Number(l + r))
                } else if let (LoxObject::String(mut l), LoxObject::String(r)) = (left, right) {
                    // Concatenates strings
                    l.push_str(&r);
                    Ok(LoxObject::String(l))
                } else {
                    Err(RuntimeError::new(
                        operator,
                        "Can only add number + number or concatenate string + string",
                    ))
                }
            }
            TokenType::Star => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    Ok(LoxObject::Number(l * r))
                } else {
                    Err(RuntimeError::new(
                        operator,
                        "Can only multiply number types",
                    ))
                }
            }
            TokenType::Slash => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    Ok(LoxObject::Number(l / r))
                } else {
                    Err(RuntimeError::new(operator, "Can only divide number types"))
                }
            }
            TokenType::Greater => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    return Ok(LoxObject::Boolean(l > r));
                } else {
                    Err(RuntimeError::new(
                        operator,
                        "Can only compare number types with >",
                    ))
                }
            }
            TokenType::GreaterEqual => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    return Ok(LoxObject::Boolean(l >= r));
                } else {
                    Err(RuntimeError::new(
                        operator,
                        "Can only compare number types with >=",
                    ))
                }
            }
            TokenType::Less => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    return Ok(LoxObject::Boolean(l < r));
                } else {
                    Err(RuntimeError::new(
                        operator,
                        "Can only compare number types with <",
                    ))
                }
            }
            TokenType::LessEqual => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    return Ok(LoxObject::Boolean(l <= r));
                } else {
                    Err(RuntimeError::new(
                        operator,
                        "Can only compare number types with <=",
                    ))
                }
            }
            TokenType::EqualEqual => {
                return Ok(LoxObject::Boolean(left == right));
            }
            TokenType::BangEqual => {
                return Ok(LoxObject::Boolean(left != right));
            }
            _ => {
                // Error out at the end of the match
                Err(RuntimeError::new(
                    operator.clone(),
                    format!("Cannot use token {} for binary operation", operator.lexeme),
                ))
            }
        }
    }

    /// Transform an Expr::Literal's token into a LoxObject
    fn evaluate_literal(&self, token: Token) -> RuntimeResult<LoxObject> {
        match token.token_type {
            TokenType::String(s) => Ok(LoxObject::String(s)),
            TokenType::Number(n) => Ok(LoxObject::Number(n)),
            TokenType::True => Ok(LoxObject::Boolean(true)),
            TokenType::False => Ok(LoxObject::Boolean(false)),
            TokenType::Nil => Ok(LoxObject::Nil),
            _ => Err(RuntimeError::new(
                token.clone(),
                format!("Tried to evaluate token '{}' as literal", token.lexeme),
            )),
        }
    }
}
