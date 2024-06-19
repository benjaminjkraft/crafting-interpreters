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

#[allow(unused_variables)]
pub trait Visitor<'a, RExpr, ROther> {
    fn visit_program(&mut self, node: &Program<'a>) -> ROther;
    fn visit_expr(&mut self, node: &Expr<'a>) -> RExpr {
        match node {
            Expr::Assign(n) => self.visit_assign_expr(n),
            Expr::Binary(n) => self.visit_binary_expr(n),
            Expr::Grouping(n) => self.visit_grouping_expr(n),
            Expr::Literal(n) => self.visit_literal_expr(n),
            Expr::Unary(n) => self.visit_unary_expr(n),
            Expr::Variable(n) => self.visit_variable_expr(n),
        }
    }
    fn visit_assign_expr(&mut self, node: &AssignExpr<'a>) -> RExpr;
    fn visit_binary_expr(&mut self, node: &BinaryExpr<'a>) -> RExpr;
    fn visit_grouping_expr(&mut self, node: &GroupingExpr<'a>) -> RExpr;
    fn visit_literal_expr(&mut self, node: &LiteralExpr) -> RExpr;
    fn visit_unary_expr(&mut self, node: &UnaryExpr<'a>) -> RExpr;
    fn visit_variable_expr(&mut self, node: &VariableExpr<'a>) -> RExpr;
    fn visit_stmt(&mut self, node: &Stmt<'a>) -> ROther {
        match node {
            Stmt::Block(n) => self.visit_block_stmt(n),
            Stmt::Expr(n) => self.visit_expr_stmt(n),
            Stmt::If(n) => self.visit_if_stmt(n),
            Stmt::Print(n) => self.visit_print_stmt(n),
            Stmt::Var(n) => self.visit_var_stmt(n),
        }
    }
    fn visit_block_stmt(&mut self, node: &BlockStmt<'a>) -> ROther;
    fn visit_expr_stmt(&mut self, node: &ExprStmt<'a>) -> ROther;
    fn visit_if_stmt(&mut self, node: &IfStmt<'a>) -> ROther;
    fn visit_print_stmt(&mut self, node: &PrintStmt<'a>) -> ROther;
    fn visit_var_stmt(&mut self, node: &VarStmt<'a>) -> ROther;
}
