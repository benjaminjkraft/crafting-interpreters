use crate::ast::*;
#[cfg(test)]
use crate::parser;
use itertools::Itertools;
use std::fmt;

struct AstPrinter {}

#[allow(dead_code)]
pub fn print<'a>(prog: Program<'a>) -> String {
    (AstPrinter {}).visit_program(&prog)
}

fn parenthesize(items: impl IntoIterator<Item = impl fmt::Display>) -> String {
    format!("({})", items.into_iter().join(" "))
}

impl<'a> Visitor<'a, String, String> for AstPrinter {
    fn visit_program(&mut self, node: &Program<'a>) -> String {
        node.stmts
            .iter()
            .map(|stmt| self.visit_stmt(stmt))
            .join("\n")
    }
    fn visit_assign_expr(&mut self, node: &AssignExpr<'a>) -> String {
        parenthesize(&["assign", node.name.lexeme, &self.visit_expr(&node.value)])
    }
    fn visit_binary_expr(&mut self, node: &BinaryExpr<'a>) -> String {
        parenthesize(&[
            node.operator.lexeme,
            &self.visit_expr(&node.left),
            &self.visit_expr(&node.right),
        ])
    }
    fn visit_grouping_expr(&mut self, node: &GroupingExpr<'a>) -> String {
        parenthesize(&["group", &self.visit_expr(&node.expr)])
    }
    fn visit_literal_expr(&mut self, node: &LiteralExpr) -> String {
        parenthesize(&[&node.value.to_string()])
    }
    fn visit_logical_expr(&mut self, node: &LogicalExpr<'a>) -> String {
        parenthesize(&[
            node.operator.lexeme,
            &self.visit_expr(&node.left),
            &self.visit_expr(&node.right),
        ])
    }
    fn visit_unary_expr(&mut self, node: &UnaryExpr<'a>) -> String {
        parenthesize(&[node.operator.lexeme, &self.visit_expr(&node.right)])
    }
    fn visit_variable_expr(&mut self, node: &VariableExpr<'a>) -> String {
        parenthesize(&["variable", node.name.lexeme])
    }

    fn visit_block_stmt(&mut self, node: &BlockStmt<'a>) -> String {
        format!(
            "(block\n{})",
            node.stmts
                .iter()
                .map(|stmt| format!("\t{}\n", self.visit_stmt(stmt)))
                .join("")
        )
    }
    fn visit_expr_stmt(&mut self, node: &ExprStmt<'a>) -> String {
        parenthesize(&["expr", &self.visit_expr(&node.expr)])
    }
    fn visit_if_stmt(&mut self, node: &IfStmt<'a>) -> String {
        let mut parts = vec![
            "if".to_string(),
            self.visit_expr(&node.condition),
            self.visit_stmt(&node.then_),
        ];
        match &node.else_ {
            Some(e) => parts.push(self.visit_stmt(e)),
            None => {}
        }
        parenthesize(parts)
    }
    fn visit_print_stmt(&mut self, node: &PrintStmt<'a>) -> String {
        parenthesize(&["print", &self.visit_expr(&node.expr)])
    }
    fn visit_var_stmt(&mut self, node: &VarStmt<'a>) -> String {
        let mut parts = vec!["var".to_string(), node.name.lexeme.to_string()];
        match &node.initializer {
            Some(e) => parts.push(self.visit_expr(e)),
            None => {}
        }
        parenthesize(parts)
    }
    fn visit_while_stmt(&mut self, node: &WhileStmt<'a>) -> String {
        parenthesize(&[
            "while",
            &self.visit_expr(&node.condition),
            &self.visit_stmt(&node.body),
        ])
    }
}

#[test]
fn test_printer() {
    insta::assert_debug_snapshot!(print(parser::must_parse("-123*(45.67);")));
}
