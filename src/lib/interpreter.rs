use std::convert::TryInto;

use crate::{parser::Expr, object::LoxObject, scanner::{Token, TokenType}, lox::Lox};

pub struct RuntimeError {
    pub token: Token,
    pub msg: String,
}

/// Any interpreter function which can error should report a Runtime Error for the 
type InterpreterResult<T> = Result<T, RuntimeError>;

impl RuntimeError {
    pub fn new(token: Token, msg: &str) -> Self {
        Self {
            token,
            msg: msg.to_owned()
        }
    }
}

pub struct Interpreter<'a> {
    lox: &'a mut Lox
}

impl<'a> Interpreter<'a> {
    
    pub fn new(lox: &'a mut Lox) -> Self {
        Self {
            lox
        }
    }
    
    pub fn interpret(&mut self, expr: Expr) {
        match self.evaluate(expr) {
            Ok(object) => {
                println!("{}", object)
            },
            Err(e) => {
                self.lox.runtime_error(e);
            },
        }
    }

    /// Top level function for evaluating an expression
    fn evaluate(&self, expr: Expr) -> InterpreterResult<LoxObject> {
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
        }
    }

    fn evaluate_unary(&self, operator: Token, expr: Expr) -> InterpreterResult<LoxObject> {
        
        // Evaluate the right hand side expression
        let right = self.evaluate(expr)?;

        match operator.token_type {
            TokenType::Bang => {
                // !some_var should return a boolean based on whether the object
                // conforms to Lox's conception of "truthiness"
                Ok(LoxObject::Boolean(right.is_truthy()))
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
    fn evaluate_binary(&self, lhs: Expr, operator: Token, rhs: Expr) -> InterpreterResult<LoxObject> { 
        let left = self.evaluate(lhs)?;
        let right = self.evaluate(rhs)?;

        match operator.token_type {
            TokenType::Minus => {
                // Applies to numbers only
                if let LoxObject::Number(left_num) = left {
                    if let LoxObject::Number(right_num) = right {
                        return Ok(LoxObject::Number(left_num - right_num));
                    }
                }
            },
            TokenType::Plus => { 
                // Adds Numbers
                if let LoxObject::Number(left_num) = left {
                    if let LoxObject::Number(right_num) = right {
                        return Ok(LoxObject::Number(left_num + right_num));
                    }
                }

                // Concatenates strings
                if let LoxObject::String(left_str) = left { 
                    if let LoxObject::String(right_str) = right { 
                        let mut buf = String::new();
                        buf.push_str(&left_str);
                        buf.push_str(&right_str);
                        return Ok(LoxObject::String(buf));
                    }
                }
            },
            TokenType::Star => { 
                if let LoxObject::Number(left_num) = left {
                    if let LoxObject::Number(right_num) = right {
                        return Ok(LoxObject::Number(left_num * right_num));
                    }
                }
            },
            TokenType::Slash => {
                if let LoxObject::Number(left_num) = left {
                    if let LoxObject::Number(right_num) = right {
                        return Ok(LoxObject::Number(left_num / right_num));
                    }
                }
            },
            TokenType::Greater => {
                if let LoxObject::Number(left_num) = left {
                    if let LoxObject::Number(right_num) = right {
                        return Ok(LoxObject::Boolean(left_num > right_num));
                    }
                }
            },
            TokenType::GreaterEqual => { 
                if let LoxObject::Number(left_num) = left {
                    if let LoxObject::Number(right_num) = right {
                        return Ok(LoxObject::Boolean(left_num >= right_num));
                    }
                }
            },
            TokenType::Less => { 
                if let LoxObject::Number(left_num) = left {
                    if let LoxObject::Number(right_num) = right {
                        return Ok(LoxObject::Boolean(left_num < right_num));
                    }
                }
            },
            TokenType::LessEqual => { 
                if let LoxObject::Number(left_num) = left {
                    if let LoxObject::Number(right_num) = right {
                        return Ok(LoxObject::Boolean(left_num <= right_num));
                    }
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
            }
        }

        Err(RuntimeError::new(operator, "Invalid expression"))
    }

    /// Transform an Expr::Literal's token into a LoxObject
    fn evaluate_literal(&self, token: Token) -> InterpreterResult<LoxObject> { 
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