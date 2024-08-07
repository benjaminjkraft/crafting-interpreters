use crate::ast::*;
use crate::error::{parse_error, LoxError};
use crate::scanner::Token;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassType {
    None,
    Class,
    Subclass,
}

struct Resolver<'src> {
    scopes: Vec<HashMap<&'src str, bool>>,
    errors: Vec<LoxError>,
    current_function: FunctionType,
    current_class: ClassType,
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
            current_class: ClassType::None,
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
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;

                self.declare(&node.name);
                self.define(&node.name);

                if let Some(ref mut sup) = &mut node.superclass {
                    self.current_class = ClassType::Subclass;
                    if sup.name.lexeme == node.name.lexeme {
                        self.errors
                            .push(parse_error(&sup.name, "A class can't inherit from itself."));
                    } else {
                        self.resolve_variable(sup);
                    }
                }

                if node.superclass.is_some() {
                    self.begin_scope();
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert("super", true);
                    }
                }

                self.begin_scope();
                // TODO: refactor define and use?
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert("this", true);
                }

                for method in &mut node.methods {
                    self.resolve_function(
                        &method.parameters,
                        &mut method.body,
                        if method.name.lexeme == "init" {
                            FunctionType::Initializer
                        } else {
                            FunctionType::Method
                        },
                    );
                }

                self.end_scope();

                if node.superclass.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
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
                    if self.current_function == FunctionType::Initializer {
                        self.errors.push(parse_error(
                            &node.keyword,
                            "Can't return a value from an initializer.",
                        ));
                    }
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

    fn resolve_variable(&mut self, node: &mut VariableExpr<'src>) {
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

    fn resolve_expr(&mut self, expr: &mut Expr<'src>) {
        match expr {
            // Interesting expressions
            Expr::Variable(node) => {
                self.resolve_variable(node);
            }
            Expr::Assign(node) => {
                self.resolve_expr(&mut node.value);
                self.resolve_local(&mut node.resolved_depth, &node.name);
            }
            Expr::Super(node) => {
                match self.current_class {
                    ClassType::None => self.errors.push(parse_error(
                        &node.keyword,
                        "Can't use 'super' outside of a class.",
                    )),
                    ClassType::Class => self.errors.push(parse_error(
                        &node.keyword,
                        "Can't use 'super' in a class with no superclass.",
                    )),
                    ClassType::Subclass => {}
                }
                self.resolve_local(&mut node.resolved_depth, &node.keyword);
            }
            Expr::This(node) => {
                if self.current_class == ClassType::None {
                    self.errors.push(parse_error(
                        &node.keyword,
                        "Can't use 'this' outside of a class.",
                    ));
                }
                self.resolve_local(&mut node.resolved_depth, &node.keyword);
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
