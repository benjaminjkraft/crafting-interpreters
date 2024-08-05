use crate::ast::*;
use crate::error::{parse_error, LoxError};
use crate::scanner::Token;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
}

struct Resolver<'src> {
    scopes: Vec<HashMap<&'src str, bool>>,
    errors: Vec<LoxError>,
    current_function: FunctionType,
}

pub fn resolve<'src>(prog: &mut Program<'src>) -> Result<(), Vec<LoxError>> {
    let mut resolver = Resolver::new();
    resolver.resolve_program(prog);
    if resolver.errors.len() == 0 {
        Ok(())
    } else {
        Err(resolver.errors)
    }
}

impl<'src> Resolver<'src> {
    fn new() -> Self {
        Resolver {
            scopes: Vec::new(),
            errors: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    fn resolve_program(&mut self, node: &mut Program<'src>) {
        self.resolve_stmts(&mut node.stmts);
    }

    fn resolve_stmts(&mut self, stmts: &mut [Stmt<'src>]) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &mut Stmt<'src>) {
        match stmt {
            // Interesting statements
            Stmt::Block(node) => {
                self.begin_scope();
                self.resolve_stmts(&mut node.stmts);
                self.end_scope();
            }
            Stmt::Class(node) => {
                self.declare(&node.name);
                self.define(&node.name);
            }
            Stmt::Function(node) => {
                self.declare(&node.name);
                self.define(&node.name);

                self.resolve_function(&node.parameters, &mut node.body, FunctionType::Function);
            }
            Stmt::Return(node) => {
                if self.current_function == FunctionType::None {
                    self.errors.push(parse_error(
                        &node.keyword,
                        "Can't return from top-level code.",
                    ));
                }
                if let Some(ref mut value) = &mut node.value {
                    self.resolve_expr(value);
                }
            }
            Stmt::Var(node) => {
                self.declare(&node.name);
                if let Some(ref mut init) = &mut node.initializer {
                    self.resolve_expr(init);
                }
                self.define(&node.name);
            }

            // Just walk
            Stmt::Expr(node) => {
                self.resolve_expr(&mut node.expr);
            }
            Stmt::If(node) => {
                self.resolve_expr(&mut node.condition);
                self.resolve_stmt(&mut node.then_);
                if let Some(ref mut else_) = &mut node.else_ {
                    self.resolve_stmt(else_);
                }
            }
            Stmt::Print(node) => {
                self.resolve_expr(&mut node.expr);
            }
            Stmt::While(node) => {
                self.resolve_expr(&mut node.condition);
                self.resolve_stmt(&mut node.body);
            }
        }
    }

    fn resolve_function(
        &mut self,
        parameters: &Vec<Token<'src>>,
        body: &mut [Stmt<'src>],
        type_: FunctionType,
    ) {
        let enclosing_function = self.current_function;
        self.current_function = type_;
        self.begin_scope();

        for parameter in parameters {
            self.declare(parameter);
            self.define(parameter);
        }
        self.resolve_stmts(body);

        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn resolve_expr(&mut self, expr: &mut Expr<'src>) {
        match expr {
            // Interesting expressions
            Expr::Variable(node) => {
                if let Some(scope) = self.scopes.last() {
                    if scope.get(node.name.lexeme) == Some(&false) {
                        self.errors.push(parse_error(
                            &node.name,
                            "Can't read local variable in its own initializer.",
                        ));
                    }
                }

                self.resolve_local(&mut node.resolved_depth, &node.name);
            }
            Expr::Assign(node) => {
                self.resolve_expr(&mut node.value);
                self.resolve_local(&mut node.resolved_depth, &node.name);
            }

            // Just walk
            Expr::Binary(node) => {
                self.resolve_expr(&mut node.left);
                self.resolve_expr(&mut node.right);
            }
            Expr::Call(node) => {
                self.resolve_expr(&mut node.callee);
                for argument in &mut node.arguments {
                    self.resolve_expr(argument);
                }
            }
            Expr::Get(node) => {
                self.resolve_expr(&mut node.object);
            }
            Expr::Grouping(node) => {
                self.resolve_expr(&mut node.expr);
            }
            Expr::Literal(_) => {}
            Expr::Logical(node) => {
                self.resolve_expr(&mut node.left);
                self.resolve_expr(&mut node.right);
            }
            Expr::Set(node) => {
                self.resolve_expr(&mut node.object);
                self.resolve_expr(&mut node.value);
            }
            Expr::Unary(node) => {
                self.resolve_expr(&mut node.right);
            }
        }
    }

    fn resolve_local(&self, depth_field: &mut Option<usize>, name: &Token<'src>) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name.lexeme) {
                *depth_field = Some(i);
                break;
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token<'src>) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name.lexeme) {
                self.errors.push(parse_error(
                    name,
                    "Already a variable with this name in this scope.",
                ));
            }
            scope.insert(name.lexeme, false);
        }
    }

    fn define(&mut self, name: &Token<'src>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme, true);
        }
    }
}

// TODO: how to test directly? (esp. without writing another traversal...)
