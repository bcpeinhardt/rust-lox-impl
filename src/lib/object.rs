use std::convert::TryFrom;

/// I tried to get by without something to replace Java's java.lang.Object, utilizing 
/// Rusts powerful enums to enhance scanner::TokenType and parser::Expr to not require it.
/// But I think that we're going to end up needing something for the interpreter. 
#[derive(Debug, Clone, PartialEq)]
pub enum LoxObject {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil
}

trait LoxObjectInner {}
impl LoxObjectInner for String {}
impl LoxObjectInner for f64 {}
impl LoxObjectInner for bool {}
impl LoxObjectInner for () {}

impl std::fmt::Display for LoxObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoxObject::String(s) => {
                write!(f, "{}", s)
            },
            LoxObject::Number(n) => {
                write!(f, "{}", n)
            },
            LoxObject::Boolean(b) => {
                write!(f, "{}", b)
            },
            LoxObject::Nil => {
                write!(f, "nil")
            },
        }
    }
}

impl LoxObject {
    pub fn is_truthy(&self) -> bool { 
        match self {
            LoxObject::Boolean(b) => *b,
            LoxObject::Nil => false,
            _ => true
        }
    }

    pub fn downcast(self) -> Box<impl LoxObjectInner> {
        match self {
            LoxObject::String(s) => Box::new(s),
            LoxObject::Number(n) => todo!(),
            LoxObject::Boolean(_) => todo!(),
            LoxObject::Nil => todo!(),
        }
    }
}