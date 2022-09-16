use uuid::Uuid;

use crate::{
    callable::LoxCallable,
    environment::{self, Environment, Scope},
    grammar::{BlockStmt, FunctionDeclarationStmt, Stmt},
    interpreter::Interpreter,
    object::LoxObject,
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
}

impl LoxFunction {
    /// Construct a function object from the function declaration statement parsed by the parser.
    pub fn from(FunctionDeclarationStmt { name, params, body }: FunctionDeclarationStmt) -> Self {
        Self { name, params, body }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        exec_env: &mut Environment,
        args: Vec<LoxObject>,
    ) -> LoxObject {
        exec_env.in_new_local_scope(|e| {
            for (i, param) in self.params.iter().enumerate() {
                e.define(&param.lexeme, args[i].clone());
            }

            // Execute each statement in the body of the function
            // If one of them returns something (return stmt),
            // break early.
            let mut return_val = LoxObject::Nil;
            for stmt in self.body.clone().into_iter() {
                if let Some(val) = interpreter.execute(stmt, e) {
                    return_val = val;
                    break;
                }
            }

            return_val
        })
    }
}

impl std::fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}
