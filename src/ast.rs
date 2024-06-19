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
pub struct PrintStmt<'a> {
    pub expr: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct VarStmt<'a> {
    pub name: scanner::Token<'a>,
    pub initializer: Option<Box<Expr<'a>>>,
}

macro_rules! visitor_impl {
    ( $type:ident < $lt:lifetime >, $method:ident  ) => {
        impl<$lt, R, V: Visitor<$lt, R>> Visited<$lt, R, V> for $type<$lt> {
            fn accept(&self, visitor: &mut V) -> R {
                visitor.$method(&self)
            }
        }
    };
    ( $type:ident, $method:ident  ) => {
        impl<'a, R, V: Visitor<'a, R>> Visited<'a, R, V> for $type {
            fn accept(&self, visitor: &mut V) -> R {
                visitor.$method(&self)
            }
        }
    };
}

visitor_impl!(Program<'a>, visit_program);
visitor_impl!(AssignExpr<'a>, visit_assign_expr);
visitor_impl!(BinaryExpr<'a>, visit_binary_expr);
visitor_impl!(GroupingExpr<'a>, visit_grouping_expr);
visitor_impl!(LiteralExpr, visit_literal_expr);
visitor_impl!(UnaryExpr<'a>, visit_unary_expr);
visitor_impl!(VariableExpr<'a>, visit_variable_expr);
visitor_impl!(BlockStmt<'a>, visit_block_stmt);
visitor_impl!(ExprStmt<'a>, visit_expr_stmt);
visitor_impl!(PrintStmt<'a>, visit_print_stmt);
visitor_impl!(VarStmt<'a>, visit_var_stmt);

impl<'a, R, V: Visitor<'a, R>> Visited<'a, R, V> for Expr<'a> {
    fn accept(&self, visitor: &mut V) -> R {
        match self {
            Expr::Assign(e) => e.accept(visitor),
            Expr::Binary(e) => e.accept(visitor),
            Expr::Grouping(e) => e.accept(visitor),
            Expr::Literal(e) => e.accept(visitor),
            Expr::Unary(e) => e.accept(visitor),
            Expr::Variable(e) => e.accept(visitor),
        }
    }
}

impl<'a, R, V: Visitor<'a, R>> Visited<'a, R, V> for Stmt<'a> {
    fn accept(&self, visitor: &mut V) -> R {
        match self {
            Stmt::Block(e) => e.accept(visitor),
            Stmt::Expr(e) => e.accept(visitor),
            Stmt::Print(e) => e.accept(visitor),
            Stmt::Var(e) => e.accept(visitor),
        }
    }
}

#[allow(unused_variables)]
pub trait Visited<'a, R, V: Visitor<'a, R>> {
    fn accept(&self, visitor: &mut V) -> R;
}

#[allow(unused_variables)]
pub trait Visitor<'a, R> {
    fn visit_program(&mut self, node: &Program<'a>) -> R;
    fn visit_assign_expr(&mut self, node: &AssignExpr<'a>) -> R;
    fn visit_binary_expr(&mut self, node: &BinaryExpr<'a>) -> R;
    fn visit_grouping_expr(&mut self, node: &GroupingExpr<'a>) -> R;
    fn visit_literal_expr(&mut self, node: &LiteralExpr) -> R;
    fn visit_unary_expr(&mut self, node: &UnaryExpr<'a>) -> R;
    fn visit_variable_expr(&mut self, node: &VariableExpr<'a>) -> R;
    fn visit_block_stmt(&mut self, node: &BlockStmt<'a>) -> R;
    fn visit_expr_stmt(&mut self, node: &ExprStmt<'a>) -> R;
    fn visit_print_stmt(&mut self, node: &PrintStmt<'a>) -> R;
    fn visit_var_stmt(&mut self, node: &VarStmt<'a>) -> R;
}
