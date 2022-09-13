use std::time::{SystemTime, UNIX_EPOCH};

use crate::{interpreter::Interpreter, object::LoxObject};

pub trait LoxCallable {
    /// The number of parameters
    fn arity(&self) -> usize;

    /// Calls the thing and returns a Lox Object
    fn call(&mut self, interpreter: &mut Interpreter, args: Vec<LoxObject>) -> LoxObject;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Clock {}

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0usize
    }

    fn call(&mut self, _: &mut Interpreter, _: Vec<LoxObject>) -> LoxObject {
        LoxObject::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64(),
        )
    }
}
