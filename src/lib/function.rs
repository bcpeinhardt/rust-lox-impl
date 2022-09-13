use crate::{
    callable::LoxCallable,
    environment::Environment,
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
    closure: Environment,
}

impl LoxFunction {
    /// Construct a function object from the function declaration statement parsed by the parser.
    pub fn from(
        FunctionDeclarationStmt { name, params, body }: FunctionDeclarationStmt,
        closure: Environment,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&mut self, interpreter: &mut Interpreter, args: Vec<LoxObject>) -> LoxObject {

        let prev = interpreter.environment.clone();
        interpreter.environment.append_child(&mut self.closure);
        interpreter.environment.append_empty_child();

        // Define each argument passed into the function in the nearest local scope.
        for (i, param) in self.params.iter().enumerate() {
            interpreter.environment.define(&param.lexeme, args[i].clone());
        }

        let mut return_val = LoxObject::Nil;
        for stmt in self.body.clone().into_iter() {
            if let Some(val) = interpreter.execute(stmt) {
                return_val = val;
            }
        }
        interpreter.environment = prev;
        return_val
    }
}

impl std::fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.lexeme)
    }
}
