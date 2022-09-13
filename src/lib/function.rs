use crate::{
    callable::LoxCallable,
    environment::Environment,
    grammar::{FunctionDeclarationStmt, Stmt, BlockStmt},
    interpreter::Interpreter,
    object::LoxObject,
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Environment
}

impl LoxFunction {
    /// Construct a function object from the function declaration statement parsed by the parser.
    pub fn from(FunctionDeclarationStmt { name, params, body }: FunctionDeclarationStmt, closure: Environment) -> Self {
        Self { name, params, body, closure }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&mut self, interpreter: &mut Interpreter, args: Vec<LoxObject>) -> LoxObject {
        // Define the passed arguments in the global environment
        // for the function to execute in, then execute the body of the function
        // in said environment.
        let mut environment = Environment::new(Some(interpreter.environment.clone()));
        for (i, param) in self.params.iter().enumerate() {
            environment.define(&param.lexeme, args[i].clone());
        }
        let res = interpreter.execute_block(BlockStmt { body: self.body.clone() }, environment);
        res.unwrap_or(LoxObject::Nil)
    }
}

impl std::fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.lexeme)
    }
}
