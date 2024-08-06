use crate::object;
use crate::scanner;
use derive_more::From;

#[derive(Debug)]
pub struct Program<'src> {
    pub stmts: Vec<Stmt<'src>>,
}

#[derive(Debug, From)]
pub enum Expr<'src> {
    Assign(AssignExpr<'src>),
    Binary(BinaryExpr<'src>),
    Call(CallExpr<'src>),
    Get(GetExpr<'src>),
    Grouping(GroupingExpr<'src>),
    Literal(LiteralExpr),
    Logical(LogicalExpr<'src>),
    Set(SetExpr<'src>),
    Super(SuperExpr<'src>),
    This(ThisExpr<'src>),
    Unary(UnaryExpr<'src>),
    Variable(VariableExpr<'src>),
}

#[derive(Debug, From)]
pub enum Stmt<'src> {
    Block(BlockStmt<'src>),
    Class(ClassStmt<'src>),
    Expr(ExprStmt<'src>),
    Function(FunctionStmt<'src>),
    If(IfStmt<'src>),
    Print(PrintStmt<'src>),
    Return(ReturnStmt<'src>),
    Var(VarStmt<'src>),
    While(WhileStmt<'src>),
}

#[derive(Debug)]
pub struct AssignExpr<'src> {
    pub name: scanner::Token<'src>,
    pub value: Box<Expr<'src>>,
    pub resolved_depth: Option<usize>,
}

#[derive(Debug)]
pub struct BinaryExpr<'src> {
    pub left: Box<Expr<'src>>,
    pub operator: scanner::Token<'src>,
    pub right: Box<Expr<'src>>,
}

#[derive(Debug)]
pub struct CallExpr<'src> {
    pub callee: Box<Expr<'src>>,
    pub paren: scanner::Token<'src>,
    pub arguments: Vec<Expr<'src>>,
}

#[derive(Debug)]
pub struct GetExpr<'src> {
    pub object: Box<Expr<'src>>,
    pub name: scanner::Token<'src>,
}

#[derive(Debug)]
pub struct GroupingExpr<'src> {
    pub expr: Box<Expr<'src>>,
}

#[derive(Debug)]
pub struct LiteralExpr {
    pub value: object::Literal,
}

#[derive(Debug)]
pub struct LogicalExpr<'src> {
    pub left: Box<Expr<'src>>,
    pub operator: scanner::Token<'src>,
    pub right: Box<Expr<'src>>,
}

#[derive(Debug)]
pub struct SetExpr<'src> {
    pub object: Box<Expr<'src>>,
    pub name: scanner::Token<'src>,
    pub value: Box<Expr<'src>>,
}

#[derive(Debug)]
pub struct SuperExpr<'src> {
    pub keyword: scanner::Token<'src>,
    pub method: scanner::Token<'src>,
    pub resolved_depth: Option<usize>,
}

#[derive(Debug)]
pub struct ThisExpr<'src> {
    pub keyword: scanner::Token<'src>,
    pub resolved_depth: Option<usize>,
}

#[derive(Debug)]
pub struct UnaryExpr<'src> {
    pub operator: scanner::Token<'src>,
    pub right: Box<Expr<'src>>,
}

#[derive(Debug)]
pub struct VariableExpr<'src> {
    pub name: scanner::Token<'src>,
    pub resolved_depth: Option<usize>,
}

#[derive(Debug)]
pub struct BlockStmt<'src> {
    pub stmts: Vec<Stmt<'src>>,
}

#[derive(Debug)]
pub struct ClassStmt<'src> {
    pub name: scanner::Token<'src>,
    pub superclass: Option<Box<VariableExpr<'src>>>,
    pub methods: Vec<FunctionStmt<'src>>,
}

#[derive(Debug)]
pub struct ExprStmt<'src> {
    pub expr: Box<Expr<'src>>,
}

#[derive(Debug)]
pub struct FunctionStmt<'src> {
    pub name: scanner::Token<'src>,
    pub parameters: Vec<scanner::Token<'src>>,
    pub body: Vec<Stmt<'src>>,
}

#[derive(Debug)]
pub struct IfStmt<'src> {
    pub condition: Box<Expr<'src>>,
    pub then_: Box<Stmt<'src>>,
    pub else_: Option<Box<Stmt<'src>>>,
}

#[derive(Debug)]
pub struct PrintStmt<'src> {
    pub expr: Box<Expr<'src>>,
}

#[derive(Debug)]
pub struct ReturnStmt<'src> {
    pub keyword: scanner::Token<'src>,
    pub value: Option<Box<Expr<'src>>>,
}

#[derive(Debug)]
pub struct VarStmt<'src> {
    pub name: scanner::Token<'src>,
    pub initializer: Option<Box<Expr<'src>>>,
}

#[derive(Debug)]
pub struct WhileStmt<'src> {
    pub condition: Box<Expr<'src>>,
    pub body: Box<Stmt<'src>>,
}
