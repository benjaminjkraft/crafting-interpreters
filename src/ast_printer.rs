#[cfg(test)]
use crate::ast::*;
#[cfg(test)]
use crate::parser;
#[cfg(test)]
use itertools::Itertools;
#[cfg(test)]
use std::fmt;

#[cfg(test)]
pub fn print<'src>(node: &Program<'src>) -> String {
    node.stmts.iter().map(|stmt| print_stmt(stmt)).join("\n")
}

#[cfg(test)]
fn parenthesize(items: impl IntoIterator<Item = impl fmt::Display>) -> String {
    format!("({})", items.into_iter().join(" "))
}

#[cfg(test)]
fn print_expr<'src>(node: &Expr<'src>) -> String {
    match node {
        Expr::Assign(node) => parenthesize(&["assign", node.name.lexeme, &print_expr(&node.value)]),
        Expr::Binary(node) => parenthesize(&[
            node.operator.lexeme,
            &print_expr(&node.left),
            &print_expr(&node.right),
        ]),
        Expr::Call(node) => {
            let mut parts = vec!["call".to_string(), print_expr(&node.callee)];
            for argument in &node.arguments {
                parts.push(print_expr(&argument));
            }
            parenthesize(&parts)
        }
        Expr::Get(node) => parenthesize(&["get", &print_expr(&node.object), node.name.lexeme]),
        Expr::Grouping(node) => parenthesize(&["group", &print_expr(&node.expr)]),
        Expr::Literal(node) => parenthesize(&[&node.value.to_string()]),
        Expr::Logical(node) => parenthesize(&[
            node.operator.lexeme,
            &print_expr(&node.left),
            &print_expr(&node.right),
        ]),
        Expr::Set(node) => parenthesize(&[
            "set",
            &print_expr(&node.object),
            node.name.lexeme,
            &print_expr(&node.value),
        ]),
        Expr::This(_) => parenthesize(&["this"]),
        Expr::Unary(node) => parenthesize(&[node.operator.lexeme, &print_expr(&node.right)]),
        Expr::Variable(node) => parenthesize(&["variable", node.name.lexeme]),
    }
}

#[cfg(test)]
fn print_block<'src>(head: &str, stmts: &Vec<Stmt<'src>>) -> String {
    let body = stmts
        .iter()
        .map(|stmt| format!("\t{}\n", print_stmt(stmt)))
        .join("");
    format!("({head}\n{body})")
}

// TODO(benkraft): ick! how to avoid?
#[cfg(test)]
fn print_function_block<'src>(head: &str, stmts: &Vec<FunctionStmt<'src>>) -> String {
    let body = stmts
        .iter()
        .map(|stmt| format!("\t{}\n", print_function(stmt)))
        .join("");
    format!("({head}\n{body})")
}

#[cfg(test)]
fn print_function<'src>(node: &FunctionStmt<'src>) -> String {
    let mut parts = vec!["fun", node.name.lexeme];
    parts.extend(node.parameters.iter().map(|param| param.lexeme));
    let body = print_block("", &node.body);
    parts.push(&body);
    parenthesize(parts)
}

#[cfg(test)]
fn print_stmt<'src>(node: &Stmt<'src>) -> String {
    match node {
        Stmt::Block(node) => print_block("block", &node.stmts),
        Stmt::Class(node) => {
            print_function_block(&format!("class {}", node.name.lexeme), &node.methods)
        }
        Stmt::Expr(node) => parenthesize(&["expr", &print_expr(&node.expr)]),
        Stmt::Function(node) => print_function(&node),
        Stmt::If(node) => {
            let mut parts = vec![
                "if".to_string(),
                print_expr(&node.condition),
                print_stmt(&node.then_),
            ];
            if let Some(e) = &node.else_ {
                parts.push(print_stmt(e))
            }
            parenthesize(parts)
        }
        Stmt::Print(node) => parenthesize(&["print", &print_expr(&node.expr)]),
        Stmt::Return(node) => {
            let mut parts = vec!["return".to_string()];
            if let Some(e) = &node.value {
                parts.push(print_expr(e))
            }
            parenthesize(parts)
        }
        Stmt::Var(node) => {
            let mut parts = vec!["var".to_string(), node.name.lexeme.to_string()];
            if let Some(e) = &node.initializer {
                parts.push(print_expr(e))
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
    insta::assert_debug_snapshot!(print(&parser::must_parse("-123*(45.67);")));
}
