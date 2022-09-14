use crate::token::Token;

/// Represents the grammar for expressions in Lox.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
    Variable(VariableExpr),
    Assignment(AssignmentExpr),
    Logical(BinaryExpr),
    Call(CallExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub lhs: Box<Expr>,
    pub operator: Token,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub operator: Token,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupingExpr {
    /// The expression inside the enclosing parentheses.
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralExpr {
    pub token: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableExpr {
    pub name: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    pub variable: Token,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub closing_paren: Token,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VariableDeclaration(VariableDeclarationStmt),
    Expression(ExpressionStmt),
    Print(PrintStmt),
    While(WhileStmt),
    FunctionDeclaration(FunctionDeclarationStmt),
    Block(BlockStmt),
    If(IfStmt),
    Return(ReturnStmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt { 
    pub condition: Expr,
    pub body: Box<Stmt>
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarationStmt {
    pub name: Token,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclarationStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStmt {
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrintStmt {
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub return_keyword: Token,
    pub value: Option<Expr>,
}
