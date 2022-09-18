use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    callable::LoxCallable, environment::Environment, interpreter::Interpreter, object::LoxObject,
};

/// Built in function clock, used for benchmarking inside a lox script
#[derive(Clone, PartialEq, Debug)]
pub struct Clock {}

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0usize
    }

    fn call(&self, _: &mut Interpreter, _: &mut Environment, _: Vec<LoxObject>) -> LoxObject {
        LoxObject::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64(),
        )
    }
}

impl std::fmt::Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn cllock>")
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
        &self,
        _interpreter: &mut Interpreter,
        env: &mut Environment,
        _: Vec<LoxObject>,
    ) -> LoxObject {
        println!("{}", env);
        LoxObject::Nil
    }
}

impl std::fmt::Display for PrintEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn print_env>")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Print {}

impl LoxCallable for Print {
    fn arity(&self) -> usize {
        1usize
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _env: &mut Environment,
        args: Vec<LoxObject>,
    ) -> LoxObject {
        println!("{}", args[0]);
        LoxObject::Nil
    }
}

impl std::fmt::Display for Print {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn print>")
    }
}
