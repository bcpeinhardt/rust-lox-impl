use std::collections::{HashMap, LinkedList};

use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorCtx},
    interpreter::RuntimeResult,
    object::LoxObject,
    token::Token, callable::{Clock, PrintEnv},
};

/// Represents a single scope of LoxObjects
#[derive(Debug, Clone, PartialEq)]
pub struct Scope(HashMap<String, LoxObject>);

impl Scope {

    pub fn new() -> Self {
        Scope(HashMap::new())
    }

    pub fn define(&mut self, name: &str, value: LoxObject) {
        self.0.insert(name.to_owned(), value);
    }
}

/// Represents a layering of scopes
#[derive(Debug, Clone, PartialEq)]
pub struct MultiScope(LinkedList<Scope>);

impl MultiScope {

    /// Creates a new multi scope with one layer.
    pub fn new() -> Self {
        let mut list = LinkedList::new();
        list.push_back(Scope::new());
        Self(list)
    }

    /// Removes and returns the innermost scope layer. Errors
    /// if trying to pop the only scope
    pub fn pop_innermost_scope(&mut self) -> Result<Scope, ()> {
        if self.0.len() == 1 {
            Err(())
        } else {
            // We know this unwrap is safe because the len is at least 2
            Ok(self.0.pop_back().unwrap())
        }
    }

    /// Returns a copy of the innermost scope layer.
    pub fn clone_innermost_scope(&self) -> Scope {

        // We know this unwrap is safe because this structure
        // will always contain at least one Scope.
        self.0
            .back()
            .unwrap()
            .clone()
    }

    /// Add a scope layer to the environment
    pub fn push_as_innermost_scope(&mut self, scope: Scope) {
        self.0.push_back(scope);
    }

    /// Defines a variable in the innermost scope. 
    pub fn define(&mut self, name: &str, value: LoxObject) {

        // We know this unwrap is safe because this struct will always contain
        // at least one inner scope.
        self.0.back_mut().unwrap().define(name, value);
    }
}

/// Represents a program execution environment.
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub local: Option<MultiScope>,
    pub global: Scope,
}

impl Environment {

    /// Construct an empty Environment
    pub fn new() -> Self {
        let global = Scope::new();
        let local = None;

        let mut new_env = Self { local, global };

        // Define the builtin functions
        new_env
            .global
            .define("clock", LoxObject::Clock(Clock {}));
        new_env
            .global
            .define("print_env", LoxObject::PrintEnv(PrintEnv {}));

        new_env
    }

    pub fn add_scope_layer(&mut self) {
        if let Some(ref mut local_scope) = self.local {
            local_scope.push_as_innermost_scope(Scope::new());
        } else {
            self.local = Some(MultiScope::new());
        }
    }

    pub fn remove_scope_layer(&mut self) -> Result<(), ()> {
        if let Some(ref mut local_scope) = self.local {
            match local_scope.pop_innermost_scope() {
                Ok(_) => {},
                Err(_) => {
                    self.local = None;
                },
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn in_new_local_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.add_scope_layer();
        let res = f(self);
        // This unwrap is safe because we just added a scope layer.
        self.remove_scope_layer().unwrap();
        res
    }

    /// Define a variable in the most local scope. If no local scope is available,
    /// defines in the global scope.
    pub fn define(&mut self, name: &str, value: LoxObject) {
        if let Some(ref mut local_scope) = self.local {
            local_scope.define(name, value.clone());
        } else {
            self.global.define(name, value);
        }
    }

    fn try_assign_local(&mut self, name: Token, value: LoxObject) -> bool {
        let mut found_var = false;
        if let Some(ref mut local_scope) = self.local {
            for scope in local_scope.0.iter_mut().rev() {
                if scope.0.contains_key(&name.lexeme) {
                    scope.0.insert(name.lexeme.clone(), value);
                    found_var = true;
                    break;
                }
            }
        }
        found_var
    }

    fn try_assign_global(&mut self, name: Token, value: LoxObject) -> bool {
        if self.global.0.contains_key(&name.lexeme) {
            self.global.0.insert(name.lexeme.clone(), value);
            true
        } else {
            false
        }
    }

    /// Reassign the value of a variable in the environment
    pub fn assign(&mut self, name: Token, value: LoxObject) -> RuntimeResult<()> {
        // Search for the variable in local. Then search global. Then error.
        if self.try_assign_local(name.clone(), value.clone())
            || self.try_assign_global(name.clone(), value.clone())
        {
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
        if let Some(ref local_scope) = self.local {
            for scope in local_scope.0.iter().rev() {
                if scope.0.contains_key(&name.lexeme) {
                    val = scope.0.get(&name.lexeme).map(|obj| obj.clone());
                    break;
                }
            }
        }
        val
    }

    fn try_get_global(&self, name: Token) -> Option<LoxObject> {
        self.global.0.get(&name.lexeme).map(|obj| obj.clone())
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
        if let Some(ref local_scope) = self.local {
            write!(f, "Global: {}\nLocal: {}\n", self.global, local_scope)
        } else {
            write!(f, "Global: {}\n", self.global)
        }
    }
}

impl std::fmt::Display for MultiScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::from("[\n");
        for scope in self.0.iter() {
            buffer.push_str(&format!("{},\n", scope));
        }
        buffer.push_str("]");
        write!(f, "{}", buffer)
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::from("{\n");
        for (key, value) in self.0.iter() {
            buffer.push_str(&format!("   {} = {},\n", key, value));
        }
        buffer.push_str("}");
        write!(f, "{}", buffer)
    }
}
