use std::convert::TryInto;

use crate::{object::LoxObject, scanner::{Token, TokenType}, lox::Lox, environment::Environment, expr::Expr, stmt::Stmt, error::{ErrorReporter, RuntimeResult, RuntimeError}};

/// The interpreter is responsible for "running" the program. 
pub struct Interpreter {

    /// Used for the programs memory (storing and retrieving variables, etc)
    environment: Environment
}

impl Interpreter {
    
    /// Constructs a new interpreter for runnign a Lox program.
    pub fn new() -> Self {
        Self {
            environment: Environment::new()
        }
    }
    
    pub fn interpret(&mut self, stmts: Vec<Stmt>, mut error_reporter: ErrorReporter) -> ErrorReporter {
        for stmt in stmts.into_iter() {
            match stmt {
                Stmt::Expression(expr) => {
                    match self.expression_statement(expr) {
                        Ok(_) => {},
                        Err(e) => {
                            error_reporter.runtime_error(e);
                        },
                    }
                },
                Stmt::Print(expr) => {
                    match self.print_statement(expr) {
                        Ok(_) => {},
                        Err(e) => {
                            error_reporter.runtime_error(e);
                        },
                    }
                },
                Stmt::VarDecl(name, maybe_expr) => {
                    match self.variable_statement(name, maybe_expr) {
                        Ok(_) => {},
                        Err(e) => {
                            error_reporter.runtime_error(e);
                        },
                    }
                },
            }
        }
        error_reporter
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
    fn evaluate(&self, expr: Expr) -> RuntimeResult<LoxObject> {
        match expr {
            Expr::Binary(lhs, operator, rhs) => {
                self.evaluate_binary(*lhs, operator, *rhs)
            },
            Expr::Grouping(e) => {
                // For a grouping just recursively evaluate the inner expression
                self.evaluate(*e)
            },
            Expr::Literal(token) => {
                self.evaluate_literal(token)
            },
            Expr::Unary(operator, e) => {
                self.evaluate_unary(operator, *e)
            },
            Expr::Variable(token) => {
                self.environment.get(token)
            },
        }
    }

    /// Converts a unary expression into a LoxObject
    fn evaluate_unary(&self, operator: Token, expr: Expr) -> RuntimeResult<LoxObject> {
        
        // Evaluate the right hand side expression
        let right = self.evaluate(expr)?;

        match operator.token_type {
            TokenType::Bang => {
                // !some_var should return a boolean based on whether the object
                // conforms to Lox's conception of "truthiness"
                Ok(LoxObject::Boolean(!right.is_truthy()))
            },
            TokenType::Minus => { 
                // The unary minus negates a number, but for anything else produces
                // a Runtime Error
                if let LoxObject::Number(n) = right {
                    Ok(LoxObject::Number(-n))
                } else {
                    Err(RuntimeError::new(operator, "unary minus can only be used with numbers"))
                }
            },
            _ => {
                // No other operators other than ! and - can be used in a unary way.
                Err(RuntimeError::new(operator.clone(), &format!("token '{}' cannot be used as unary", operator.lexeme)))
            }
        }
    }

    /// Converts a binary expression into a LoxObject
    fn evaluate_binary(&self, lhs: Expr, operator: Token, rhs: Expr) -> RuntimeResult<LoxObject> { 

        // Evaluate tje left and right expressions.
        let left = self.evaluate(lhs)?;
        let right = self.evaluate(rhs)?;

        match operator.token_type {
            TokenType::Minus => {
                // Applies to numbers only
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    Ok(LoxObject::Number(l - r))
                } else {
                    Err(RuntimeError::new(operator, "Can only subtract number types"))
                }
            },
            TokenType::Plus => { 
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left.clone(), right.clone()) {
                    // Adds Numbers
                    Ok(LoxObject::Number(l + r))
                } else if let (LoxObject::String(mut l), LoxObject::String(r)) = (left, right) { 
                    // Concatenates strings
                    l.push_str(&r);
                    Ok(LoxObject::String(l))
                } else {
                    Err(RuntimeError::new(operator, "Can only add number + number or concatenate string + string"))
                }
            },
            TokenType::Star => { 
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    Ok(LoxObject::Number(l * r))
                } else {
                    Err(RuntimeError::new(operator, "Can only multiply number types"))
                }
            },
            TokenType::Slash => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    Ok(LoxObject::Number(l / r))
                } else {
                    Err(RuntimeError::new(operator, "Can only divide number types"))
                }
            },
            TokenType::Greater => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    return Ok(LoxObject::Boolean(l > r));
                } else {
                    Err(RuntimeError::new(operator, "Can only compare number types with >"))
                }
            },
            TokenType::GreaterEqual => { 
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    return Ok(LoxObject::Boolean(l >= r));
                } else {
                    Err(RuntimeError::new(operator, "Can only compare number types with >="))
                }
            },
            TokenType::Less => { 
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    return Ok(LoxObject::Boolean(l < r));
                } else {
                    Err(RuntimeError::new(operator, "Can only compare number types with <"))
                }
            },
            TokenType::LessEqual => { 
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (left, right) {
                    return Ok(LoxObject::Boolean(l <= r));
                } else {
                    Err(RuntimeError::new(operator, "Can only compare number types with <="))
                }
            },
            TokenType::EqualEqual => { 
                return Ok(LoxObject::Boolean(left == right));
            },
            TokenType::BangEqual => { 
                return Ok(LoxObject::Boolean(left != right));
            }
            _ => {
                // Error out at the end of the match
                Err(RuntimeError::new(operator.clone(), &format!("Cannot use token {} for binary operation", operator.lexeme)))
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
            _ => Err(RuntimeError::new(token.clone(), &format!("Tried to evaluate token '{}' as literal", token.lexeme)))
        }
    }
}