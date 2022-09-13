use std::collections::{HashMap, LinkedList};

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorCtx},
    interpreter::RuntimeResult,
    object::LoxObject,
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Environment(LinkedList<HashMap<String, LoxObject>>);

impl Environment {
    /// Construct an empty Environment
    pub fn new() -> Self {
        let global = HashMap::new();
        let mut list = LinkedList::new();
        list.push_back(global);
        Self(list)
    }

    pub fn append_child(&mut self, env: &mut Environment) {
        self.0.append(&mut env.0);
    }

    pub fn append_empty_child(&mut self) {
        self.0.push_back(HashMap::new());
    }

    pub fn pop_child(&mut self) {
        self.0.pop_back();
    }

    /// Define a variable in the environment
    pub fn define(&mut self, name: &str, value: LoxObject) {
        self.0.back_mut().map(|hm| hm.insert(name.to_owned(), value));
    }

    /// Define a variable in the top level scope of the environment
    pub fn define_global(&mut self, name: &str, value: LoxObject) {
        self.0.front_mut().map(|hm| hm.insert(name.to_owned(), value));
    }

    /// Reassign the value of a variable in the environment
    pub fn assign(&mut self, name: Token, value: LoxObject) -> RuntimeResult<()> {

        let mut found_var = false;
        for hm in self.0.iter_mut().rev() {
            if hm.contains_key(&name.lexeme) {
                hm.insert(name.lexeme.clone(), value);
                found_var = true;
                break;
            }
        }

        if found_var {
            Ok(())
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

        let mut val = None;
        for hm in self.0.iter().rev() {
            if hm.contains_key(&name.lexeme) {
                val = hm.get(&name.lexeme).map(|obj| obj.clone());
                break;
            }
        }

        if val.is_some() {
            Ok(val.unwrap())
        } else {
            Err(RuntimeError::WithMsg(
                RuntimeErrorCtx {
                    token: name.clone(),
                },
                format!("Undefined variable {}", name.lexeme),
            ))
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}
