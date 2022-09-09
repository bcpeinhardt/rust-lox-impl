/// The job of this enum is essentially to map Lox Objects to Rust types. It is our replacement
/// for the use of java.lang.Object in the Interpreter.
#[derive(Debug, Clone, PartialEq)]
pub enum LoxObject {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
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
