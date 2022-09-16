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

/// Built in function clock, used for benchmarking inside a lox script
#[derive(Clone, PartialEq, Debug)]
pub struct Clock {}

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0usize
    }

    fn call(&mut self, _: &mut Interpreter, _: &mut Environment, _: Vec<LoxObject>) -> LoxObject {
        LoxObject::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64(),
        )
    }
}

/// Built in function print_env, for printing out the different memory scopes
/// and variables at a given point in a lox script. Useful for debugging in a lox script.
#[derive(Debug, Clone, PartialEq)]
pub struct PrintEnv {}

impl LoxCallable for PrintEnv {
    fn arity(&self) -> usize {
        0usize
    }

    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        env: &mut Environment,
        _: Vec<LoxObject>,
    ) -> LoxObject {
        println!("{}", env);
        LoxObject::Nil
    }
}
