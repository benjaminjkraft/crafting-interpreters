use crate::object;
use crate::scanner;
use derive_more::From;

#[derive(Debug)]
pub struct Program<'a> {
    pub stmts: Vec<Stmt<'a>>,
}

#[derive(Debug, From)]
pub enum Expr<'a> {
    Assign(AssignExpr<'a>),
    Binary(BinaryExpr<'a>),
    Grouping(GroupingExpr<'a>),
    Literal(LiteralExpr),
    Logical(LogicalExpr<'a>),
    Unary(UnaryExpr<'a>),
    Variable(VariableExpr<'a>),
}

#[derive(Debug, From)]
pub enum Stmt<'a> {
    Block(BlockStmt<'a>),
    Expr(ExprStmt<'a>),
    If(IfStmt<'a>),
    Print(PrintStmt<'a>),
    Var(VarStmt<'a>),
    While(WhileStmt<'a>),
}

#[derive(Debug)]
pub struct AssignExpr<'a> {
    pub name: scanner::Token<'a>,
    pub value: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct BinaryExpr<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: scanner::Token<'a>,
    pub right: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct GroupingExpr<'a> {
    pub expr: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct LiteralExpr {
    pub value: object::Object,
}

#[derive(Debug)]
pub struct LogicalExpr<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: scanner::Token<'a>,
    pub right: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct UnaryExpr<'a> {
    pub operator: scanner::Token<'a>,
    pub right: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct VariableExpr<'a> {
    pub name: scanner::Token<'a>,
}

#[derive(Debug)]
pub struct BlockStmt<'a> {
    pub stmts: Vec<Stmt<'a>>,
}

#[derive(Debug)]
pub struct ExprStmt<'a> {
    pub expr: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct IfStmt<'a> {
    pub condition: Box<Expr<'a>>,
    pub then_: Box<Stmt<'a>>,
    pub else_: Option<Box<Stmt<'a>>>,
}

#[derive(Debug)]
pub struct PrintStmt<'a> {
    pub expr: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct VarStmt<'a> {
    pub name: scanner::Token<'a>,
    pub initializer: Option<Box<Expr<'a>>>,
}

#[derive(Debug)]
pub struct WhileStmt<'a> {
    pub condition: Box<Expr<'a>>,
    pub body: Box<Stmt<'a>>,
}
