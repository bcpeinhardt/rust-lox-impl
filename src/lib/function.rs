use uuid::Uuid;

use crate::{
    callable::LoxCallable,
    environment::{Environment, self, Scope},
       grammar::{BlockStmt, FunctionDeclarationStmt, Stmt},
    interpreter::Interpreter,
    object::LoxObject,
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>
}

impl LoxFunction {
    /// Construct a function object from the function declaration statement parsed by the parser.
    pub fn from(
        FunctionDeclarationStmt { name, params, body }: FunctionDeclarationStmt
    ) -> Self {
        Self {
            name,
            params,
            body
        }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&mut self, interpreter: &mut Interpreter, args: Vec<LoxObject>) -> LoxObject {

        // Define each argument passed into the function in their own localer scope.
        interpreter.environment.append_empty_layer_to_local();
        for (i, param) in self.params.iter().enumerate() {
            interpreter.environment.define(&param.lexeme, args[i].clone());
        }

        // Execute each statement in the body of the function
        // If one of them returns something (return stmt),
        // break early.
        let mut return_val = LoxObject::Nil;
        for stmt in self.body.clone().into_iter() {
            if let Some(val) = interpreter.execute(stmt) {
                return_val = val;
                break;
            }
        }

        // Remove the local variable scope layer
        interpreter.environment.pop_most_local();
        
        // Return the return value of the function.
        return_val
    }
}

impl std::fmt::Display for LoxFunction { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.lexeme)
    }
}
