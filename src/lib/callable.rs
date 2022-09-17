use dyn_clone::DynClone;

use crate::{environment::Environment, interpreter::Interpreter, object::LoxObject};

/// This trait is implemented on any Lox Structure that acts like a function
/// Requires Clone and Display.
pub trait LoxCallable: DynClone + std::fmt::Display {
    /// The number of parameters
    fn arity(&self) -> usize;

    /// Calls the thing and returns a Lox Object
    fn call(
        &self,
        interpreter: &mut Interpreter,
        env: &mut Environment,
        args: Vec<LoxObject>,
    ) -> LoxObject;
}
dyn_clone::clone_trait_object!(LoxCallable);
