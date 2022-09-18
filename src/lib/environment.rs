use std::collections::{HashMap, LinkedList};

use crate::{
    builtin_functions::{Clock, Print, PrintEnv},
    error::runtime_error::{RuntimeError, RuntimeErrorCtx},
    interpreter::RuntimeResult,
    object::LoxObject,
    token::Token,
};

/// Represents a single scope of LoxObjects
#[derive(Clone, PartialEq)]
pub struct Scope(HashMap<String, LoxObject>);

impl Scope {
    /// Creates a new Scope
    pub fn new() -> Self {
        Scope(HashMap::new())
    }

    /// Sets the value for a variable in the given scope. Optionally returns the old obj
    /// if the variable was previously defined.
    pub fn define(&mut self, name: &str, value: LoxObject) -> Option<LoxObject> {
        self.0.insert(name.to_owned(), value)
    }

    /// If a variable is already defined in the scope, reassigns
    /// it and returns the old value. Returns `None` if the variable is not
    /// yet defined.
    pub fn assign(&mut self, name: &str, value: LoxObject) -> Option<LoxObject> {
        if self.0.contains_key(name) {
            self.0.insert(name.to_owned(), value)
        } else {
            None
        }
    }

    /// Tries to retrieve a variable from the scope
    pub fn get(&self, name: &str) -> Option<LoxObject> {
        self.0.get(name).map(|obj| obj.clone())
    }
}

/// Represents a layering of scopes
#[derive(Clone, PartialEq)]
pub struct MultiScope(LinkedList<Scope>);

impl MultiScope {
    /// Creates a new multi scope with one layer.
    pub fn new() -> Self {
        let mut list = LinkedList::new();
        list.push_back(Scope::new());
        Self(list)
    }

    /// Bubbling up iterator methods. Iterates from inside out (local scope to outer scope)
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.0.iter().rev()
    }

    /// Bubbling up iterator methods. Iterates from inside out (local scope to outer scope)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Scope> {
        self.0.iter_mut().rev()
    }

    /// Bubbling up iterator methods. Iterates from inside out (local scope to outer scope)
    pub fn into_iter(self) -> impl Iterator<Item = Scope> {
        self.0.into_iter().rev()
    }

    /// Get a mutable reference to the innermost (most local) scope
    fn innermost_mut(&mut self) -> &mut Scope {
        // Unwrap is safe because inner list never has 0 elements
        self.iter_mut().next().unwrap()
    }

    /// Removes and returns the innermost scope layer.
    /// Important! If there is only one layer, will return None rather
    /// than popping. This is because of the sematic choice to
    /// consider an empty MultiScope an invalid state, and to
    /// represent the "local" member of an `Environment` as an
    /// `Option<MultiScope>` where MultiScope always has at least one scope
    /// to operate in. In this way, we escalate the information
    /// about whether or not we are operating in the local or global scope
    /// to the environment struct, rather than having to implement
    /// a ton of failable methods on multiscope.
    pub fn pop_innermost_scope(&mut self) -> Option<Scope> {
        if self.0.len() == 1 {
            None
        } else {
            self.0.pop_back()
        }
    }

    /// For retrieving the final layer of a MultiScope.
    /// Takes self because an empty MultiScope is an invalid state.
    /// Use with `pop_innermost_scope` to ensure there is only one layer to pop.
    /// # Example
    /// ```
    ///  use rust_lox_impl::environment::{MultiScope, Scope};
    ///
    /// let mut multi_scope = MultiScope::new();
    /// multi_scope.push_as_innermost_scope(Scope::new());
    ///
    /// let end_scope = if let Some(scope) = multi_scope.pop_innermost_scope() {
    ///     scope
    /// } else {
    ///     multi_scope.consume_final_layer()
    /// };
    /// ```
    pub fn consume_final_layer(mut self) -> Scope {
        // We know this unwrap is safe because always has at least one scope.
        self.0.pop_front().unwrap()
    }

    /// Add a scope layer to the environment
    pub fn push_as_innermost_scope(&mut self, scope: Scope) {
        self.0.push_back(scope);
    }

    /// Defines a variable in the innermost scope.
    pub fn define(&mut self, name: &str, value: LoxObject) {
        self.innermost_mut().define(name, value);
    }

    /// Tries to assign Lox Object to the variable in the closest scope. Returns
    /// the Some(old_obj) if successful or None if the variables isn't defined in any
    /// of the scopes.
    pub fn assign(&mut self, name: &str, value: LoxObject) -> Option<LoxObject> {
        let mut old_value = None;
        for scope in self.iter_mut() {
            if let Some(obj) = scope.assign(name, value.clone()) {
                old_value = Some(obj);
                break;
            }
        }
        old_value
    }

    /// Tries to get a variable from the closest scope it can. Returns
    /// None if the variable is not defined in any scope.
    pub fn get(&self, name: &str) -> Option<LoxObject> {
        for scope in self.iter() {
            if let Some(obj) = scope.get(name) {
                return Some(obj);
            }
        }
        None
    }
}

/// Represents a program execution environment.
#[derive(Clone, PartialEq)]
pub struct Environment {
    /// Represents any local scopes created during the execution of the program
    local: Option<MultiScope>,

    /// Represents the global scope of the program
    global: Scope,
}

impl Environment {
    /// Construct an new Environment. Contains only a global scope
    /// with builtin Lox functions defined.
    pub fn new() -> Self {
        let global = Scope::new();
        let local = None;

        let mut new_env = Self { local, global };

        // Define the builtin functions
        new_env
            .global
            .define("clock", LoxObject::Function(Box::new(Clock {})));
        new_env
            .global
            .define("print_env", LoxObject::Function(Box::new(PrintEnv {})));
        new_env
            .global
            .define("print", LoxObject::Function(Box::new(Print {})));

        new_env
    }

    /// Adds a layer to the local scope or creates a local scope if one
    /// does not exist yet
    fn add_scope_layer(&mut self) {
        if let Some(ref mut local_scope) = self.local {
            local_scope.push_as_innermost_scope(Scope::new());
        } else {
            self.local = Some(MultiScope::new());
        }
    }

    /// Removes a layer from the local scope.
    fn pop_scope_layer(&mut self) -> Option<Scope> {
        if let Some(ref mut local_scope) = self.local {
            if let Some(scope) = local_scope.pop_innermost_scope() {
                // There is a local scope with an extra layer to pop
                Some(scope)
            } else {
                // The local scope only has one layer, so we set it to None
                let res = Some(local_scope.clone().consume_final_layer());
                self.local = None;
                res
            }
        } else {
            // There is no local scope, so we return None;
            None
        }
    }

    /// Perform some operation inside an extra scope layer. Used for block stmts,
    /// functions, etc.
    pub fn in_new_local_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.add_scope_layer();
        let res = f(self);
        // This unwrap is safe because we just added a scope layer.
        self.pop_scope_layer();
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

    /// Reassign the value of a variable in the environment
    pub fn assign(&mut self, name: Token, value: LoxObject) -> RuntimeResult<()> {
        self.local
            .as_mut()
            .map(|local_scope| local_scope.assign(&name.lexeme, value.clone()))
            .flatten()
            .or(self.global.assign(&name.lexeme, value.clone()))
            .ok_or(RuntimeError::WithMsg(
                RuntimeErrorCtx {
                    token: name.clone(),
                },
                format!("Undefined variable {}", name.lexeme),
            ))
            .map(|_| ())
    }

    /// Retrieve a variable from the environment
    pub fn get(&self, name: Token) -> RuntimeResult<LoxObject> {
        self.local
            .as_ref()
            .map(|local_scope| local_scope.get(&name.lexeme))
            .flatten()
            .or(self.global.get(&name.lexeme))
            .ok_or(RuntimeError::WithMsg(
                RuntimeErrorCtx {
                    token: name.clone(),
                },
                format!("Undefined variable {}", name.lexeme),
            ))
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
