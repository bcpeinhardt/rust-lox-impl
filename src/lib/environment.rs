use std::collections::{HashMap, LinkedList};

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorCtx},
    interpreter::RuntimeResult,
    object::LoxObject,
    token::Token,
};

pub type Scope = HashMap<String, LoxObject>;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    local: LinkedList<Scope>,
    global: Scope
}

impl Environment {
    /// Construct an empty Environment
    pub fn new() -> Self {
        let global = HashMap::new();
        let mut local = LinkedList::new();
        local.push_back(HashMap::new());
        
        Self {
            local,
            global
        }
    }

    pub fn from_maps(local_scope: Scope, global: Scope) -> Self { 
        let mut local = LinkedList::new();
        local.push_back(local_scope);
        Self {
            local,
            global
        }
    }

    pub fn global(&self) -> Scope {
        self.global.clone()
    }

    pub fn all_local(&self) -> LinkedList<Scope> {
        self.local.clone()
    }

    pub fn most_local(&self) -> Scope { 
        self.local.clone().pop_back().unwrap_or(HashMap::new())
    }

    pub fn pop_most_local(&mut self) -> Scope {
        self.local.pop_back().unwrap_or(HashMap::new())
    }

    pub fn append_to_local(&mut self, scope: Scope) {
        self.local.push_back(scope);
    }

    pub fn append_empty_layer_to_local(&mut self) {
        self.local.push_back(HashMap::new());
    }

    /// Define a variable in the environment
    pub fn define(&mut self, name: &str, value: LoxObject) {
        self.local.back_mut().map(|hm| hm.insert(name.to_owned(), value));
    }

    /// Define a variable in the top level scope of the environment
    pub fn define_global(&mut self, name: &str, value: LoxObject) {
        self.global.insert(name.to_owned(), value);
    }

    fn try_assign_local(&mut self, name: Token, value: LoxObject) -> bool {
        let mut found_var = false;
        for hm in self.local.iter_mut().rev() {
            if hm.contains_key(&name.lexeme) {
                hm.insert(name.lexeme.clone(), value);
                found_var = true;
                break;
            }
        }
        found_var
    }

    fn try_assign_global(&mut self, name: Token, value: LoxObject) -> bool { 
        if self.global.contains_key(&name.lexeme) {
            self.global.insert(name.lexeme.clone(), value);
            true
        } else {
            false
        }
    }

    /// Reassign the value of a variable in the environment
    pub fn assign(&mut self, name: Token, value: LoxObject) -> RuntimeResult<()> {

        // Search for the variable in local. Then search global. Then error.
        if self.try_assign_local(name.clone(), value.clone()) || self.try_assign_global(name.clone(), value.clone()) { 
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

    fn try_get_local(&self, name: Token) -> Option<LoxObject> {
        let mut val = None;
        for hm in self.local.iter().rev() {
            if hm.contains_key(&name.lexeme) {
                val = hm.get(&name.lexeme).map(|obj| obj.clone());
                break;
            }
        }
        val
    }

    fn try_get_global(&self, name: Token) -> Option<LoxObject> {
        self.global.get(&name.lexeme).map(|obj| obj.clone())
    }

    /// Retrieve a variable from the environment
    pub fn get(&self, name: Token) -> RuntimeResult<LoxObject> {

        if let Some(obj) = self.try_get_local(name.clone()) { 
            Ok(obj)
        } else if let Some(obj) = self.try_get_global(name.clone()) { 
            Ok(obj)
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
        write!(f, "Global: {:#?}\nLocal: {:#?}\n", self.global, self.local.clone().into_iter().map(|hm| {
            hm.into_iter().filter(|(_, v)| {
                if let LoxObject::Function(_)  = v {
                    false
                } else {
                    true
                }
            }).collect::<HashMap<String, LoxObject>>()
        }).collect::<LinkedList<HashMap<String, LoxObject>>>())
    }
}
