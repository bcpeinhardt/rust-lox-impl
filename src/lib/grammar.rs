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

/// Represents a Binary Expression.
/// (Two expressions with an operator in the middle)
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub lhs: Box<Expr>,
    pub operator: Token,
    pub rhs: Box<Expr>,
}

/// Represents a Unary Expression.
/// (An operator on the left and an expression to the right, i.e !some_func() or -7)
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub operator: Token,
    pub rhs: Box<Expr>,
}

/// Represents an expression enclosed in parentheses.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupingExpr {
    pub expr: Box<Expr>,
}

/// Represents a literal value, like a number or string.
#[derive(Debug, Clone, PartialEq)]
pub struct LiteralExpr {
    pub token: Token,
}

/// Represents a single variable.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableExpr {
    pub name: Token
}

/// Represents variable assignment
/// Note. Variable assignment is an expression, not a statement.
/// Thus the expression `name = "Ben"` actually evaluates
/// to the string "Ben"
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    pub variable: Token,
    pub expr: Box<Expr>,
}

/// Represents a function call (or anything callable like a method)
/// For example: `clock()`
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub closing_paren: Token,
    pub args: Vec<Expr>,
}

/// Represents the grammar for statements in Lox.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VariableDeclaration(VariableDeclarationStmt),
    Expression(ExpressionStmt),
    While(WhileStmt),
    FunctionDeclaration(FunctionDeclarationStmt),
    Block(BlockStmt),
    If(IfStmt),
    Return(ReturnStmt),
}

/// Represents a while loop.
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

/// Represents variable declaration
/// `var a = true;`
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarationStmt {
    pub name: Token,
    pub initializer: Option<Expr>,
}

/// Represents a function definition.
/// `fun show_name() { print "Ben"; }`
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclarationStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

/// Represents an expression statement (an expression followed by a semi colon).
/// The most common of these is a single function call
/// `doTheThing();`
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStmt {
    pub expr: Expr,
}

/// Represents some code between braces
/// `{ ... some code ... }`
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    pub body: Vec<Stmt>,
}

/// Represents an if statement.
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

/// Represents a return statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub return_keyword: Token,
    pub value: Option<Expr>,
}
