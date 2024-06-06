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

pub struct BinaryExpr<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: scanner::Token<'a>,
    pub right: Box<Expr<'a>>,
}

pub struct GroupingExpr<'a> {
    pub expr: Box<Expr<'a>>,
}

pub struct LiteralExpr {
    pub value: object::Object,
}

pub struct UnaryExpr<'a> {
    pub operator: scanner::Token<'a>,
    pub right: Box<Expr<'a>>,
}

macro_rules! visitor_impl {
    ( $type:ident < $lt:lifetime >, $method:ident  ) => {
        impl<$lt, R> Visited<$lt, R> for $type<$lt> {
            fn accept(&self, visitor: impl Visitor<$lt, R>) -> R {
                visitor.$method(&self)
            }
        }
    };
    ( $type:ident, $method:ident  ) => {
        impl<'a, R> Visited<'a, R> for $type {
            fn accept(&self, visitor: impl Visitor<'a, R>) -> R {
                visitor.$method(&self)
            }
        }
    };
}

visitor_impl!(BinaryExpr<'a>, visit_binary_expr);
visitor_impl!(GroupingExpr<'a>, visit_grouping_expr);
visitor_impl!(LiteralExpr, visit_literal_expr);
visitor_impl!(UnaryExpr<'a>, visit_unary_expr);

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
