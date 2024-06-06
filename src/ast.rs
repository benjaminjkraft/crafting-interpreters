use crate::object;
use crate::scanner;
use derive_more::From;

#[derive(From)]
pub enum Expr<'a> {
    Binary(BinaryExpr<'a>),
    Grouping(GroupingExpr<'a>),
    Literal(LiteralExpr),
    Unary(UnaryExpr<'a>),
}

impl<'a, R> Visited<'a, R> for Expr<'a> {
    fn accept(&self, visitor: impl Visitor<'a, R>) -> R {
        match self {
            Expr::Binary(e) => e.accept(visitor),
            Expr::Grouping(e) => e.accept(visitor),
            Expr::Literal(e) => e.accept(visitor),
            Expr::Unary(e) => e.accept(visitor),
        }
    }
}

pub struct BinaryExpr<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: scanner::Token<'a>,
    pub right: Box<Expr<'a>>,
}

impl<'a, R> Visited<'a, R> for BinaryExpr<'a> {
    fn accept(&self, visitor: impl Visitor<'a, R>) -> R {
        visitor.visit_binary_expr(&self)
    }
}

pub struct GroupingExpr<'a> {
    pub expr: Box<Expr<'a>>,
}

impl<'a, R> Visited<'a, R> for GroupingExpr<'a> {
    fn accept(&self, visitor: impl Visitor<'a, R>) -> R {
        visitor.visit_grouping_expr(&self)
    }
}

pub struct LiteralExpr {
    pub value: object::Object,
}

impl<'a, R> Visited<'a, R> for LiteralExpr {
    fn accept(&self, visitor: impl Visitor<'a, R>) -> R {
        visitor.visit_literal_expr(&self)
    }
}

pub struct UnaryExpr<'a> {
    pub operator: scanner::Token<'a>,
    pub right: Box<Expr<'a>>,
}

impl<'a, R> Visited<'a, R> for UnaryExpr<'a> {
    fn accept(&self, visitor: impl Visitor<'a, R>) -> R {
        visitor.visit_unary_expr(&self)
    }
}

#[allow(unused_variables)]
pub trait Visited<'a, R> {
    fn accept(&self, visitor: impl Visitor<'a, R>) -> R;
}

#[allow(unused_variables)]
pub trait Visitor<'a, R> {
    fn visit_binary_expr(&self, node: &BinaryExpr<'a>) -> R;
    fn visit_grouping_expr(&self, node: &GroupingExpr<'a>) -> R;
    fn visit_literal_expr(&self, node: &LiteralExpr) -> R;
    fn visit_unary_expr(&self, node: &UnaryExpr<'a>) -> R;
}
