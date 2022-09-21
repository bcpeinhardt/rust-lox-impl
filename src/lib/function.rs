use crate::{
    callable::LoxCallable,
    environment::{Environment, Scope},
    grammar::{FunctionDeclarationStmt, Stmt},
    interpreter::Interpreter,
    object::LoxObject,
    token::Token,
};

/// Represents a Lox Function Object
#[derive(Clone, PartialEq)]
pub struct LoxFunction {
    /// The token of the function name from the function declaration
    name: Token,

    /// The tokens from the function declaration describing the parameters
    /// of the function.
    params: Vec<Token>,

    /// The parsed list of statements from the body of the function declaration.
    body: Vec<Stmt>,

    /// This is a basically a place for the function to store private state between calls.
    /// Lox functions are closures, and when they are declared, if they reference 
    /// a variable from an enclosing scope, they should basically get a copy of that 
    /// variable as private state.
    /// 
    /// The book does an entire chapter implementing variable resolving and binding 
    /// as a separate pass. 
    state: Scope
}

impl LoxFunction {
    /// Construct a function object from the function declaration statement parsed by the parser.
    pub fn from(FunctionDeclarationStmt { name, params, body }: FunctionDeclarationStmt) -> Self {
        Self { name, params, body, state: Scope::new() }
    }
}

impl LoxCallable for LoxFunction {
    /// Returns the number of parameters the function expects
    fn arity(&self) -> usize {
        self.params.len()
    }

    /// Calls the function
    fn call(
        &self,
        interpreter: &mut Interpreter,
        exec_env: &mut Environment,
        args: Vec<LoxObject>,
    ) -> LoxObject {
        // In a new scope
        exec_env.in_new_local_scope(|e| {
            // Define all the arguments of the function as local
            // variables
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

            // Return the result of the function
            return_val
        })
    }
}

impl std::fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}
