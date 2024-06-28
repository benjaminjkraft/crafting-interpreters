#[cfg(test)]
use crate::ast::*;
#[cfg(test)]
use crate::parser;
#[cfg(test)]
use itertools::Itertools;
#[cfg(test)]
use std::fmt;

#[cfg(test)]
pub fn print<'a>(node: Program<'a>) -> String {
    node.stmts.iter().map(|stmt| print_stmt(stmt)).join("\n")
}

#[cfg(test)]
fn parenthesize(items: impl IntoIterator<Item = impl fmt::Display>) -> String {
    format!("({})", items.into_iter().join(" "))
}

#[cfg(test)]
fn print_expr<'a>(node: &Expr<'a>) -> String {
    match node {
        Expr::Assign(node) => parenthesize(&["assign", node.name.lexeme, &print_expr(&node.value)]),
        Expr::Binary(node) => parenthesize(&[
            node.operator.lexeme,
            &print_expr(&node.left),
            &print_expr(&node.right),
        ]),
        Expr::Grouping(node) => parenthesize(&["group", &print_expr(&node.expr)]),
        Expr::Literal(node) => parenthesize(&[&node.value.to_string()]),
        Expr::Logical(node) => parenthesize(&[
            node.operator.lexeme,
            &print_expr(&node.left),
            &print_expr(&node.right),
        ]),
        Expr::Unary(node) => parenthesize(&[node.operator.lexeme, &print_expr(&node.right)]),
        Expr::Variable(node) => parenthesize(&["variable", node.name.lexeme]),
    }
}

#[cfg(test)]
fn print_stmt<'a>(node: &Stmt<'a>) -> String {
    match node {
        Stmt::Block(node) => format!(
            "(block\n{})",
            node.stmts
                .iter()
                .map(|stmt| format!("\t{}\n", print_stmt(stmt)))
                .join("")
        ),
        Stmt::Expr(node) => parenthesize(&["expr", &print_expr(&node.expr)]),
        Stmt::If(node) => {
            let mut parts = vec![
                "if".to_string(),
                print_expr(&node.condition),
                print_stmt(&node.then_),
            ];
            match &node.else_ {
                Some(e) => parts.push(print_stmt(e)),
                None => {}
            }
            parenthesize(parts)
        }
        Stmt::Print(node) => parenthesize(&["print", &print_expr(&node.expr)]),
        Stmt::Var(node) => {
            let mut parts = vec!["var".to_string(), node.name.lexeme.to_string()];
            match &node.initializer {
                Some(e) => parts.push(print_expr(e)),
                None => {}
            }
            parenthesize(parts)
        }
        Stmt::While(node) => parenthesize(&[
            "while",
            &print_expr(&node.condition),
            &print_stmt(&node.body),
        ]),
    }
}

#[test]
fn test_printer() {
    insta::assert_debug_snapshot!(print(parser::must_parse("-123*(45.67);")));
}
