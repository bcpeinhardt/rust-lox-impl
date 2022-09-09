use crate::{scanner::Token, expr::Expr};

/// Similarly to the Expr enum, rather than using a macro to generate classes for each type
/// of statement, we will simply use an enum. Yay Rust.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Contains the token for the name and the expression for the initializer
    /// (None value indicates uninitialized variable)
    VarDecl(Token, Option<Expr>),

    /// An expression statement is simply an expression terminated with a semi colon
    Expression(Expr),

    /// In Lox, the "print" is a keyword so we have specific print statments. Like 
    /// an expression statement but with the keyword "print" in front.
    Print(Expr)
}