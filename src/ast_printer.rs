use crate::ast::Visited;
use crate::ast::*;
use crate::object;
use crate::scanner;
use itertools::Itertools;

pub struct AstPrinter {}

impl<'a> AstPrinter {
    pub fn print(&self, expr: Expr<'a>) -> String {
        expr.accept(self)
    }

    fn parenthesize(&self, name: &'a str, exprs: Vec<&Box<Expr<'a>>>) -> String {
        return format!(
            "({}{})",
            name,
            exprs
                .into_iter()
                .map(|e| format!(" {}", (*e).accept(self)))
                .join("")
        );
    }
}

impl<'a> Visitor<'a, String> for &AstPrinter {
    fn visit_binary_expr(&self, node: &BinaryExpr<'a>) -> String {
        self.parenthesize(node.operator.lexeme, vec![&node.left, &node.right])
    }
    fn visit_grouping_expr(&self, node: &GroupingExpr<'a>) -> String {
        self.parenthesize("group", vec![&node.expr])
    }
    fn visit_literal_expr(&self, node: &LiteralExpr) -> String {
        self.parenthesize(&node.value.to_string(), vec![])
    }
    fn visit_unary_expr(&self, node: &UnaryExpr<'a>) -> String {
        self.parenthesize(node.operator.lexeme, vec![&node.right])
    }
}

#[test]
fn test_printer() {
    let expr = Expr::Binary(BinaryExpr {
        left: Box::new(Expr::Unary(UnaryExpr {
            operator: scanner::Token {
                type_: scanner::TokenType::Minus,
                lexeme: "-",
                literal: object::Object::Nil,
                line: 1,
            },
            right: Box::new(Expr::Literal(LiteralExpr {
                value: object::Object::Int(123),
            })),
        })),
        operator: scanner::Token {
            type_: scanner::TokenType::Star,
            lexeme: "*",
            literal: object::Object::Nil,
            line: 1,
        },
        right: Box::new(Expr::Grouping(GroupingExpr {
            expr: Box::new(Expr::Literal(LiteralExpr {
                value: object::Object::Float(45.67),
            })),
        })),
    });
    let printer = AstPrinter {};
    insta::assert_debug_snapshot!(printer.print(expr))
}
