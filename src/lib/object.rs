use crate::callable::LoxCallable;

/// The job of this enum is essentially to map Lox Objects to Rust types. It is our replacement
/// for the use of java.lang.Object in the Interpreter.
#[derive(Clone)]
pub enum LoxObject {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
    Function(Box<dyn LoxCallable>),
}

impl PartialEq for LoxObject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::Boolean(l), Self::Boolean(r)) => l == r,

            // Functions are never equal, even if the code is
            // equivalent, functions enclose different environments, so semantically
            // in Lox they should never be the same.
            (Self::Function(_), Self::Function(_)) => false,
            _ => false,
        }
    }
}

impl LoxObject {
    /// Function casts a LoxObject to a bool
    pub fn is_truthy(&self) -> bool {
        match self {
            // Boolean is its own value
            LoxObject::Boolean(b) => *b,

            // Nil is False
            LoxObject::Nil => false,

            // Zero is false
            LoxObject::Number(n) => *n != 0f64,

            // Everything else is true
            _ => true,
        }
    }
}

/// Implemented this as a convenience. I'm willing to bet there's
/// a crate aimed at deriving these for all associated values,
/// like a sort of cast for enums.
impl TryFrom<LoxObject> for f64 {
    type Error = ();

    fn try_from(value: LoxObject) -> Result<Self, Self::Error> {
        if let LoxObject::Number(n) = value {
            Ok(n)
        } else {
            Err(())
        }
    }
}

impl std::fmt::Display for LoxObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoxObject::String(s) => {
                write!(f, "{}", s)
            }
            LoxObject::Number(n) => {
                write!(f, "{}", n)
            }
            LoxObject::Boolean(b) => {
                write!(f, "{}", b)
            }
            LoxObject::Nil => {
                write!(f, "nil")
            }
            LoxObject::Function(function) => {
                write!(f, "{}", function)
            }
        }
    }
}
