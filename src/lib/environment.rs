use std::collections::HashMap;

use crate::{object::LoxObject, scanner::Token, error::{RuntimeResult, RuntimeError}};

/// The environment is responsible for the memory of the program. Each instance of the interpreter gets 
/// it's own environment for storing variables etc.
pub struct Environment {

    /// Represents the global variables in the program
    variables: HashMap<String, LoxObject>,
}

impl Environment {

    /// Construct an empty Environment
    pub fn new() -> Self { 
        Self {
            variables: HashMap::new()
        }
    }

    /// Define a variable in the environment
    pub fn define(&mut self, name: &str, value: LoxObject) {
        let _ = self.variables.insert(name.to_owned(), value);
    }

    /// Retrieve a variable from the environment
    pub fn get(&self, name: Token) -> RuntimeResult<LoxObject> {
        match self.variables.get(&name.lexeme) {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError::new(name.clone(), &format!("Undefined variable '{}'", &name.lexeme))),
        }
    }
}