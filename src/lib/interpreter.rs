use crate::{
    environment::Environment,
    error::{error_reporter::ErrorReporter, runtime_error::RuntimeError},
    function::LoxFunction,
    grammar::{
        AssignmentExpr, BinaryExpr, BlockStmt, CallExpr, Expr, FunctionDeclarationStmt,
        GroupingExpr, IfStmt, LiteralExpr, ReturnStmt, Stmt, UnaryExpr, VariableDeclarationStmt,
        VariableExpr, WhileStmt,
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

    /// Executes a list of Lox Statements in a dedicated environment.
    pub fn interpret(&mut self, stmts: Vec<Stmt>) {
        let mut environment = Environment::new();
        for stmt in stmts.into_iter() {
            self.execute(stmt, &mut environment);
        }
    }

    /// Execute a single Lox statement in the given environemt. Returns an optional
    /// return value for handling early returns (i.e. a return statement halfway through
    /// a function body)
    pub fn execute(&mut self, stmt: Stmt, exec_env: &mut Environment) -> Option<LoxObject> {
        match stmt {
            // An expression statement doesn't return anything, so just
            // evaluate the expr and report an error if there is one.
            // Then return None.
            Stmt::Expression(stmt) => {
                let _ = self.evaluate(stmt.expr, exec_env).map_err(|e| self.error_reporter.error(e));
                None
            }
            // An variable declaration statement doesn't return anything, so just
            // execute the stmt and report an error if there is one.
            // Then return None.
            Stmt::VariableDeclaration(var_dec_stmt) => {
                let _ = self.variable_statement(var_dec_stmt, exec_env).map_err(|e| self.error_reporter.error(e));
                None
            }
            // Executing a successfully parsed block stmt won't fail,
            // (if the body fails to execute because of some error, it will
            // be handled by another branch of this match stmt)
            // so just bubble up the optional return value.
            Stmt::Block(block_stmt) => {
                self.execute_block(block_stmt, exec_env)
            }
            // An if stmt can fail (because it has to evaluate the condition) 
            // and can have a return value, so if there's an
            // error report it and return `None`, or if no error bubble up the 
            // optional return value.
            Stmt::If(if_stmt) => self.if_statement(if_stmt, exec_env).map_err(|e| self.error_reporter.error(e)).ok().flatten()
            ,
            // Interpreting a function declaration statement doesn't return anything
            // and can't fail, so just
            // execute the stmt and return None.
            Stmt::FunctionDeclaration(func_decl_stmt) => {
                self.function_declaration(func_decl_stmt, exec_env);
                None
            }
            // Return statement always returns something, hence the name. Report any error then throw it away
            // and return the return value.
            Stmt::Return(return_stmt) => {
                self.return_statement(return_stmt, exec_env).map_err(|e| self.error_reporter.error(e)).ok()
            }
            // A while stmt can fail (because it has to evaluate the condition) 
            // and can have a return value, so if there's an
            // error report it and return `None`, or if no error bubble up the 
            // optional return value.
            Stmt::While(while_stmt) => self.while_statement(while_stmt, exec_env).map_err(|e| self.error_reporter.error(e)).ok().flatten(),
        }
    }

    /// Execute a while statement
    fn while_statement(
        &mut self,
        WhileStmt { condition, body }: WhileStmt,
        exec_env: &mut Environment,
    ) -> RuntimeResult<Option<LoxObject>> {
        // If the condition evaluates without an error and the result
        // is "truthy", execute the body.
        while self.evaluate(condition.clone(), exec_env)?.is_truthy() {
            // Execute the body of the while statement. If if has a return value (i.e. we hit a return statement),
            // return the return value.
            let maybe_return = self.execute(*body.clone(), exec_env);
            if maybe_return.is_some() {
                return Ok(maybe_return);
            }
        }

        // No return statement was hit in the while loop,
        // so return None.
        Ok(None)
    }

    /// Executes a return statement.
    fn return_statement(
        &mut self,
        ReturnStmt { value, .. }: ReturnStmt,
        exec_env: &mut Environment,
    ) -> RuntimeResult<LoxObject> {
        // Evaluate the expression if one was provided, otherwise return nil.
        value
            .map(|expr| self.evaluate(expr, exec_env))
            .transpose()
            .map(|maybe_val| maybe_val.unwrap_or(LoxObject::Nil))
    }

    /// Execute a function declaration statement. A function declaration statement cant cause
    /// a return and can't fail (because by this point it has been parsed), so it doesn't
    /// return anything.
    fn function_declaration(
        &mut self,
        func_decl_stmt: FunctionDeclarationStmt,
        exec_env: &mut Environment,
    ) {
        // Create a LoxObject for the function and define it in the current scope.
        let name = func_decl_stmt.name.clone();
        let function = LoxFunction::from(func_decl_stmt);
        exec_env.define(&name.lexeme, LoxObject::Function(Box::new(function)));
    }

    /// Executes an if statement.
    fn if_statement(
        &mut self,
        IfStmt {
            condition,
            then_branch,
            else_branch,
        }: IfStmt,
        exec_env: &mut Environment,
    ) -> RuntimeResult<Option<LoxObject>> {
        Ok(if self.evaluate(condition, exec_env)?.is_truthy() {
            // If the condition evaluates to true, execute the if branch.
            self.execute(*then_branch, exec_env)
        } else if let Some(stmt) = else_branch {
            // If the condition evaluates to false and there's an else branch, execute it.
            self.execute(*stmt, exec_env)
        } else {
            // We never executed anything so return None.
            None
        })
    }

    /// Executes a block statement
    pub fn execute_block(
        &mut self,
        BlockStmt { body }: BlockStmt,
        exec_env: &mut Environment,
    ) -> Option<LoxObject> {
        // In a new block scope
        exec_env.in_new_local_scope(|e| {
            for stmt in body.into_iter() {
                // Execute each statement in the block and return if necessary.
                let maybe_return = self.execute(stmt, e);
                if maybe_return.is_some() {
                    return maybe_return;
                }
            }

            // Otherwise, no return value
            None
        })
    }

    /// Executes a variable declaration statement.
    fn variable_statement(
        &mut self,
        VariableDeclarationStmt { name, initializer }: VariableDeclarationStmt,
        exec_env: &mut Environment,
    ) -> RuntimeResult<()> {
        // Evaluate the initializer if one was provided, or
        // default to nil.
        let value = initializer
            .map(|expr| self.evaluate(expr, exec_env))
            .transpose()?
            .unwrap_or(LoxObject::Nil);

        // Define the variable in the current scope.
        exec_env.define(&name.lexeme, value);
        Ok(())
    }

    /// Top level function for evaluating an expression
    fn evaluate(&mut self, expr: Expr, exec_env: &mut Environment) -> RuntimeResult<LoxObject> {
        match expr {
            Expr::Binary(binary) => self.evaluate_binary(binary, exec_env),

            // For a grouping, just evaluate the inner expression.
            Expr::Grouping(GroupingExpr { expr }) => self.evaluate(*expr, exec_env),
            Expr::Literal(literal) => Ok(self.evaluate_literal(literal)),
            Expr::Unary(unary) => self.evaluate_unary(unary, exec_env),

            // For a variable, just lookup the variable in the environment.
            Expr::Variable(VariableExpr { name }) => exec_env.get(name),
            Expr::Assignment(assignment) => self.evaluate_assignment(assignment, exec_env),
            Expr::Logical(binary) => self.evaluate_logical_expression(binary, exec_env),
            Expr::Call(call) => self.evaluate_call_expr(call, exec_env),
        }
    }

    /// Evaluate an assignment expression.
    /// Note: A variable assignment expression evaluates
    /// to the new value of the variable. This has interesting
    /// implications, as it allows syntax like
    /// ```lox
    /// if (a = true) {
    ///     // executes
    /// }
    /// ```
    fn evaluate_assignment(
        &mut self,
        AssignmentExpr { variable, expr }: AssignmentExpr,
        exec_env: &mut Environment,
    ) -> RuntimeResult<LoxObject> {
        // Evaluate the expression
        let value = self.evaluate(*expr, exec_env)?;

        // Update the variable in the environment to be the value of the expression.
        exec_env.assign(variable, value.clone())?;

        // Return the new value
        Ok(value)
    }

    /// Evaluates a call expression.
    fn evaluate_call_expr(
        &mut self,
        CallExpr {
            callee,
            closing_paren,
            args,
        }: CallExpr,
        exec_env: &mut Environment,
    ) -> RuntimeResult<LoxObject> {
        // Lookup the function in the environment by evaluating the variable.
        let callee = self.evaluate(*callee, exec_env)?;

        // Evaluate each argument of the function
        let args = args
            .into_iter()
            .map(|arg| self.evaluate(arg, exec_env))
            .collect::<Result<Vec<_>, _>>()?;

        if let LoxObject::Function(function) = callee {
            if args.len() != function.arity() {
                Err(RuntimeError::new(
                    closing_paren,
                    format!(
                        "Expect {} arguments but got {}",
                        function.arity(),
                        args.len()
                    ),
                ))
            } else {
                Ok(function.call(self, exec_env, args))
            }
        } else {
            Err(RuntimeError::new(
                closing_paren,
                "Can only call functions and classes.",
            ))
        }
    }

    // Evaluates `and` and `or` expressions
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
            // We derive PartialEq on LoxObject so these are freebies
            TokenType::EqualEqual => Ok(LoxObject::Boolean(left == right)),
            TokenType::BangEqual => Ok(LoxObject::Boolean(left != right)),

            // The `+` operator adds numbers and concatenates strings in lox, so we
            // handle both cases and error otherwise.
            TokenType::Plus => {
                if let (LoxObject::Number(l), LoxObject::Number(r)) = (&left, &right) {
                    Ok(LoxObject::Number(l + r))
                } else if let (LoxObject::String(mut l), LoxObject::String(r)) = (left, right) {
                    l.push_str(&r);
                    Ok(LoxObject::String(l))
                } else {
                    Err(RuntimeError::new(
                        operator,
                        "Can only add number + number or concatenate string + string",
                    ))
                }
            }
            _ => {
                // The rest of the operators only apply to numbers, so we can build the error
                // and try to downcast the LoxObjects into f64s once, then apply them appropriately.
                let error: RuntimeError = RuntimeError::new(
                    operator.clone(),
                    format!(
                        "Operator `{}` only applies to number types",
                        operator.lexeme
                    ),
                );
                let l = f64::try_from(left).map_err(|_| error.clone())?;
                let r = f64::try_from(right).map_err(|_| error.clone())?;

                match operator.token_type {
                    TokenType::Minus => Ok(LoxObject::Number(l - r)),
                    TokenType::Star => Ok(LoxObject::Number(l * r)),
                    TokenType::Slash => Ok(LoxObject::Number(l / r)),
                    TokenType::Greater => Ok(LoxObject::Boolean(l > r)),
                    TokenType::GreaterEqual => Ok(LoxObject::Boolean(l >= r)),
                    TokenType::Less => Ok(LoxObject::Boolean(l < r)),
                    TokenType::LessEqual => Ok(LoxObject::Boolean(l <= r)),
                    _ => {
                        // Error out at the end of the match
                        Err(RuntimeError::new(
                            operator.clone(),
                            format!("Cannot use token {} for binary operation", operator.lexeme),
                        ))
                    }
                }
            }
        }
    }

    /// Transform an Expr::Literal's token into a LoxObject
    /// # Panics
    /// Panics if the token within the parse LiteralExpr is not a Literal
    fn evaluate_literal(&self, LiteralExpr { token }: LiteralExpr) -> LoxObject {
        match token.token_type {
            TokenType::String(s) => LoxObject::String(s),
            TokenType::Number(n) => LoxObject::Number(n),
            TokenType::True => LoxObject::Boolean(true),
            TokenType::False => LoxObject::Boolean(false),
            TokenType::Nil => LoxObject::Nil,
            _ => panic!("Parsed token {} as a Literal", token),
        }
    }
}
