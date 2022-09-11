use std::collections::HashMap;

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorCtx},
    interpreter::RuntimeResult,
    object::LoxObject,
    token::Token,
};

/// The environment is responsible for the memory of the program. Each instance of the interpreter gets
/// it's own environment for storing variables etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    /// Represents the global variables in the program
    variables: HashMap<String, LoxObject>,

    /// A reference to a parent environment, if there is one.
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    /// Construct an empty Environment
    pub fn new(enclosing_env: Option<Environment>) -> Self {
        // Box up the enclosing environment
        let enclosing = enclosing_env.map(|env| Box::new(env));

        Self {
            variables: HashMap::new(),
            enclosing,
        }
    }

    /// Define a variable in the environment
    pub fn define(&mut self, name: &str, value: LoxObject) {
        let _ = self.variables.insert(name.to_owned(), value);
    }

    /// Reassign the value of a variable in the environment
    pub fn assign(&mut self, name: Token, value: LoxObject) -> RuntimeResult<()> {
        if self.variables.contains_key(&name.lexeme) {
            self.variables.insert(name.lexeme, value);
            Ok(())
        } else if let Some(ref mut env) = self.enclosing {
            env.assign(name, value)
        } else {
            Err(RuntimeError::WithMsg(
                RuntimeErrorCtx {
                    token: name.clone(),
                },
                format!("Undefined variable {}", name.lexeme),
            ))
        }
    }

    /// Retrieve a variable from the environment
    pub fn get(&self, name: Token) -> RuntimeResult<LoxObject> {
        match self.variables.get(&name.lexeme) {
            Some(v) => Ok(v.clone()),
            None => {
                if let Some(ref env) = self.enclosing {
                    env.get(name)
                } else {
                    Err(RuntimeError::WithMsg(
                        RuntimeErrorCtx {
                            token: name.clone(),
                        },
                        format!("Undefined variable '{}'", &name.lexeme),
                    ))
                }
            }
        }
    }
}
