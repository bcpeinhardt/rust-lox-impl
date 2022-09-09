use crate::scanner::Token;

/// Represents the grammar for expressions in Lox. The parse construct these from a list of tokens,
/// and the interpreter evaluates them.
#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Token),
    Unary(Token, Box<Expr>),
    Variable(Token)
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Binary(lhs, token, rhs) => {
                write!(f, "({} {} {})", token.lexeme, lhs, rhs)
            }
            Expr::Grouping(expr) => {
                write!(f, "(group {})", expr)
            }
            Expr::Literal(token) => {
                write!(f, "{}", token.lexeme)
            }
            Expr::Unary(token, expr) => {
                write!(f, "({} {})", token.lexeme, expr)
            }
            Expr::Variable(token) => {
                write!(f, "{}", token.lexeme)
            },
        }
    }
}