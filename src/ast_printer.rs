use crate::ast::Visited;
use crate::ast::*;
#[cfg(test)]
use crate::parser;
use itertools::Itertools;

struct AstPrinter {}

pub fn print<'a>(expr: Expr<'a>) -> String {
    expr.accept(&mut AstPrinter {})
}

impl<'a> AstPrinter {
    fn parenthesize(&mut self, name: &'a str, exprs: Vec<&Box<Expr<'a>>>) -> String {
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

impl<'a> Visitor<'a, String> for AstPrinter {
    fn visit_binary_expr(&mut self, node: &BinaryExpr<'a>) -> String {
        self.parenthesize(node.operator.lexeme, vec![&node.left, &node.right])
    }
    fn visit_grouping_expr(&mut self, node: &GroupingExpr<'a>) -> String {
        self.parenthesize("group", vec![&node.expr])
    }
    fn visit_literal_expr(&mut self, node: &LiteralExpr) -> String {
        self.parenthesize(&node.value.to_string(), vec![])
    }
    fn visit_unary_expr(&mut self, node: &UnaryExpr<'a>) -> String {
        self.parenthesize(node.operator.lexeme, vec![&node.right])
    }
}

#[test]
fn test_printer() {
    insta::assert_debug_snapshot!(print(parser::must_parse("-123*(45.67)")));
}
