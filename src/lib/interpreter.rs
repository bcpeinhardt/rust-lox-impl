use crate::{
    callable::{Clock, LoxCallable, PrintEnv},
    environment::{self, Environment, Scope},
    error::{error_reporter::ErrorReporter, runtime_error::RuntimeError},
    function::LoxFunction,
    grammar::{
        AssignmentExpr, BinaryExpr, BlockStmt, CallExpr, Expr, ExpressionStmt,
        FunctionDeclarationStmt, GroupingExpr, IfStmt, LiteralExpr, PrintStmt, ReturnStmt, Stmt,
        UnaryExpr, VariableDeclarationStmt, VariableExpr, WhileStmt,
    },
    object::LoxObject,
    token::TokenType,
};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// The interpreter is responsible for "running" the program.
#[derive(Clone)]
pub struct Interpreter {
    pub error_reporter: ErrorReporter,
}

impl Interpreter {
    /// Constructs a new interpreter for running a Lox program.
    pub fn new() -> Self {
        Self {
            error_reporter: ErrorReporter::new(),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) {
        let mut environment = Environment::new();
        for stmt in stmts.into_iter() {
            self.execute(stmt, &mut environment);
        }
    }

    pub fn execute(&mut self, stmt: Stmt, exec_env: &mut Environment) -> Option<LoxObject> {
        match stmt {
            Stmt::Expression(expr_stmt) => {
                if let Err(e) = self.expression_statement(expr_stmt, exec_env) {
                    self.error_reporter.error(e);
                }
            }
            Stmt::Print(print_stmt) => {
                if let Err(e) = self.print_statement(print_stmt, exec_env) {
                    self.error_reporter.error(e);
                }
            }
            Stmt::VariableDeclaration(var_dec_stmt) => {
                if let Err(e) = self.variable_statement(var_dec_stmt, exec_env) {
                    self.error_reporter.error(e);
                }
            }
            Stmt::Block(block_stmt) => {
                return self.execute_block(block_stmt, exec_env);
            }
            Stmt::If(if_stmt) => match self.if_statement(if_stmt, exec_env) {
                Ok(maybe_return_value) => match maybe_return_value {
                    Some(value) => {
                        return Some(value);
                    }
                    None => {}
                },
                Err(e) => {
                    self.error_reporter.error(e);
                }
            },
            Stmt::FunctionDeclaration(func_decl_stmt) => {
                self.function_declaration_statement(func_decl_stmt, exec_env);
            }
            Stmt::Return(ReturnStmt { value, .. }) => {
                let return_value = if let Some(expr) = value {
                    match self.evaluate(expr, exec_env) {
                        Ok(obj) => obj,
                        Err(e) => {
                            self.error_reporter.error(e);
                            LoxObject::Nil
                        }
                    }
                } else {
                    LoxObject::Nil
                };
                return Some(return_value);
            }
            Stmt::While(WhileStmt { condition, body }) => loop {
                match self.evaluate(condition.clone(), exec_env) {
                    Ok(obj) => {
                        if obj.is_truthy() {
                            if let Some(return_value) = self.execute(*body.clone(), exec_env) {
                                return Some(return_value);
                            }
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        self.error_reporter.error(e);
                    }
                }
            },
        }
        None
    }

    fn function_declaration_statement(
        &mut self,
        func_decl_stmt: FunctionDeclarationStmt,
        exec_env: &mut Environment,
    ) {
        let name = func_decl_stmt.name.clone();
        let function = LoxFunction::from(func_decl_stmt);
        exec_env.define(&name.lexeme, LoxObject::Function(function));
    }

    fn if_statement(
        &mut self,
        IfStmt {
            condition,
            then_branch,
            else_branch,
        }: IfStmt,
        exec_env: &mut Environment,
    ) -> RuntimeResult<Option<LoxObject>> {
        if self.evaluate(condition, exec_env)?.is_truthy() {
            let val = self.execute(*then_branch, exec_env);
            Ok(val)
        } else if let Some(stmt) = else_branch {
            Ok(self.execute(*stmt, exec_env))
        } else {
            Ok(None)
        }
    }

    pub fn execute_block(
        &mut self,
        BlockStmt { body }: BlockStmt,
        exec_env: &mut Environment,
    ) -> Option<LoxObject> {
        let mut cloned_interpreter = self.clone();
        exec_env.in_new_local_scope(|e| {
            for stmt in body.into_iter() {
                let maybe_return = cloned_interpreter.execute(stmt, e);
                if maybe_return.is_some() {
                    return maybe_return;
                }
            }
            None
        })
    }

    fn expression_statement(
        &mut self,
        ExpressionStmt { expr }: ExpressionStmt,
        exec_env: &mut Environment,
    ) -> RuntimeResult<()> {
        self.evaluate(expr, exec_env)?;
        Ok(())
    }

    fn print_statement(
        &mut self,
        PrintStmt { expr }: PrintStmt,
        exec_env: &mut Environment,
    ) -> RuntimeResult<()> {
        let res = self.evaluate(expr, exec_env)?;
        println!("{}", res);
        Ok(())
    }

    fn variable_statement(
        &mut self,
        VariableDeclarationStmt { name, initializer }: VariableDeclarationStmt,
        exec_env: &mut Environment,
    ) -> RuntimeResult<()> {
        let mut value = LoxObject::Nil;
        if let Some(expr) = initializer {
            value = self.evaluate(expr, exec_env)?;
        }
        exec_env.define(&name.lexeme, value);
        Ok(())
    }

    /// Top level function for evaluating an expression
    fn evaluate(&mut self, expr: Expr, exec_env: &mut Environment) -> RuntimeResult<LoxObject> {
        match expr {
            Expr::Binary(binary) => self.evaluate_binary(binary, exec_env),

            // For a grouping, just evaluate the inner expression.
            Expr::Grouping(GroupingExpr { expr }) => self.evaluate(*expr, exec_env),
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Unary(unary) => self.evaluate_unary(unary, exec_env),

            // For a variable, just lookup the variable in the environment.
            Expr::Variable(VariableExpr { name }) => exec_env.get(name),
            Expr::Assignment(assignment) => self.evaluate_assignment(assignment, exec_env),
            Expr::Logical(binary) => self.evaluate_logical_expression(binary, exec_env),
            Expr::Call(call) => self.evaluate_call_expr(call, exec_env),
        }
    }

    fn evaluate_assignment(
        &mut self,
        AssignmentExpr { variable, expr }: AssignmentExpr,
        exec_env: &mut Environment,
    ) -> RuntimeResult<LoxObject> {
        let value = self.evaluate(*expr, exec_env)?;
        exec_env.assign(variable, value.clone())?;
        Ok(value)
    }

    fn evaluate_call_expr(
        &mut self,
        CallExpr {
            callee,
            closing_paren,
            args,
        }: CallExpr,
        exec_env: &mut Environment,
    ) -> RuntimeResult<LoxObject> {
        let callee = self.evaluate(*callee, exec_env)?;

        let mut args_evaluated = vec![];
        let len = args.len();
        for arg in args.into_iter() {
            args_evaluated.push(self.evaluate(arg, exec_env)?);
        }

        if let LoxObject::Function(mut function) = callee {
            if len != function.arity() {
                Err(RuntimeError::new(
                    closing_paren,
                    format!("Expect {} arguments but got {}", function.arity(), len),
                ))
            } else {
                Ok(function.call(self, exec_env, args_evaluated))
            }
        } else if let LoxObject::Clock(mut function) = callee {
            if len != function.arity() {
                Err(RuntimeError::new(
                    closing_paren,
                    format!("Expect {} arguments but got {}", function.arity(), len),
                ))
            } else {
                Ok(function.call(self, exec_env, args_evaluated))
            }
        } else if let LoxObject::PrintEnv(mut function) = callee {
            if len != function.arity() {
                Err(RuntimeError::new(
                    closing_paren,
                    format!("Expect {} arguments but got {}", function.arity(), len),
                ))
            } else {
                Ok(function.call(self, exec_env, args_evaluated))
            }
        } else {
            Err(RuntimeError::new(
                closing_paren,
                "Can only call functions and classes.",
            ))
        }
    }

    fn evaluate_logical_expression(
        &mut self,
        BinaryExpr { lhs, operator, rhs }: BinaryExpr,
        exec_env: &mut Environment,
    ) -> RuntimeResult<LoxObject> {
        let left = self.evaluate(*lhs, exec_env)?;
        if (operator.token_type == TokenType::Or && left.is_truthy()) || !left.is_truthy() {
            // Short circuit
            Ok(left)
        } else {
            // Doesn't short circuit, must evaluate rhs
            self.evaluate(*rhs, exec_env)
        }
    }

    /// Converts a unary expression into a LoxObject
    fn evaluate_unary(
        &mut self,
        UnaryExpr { operator, rhs }: UnaryExpr,
        exec_env: &mut Environment,
    ) -> RuntimeResult<LoxObject> {
        // Evaluate the right hand side expression
        let right = self.evaluate(*rhs, exec_env)?;

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
        BinaryExpr { lhs, operator, rhs }: BinaryExpr,
        exec_env: &mut Environment,
    ) -> RuntimeResult<LoxObject> {
        // Evaluate the left and right expressions.
        let left = self.evaluate(*lhs, exec_env)?;
        let right = self.evaluate(*rhs, exec_env)?;

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
            TokenType::EqualEqual => Ok(LoxObject::Boolean(left == right)),
            TokenType::BangEqual => Ok(LoxObject::Boolean(left != right)),
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
    fn evaluate_literal(&self, LiteralExpr { token }: LiteralExpr) -> RuntimeResult<LoxObject> {
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
