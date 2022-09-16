use std::time::{SystemTime, UNIX_EPOCH};

use crate::{environment::Environment, interpreter::Interpreter, object::LoxObject};

/// This trait is implemented on any Lox Structure that acts like a function
pub trait LoxCallable {
    /// The number of parameters
    fn arity(&self) -> usize;

    /// Calls the thing and returns a Lox Object
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        env: &mut Environment,
        args: Vec<LoxObject>,
    ) -> LoxObject;
}
