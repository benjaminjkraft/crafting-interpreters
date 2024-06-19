use crate::ast::*;
#[cfg(test)]
use crate::parser;
use itertools::Itertools;

struct AstPrinter {}

pub fn print<'a>(prog: Program<'a>) -> String {
    (AstPrinter {}).visit_program(&prog)
}

impl<'a> AstPrinter {
    fn parenthesize(&mut self, name: &'a str, exprs: Vec<&Box<Expr<'a>>>) -> String {
        return format!(
            "({}{})",
            name,
            exprs
                .into_iter()
                .map(|e| format!(" {}", self.visit_expr(e)))
                .join("")
        );
    }
}

impl<'a> Visitor<'a, String, String> for AstPrinter {
    fn visit_program(&mut self, node: &Program<'a>) -> String {
        node.stmts
            .iter()
            .map(|stmt| self.visit_stmt(stmt))
            .join("\n")
    }
    fn visit_assign_expr(&mut self, node: &AssignExpr<'a>) -> String {
        self.parenthesize(&format!("assign {}", node.name.lexeme), vec![&node.value])
    }
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
    fn visit_variable_expr(&mut self, node: &VariableExpr<'a>) -> String {
        format!("(variable {})", node.name.lexeme)
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
        self.parenthesize("expr", vec![&node.expr])
    }
    fn visit_print_stmt(&mut self, node: &PrintStmt<'a>) -> String {
        self.parenthesize("print", vec![&node.expr])
    }
    fn visit_var_stmt(&mut self, node: &VarStmt<'a>) -> String {
        let start = format!("var {}", node.name.lexeme);
        self.parenthesize(&start, node.initializer.iter().collect())
    }
}

#[test]
fn test_printer() {
    insta::assert_debug_snapshot!(print(parser::must_parse("-123*(45.67);")));
}
