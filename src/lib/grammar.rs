use crate::scanner::Token;

/// Represents the grammar for expressions in Lox. The parse construct these from a list of tokens,
/// and the interpreter evaluates them.
#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Token),
    Unary(Token, Box<Expr>),
    Variable(Token),

    /// Takes the name of the variable (as a token) and the expression to evaluate and assign
    Assignment(Token, Box<Expr>),
}

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
    Print(Expr),

    /// Represents block scope { ... }
    Block(Vec<Stmt>),
}
