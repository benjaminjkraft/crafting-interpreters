use crate::ast::*;
use crate::environment::Environment;
use crate::error::{runtime_error, LoxError};
use crate::object::Object;
use crate::parser;
use crate::scanner;
use crate::scanner::TokenType;

pub struct Interpreter<F: FnMut(String)> {
    printer: F,
    environment: Environment,
}

pub fn interpreter() -> Interpreter<impl FnMut(String)> {
    Interpreter {
        printer: |s| println!("{}", s),
        environment: Environment::new(),
    }
}

pub fn evaluate_source<F: FnMut(String)>(
    interpreter: &mut Interpreter<F>,
    source: &str,
) -> Result<(), LoxError> {
    let tokens = scanner::scan_tokens(source)?;
    let prog = parser::parse(tokens)?;
    for stmt in prog.iter() {
        interpreter.execute(&stmt)?;
    }
    Ok(())
}

impl<'a, F: FnMut(String)> Interpreter<F> {
    fn execute(&mut self, stmt: &Stmt<'a>) -> Result<Object, LoxError> {
        stmt.accept(self)
    }

    fn evaluate(&mut self, expr: &Expr<'a>) -> Result<Object, LoxError> {
        expr.accept(self)
    }

    fn is_truthy(&mut self, obj: Object) -> bool {
        match obj {
            Object::Nil => false,
            Object::Bool(b) => b,
            _ => true,
        }
    }

    fn is_equal(&mut self, left: Object, right: Object) -> bool {
        match (left, right) {
            (Object::Nil, Object::Nil) => true,
            (Object::Nil, _) | (_, Object::Nil) => false,
            (Object::Bool(l), Object::Bool(r)) => l == r,
            (Object::Bool(_), _) | (_, Object::Bool(_)) => false,
            (Object::String(l), Object::String(r)) => l == r,
            (Object::String(_), _) | (_, Object::String(_)) => false,
            // Note: matching IEEE semantics rather than Java .equals semantics, because clox does
            // that anyway and I can't be bothered to match Java's nonsense.
            (Object::Number(l), Object::Number(r)) => l == r,
            // (Object::Number(_), _) | (_, Object::Number(_)) => false,
        }
    }

    pub fn stringify(&self, obj: Object) -> String {
        match obj {
            Object::Nil => "nil".to_string(),
            Object::Bool(b) => b.to_string(),
            Object::String(s) => s.clone(),
            Object::Number(n) => n.to_string(),
        }
    }
}

impl<'a, F: FnMut(String)> Visitor<'a, Result<Object, LoxError>> for Interpreter<F> {
    fn visit_binary_expr(&mut self, node: &BinaryExpr<'a>) -> Result<Object, LoxError> {
        let left = self.evaluate(&node.left)?;
        let right = self.evaluate(&node.right)?;

        match node.operator.type_ {
            TokenType::Minus => match (left, right) {
                (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l - r)),
                (_, _) => runtime_error(&node.operator, "invalid types for subtraction"),
            },
            TokenType::Plus => match (left, right) {
                (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l + r)),
                (Object::String(l), Object::String(r)) => Ok(Object::String(l + &r)),
                (_, _) => runtime_error(&node.operator, "invalid types for addition"),
            },
            TokenType::Slash => match (left, right) {
                (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l / r)),
                (_, _) => runtime_error(&node.operator, "invalid types for division"),
            },
            TokenType::Star => match (left, right) {
                (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l * r)),
                (_, _) => runtime_error(&node.operator, "invalid types for multiplication"),
            },
            TokenType::Greater => match (left, right) {
                (Object::Number(l), Object::Number(r)) => Ok(Object::Bool(l > r)),
                (_, _) => runtime_error(&node.operator, "invalid types for comparison"),
            },
            TokenType::GreaterEqual => match (left, right) {
                (Object::Number(l), Object::Number(r)) => Ok(Object::Bool(l >= r)),
                (_, _) => runtime_error(&node.operator, "invalid types for comparison"),
            },
            TokenType::Less => match (left, right) {
                (Object::Number(l), Object::Number(r)) => Ok(Object::Bool(l < r)),
                (_, _) => runtime_error(&node.operator, "invalid types for comparison"),
            },
            TokenType::LessEqual => match (left, right) {
                (Object::Number(l), Object::Number(r)) => Ok(Object::Bool(l <= r)),
                (_, _) => runtime_error(&node.operator, "invalid types for comparison"),
            },
            TokenType::EqualEqual => Ok(Object::Bool(self.is_equal(left, right))),
            TokenType::BangEqual => Ok(Object::Bool(!self.is_equal(left, right))),
            _ => runtime_error(&node.operator, "unknown operator (parser bug?)"),
        }
    }
    fn visit_grouping_expr(&mut self, node: &GroupingExpr<'a>) -> Result<Object, LoxError> {
        return self.evaluate(&node.expr);
    }
    fn visit_literal_expr(&mut self, node: &LiteralExpr) -> Result<Object, LoxError> {
        return Ok(node.value.clone());
    }
    fn visit_unary_expr(&mut self, node: &UnaryExpr<'a>) -> Result<Object, LoxError> {
        let right = self.evaluate(&node.right)?;

        match node.operator.type_ {
            TokenType::Bang => Ok(Object::Bool(!self.is_truthy(right))),
            TokenType::Minus => match right {
                Object::Number(n) => Ok(Object::Number(-n)),
                _ => runtime_error(&node.operator, "invalid type for negation"),
            },
            _ => runtime_error(&node.operator, "unknown operator (parser bug?)"),
        }
    }

    fn visit_variable_expr(&mut self, node: &VariableExpr<'a>) -> Result<Object, LoxError> {
        return self.environment.get(&node.name);
    }

    fn visit_expr_stmt(&mut self, node: &ExprStmt<'a>) -> Result<Object, LoxError> {
        self.evaluate(&node.expr)?;
        // TODO: visitor with different return for stmts?
        return Ok(Object::Nil);
    }

    fn visit_print_stmt(&mut self, node: &PrintStmt<'a>) -> Result<Object, LoxError> {
        let value = self.evaluate(&node.expr)?;
        let stringified = self.stringify(value);
        (self.printer)(stringified);
        // TODO: visitor with different return for stmts?
        return Ok(Object::Nil);
    }

    fn visit_var_stmt(&mut self, node: &VarStmt<'a>) -> Result<Object, LoxError> {
        let value = match &node.initializer {
            Some(expr) => self.evaluate(&expr)?,
            None => Object::Nil,
        };

        self.environment.define(node.name.lexeme, value);
        // TODO: visitor with different return for stmts?
        return Ok(Object::Nil);
    }
}

#[cfg(test)]
pub fn execute_for_tests(source: &str) -> Result<Vec<String>, LoxError> {
    let mut printed: Vec<String> = Vec::new();
    let mut interpreter = Interpreter {
        printer: |s| printed.push(s),
        environment: Environment::new(),
    };
    evaluate_source(&mut interpreter, source)?;
    Ok(printed)
}

#[cfg(test)]
fn assert_prints(source: &str, expected: Result<Vec<&str>, &str>) {
    let actual = execute_for_tests(source);
    match (actual, expected) {
        (Ok(a), Ok(e)) => assert_eq!(a, e.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
        (Ok(a), Err(e)) => assert!(false, "Expected error {}, got {:?}", e, a),
        (Err(a), Err(e)) => assert_eq!(a.to_string(), e),
        (Err(a), Ok(e)) => assert!(false, "Expected {:?}, got error {}", e, a),
    }
}

#[test]
fn test_evaluate_expr() {
    assert_prints("print 1 + 2;", Ok(vec!["3"]));
    assert_prints("print 1 == 1;", Ok(vec!["true"]));
    assert_prints("print 1 == true;", Ok(vec!["false"]));
    assert_prints("print 1/0 == 1/0;", Ok(vec!["true"]));
    assert_prints("print 0/0 == 0/0;", Ok(vec!["false"]));
    assert_prints("print false == nil;", Ok(vec!["false"]));
    assert_prints("print 1 >= 1;", Ok(vec!["true"]));
    assert_prints("print 1 > 1;", Ok(vec!["false"]));
    assert_prints(
        "print true > false;",
        Err("[line 1] Error: invalid types for comparison"),
    );
    assert_prints(
        r#"print 1 + "a";"#,
        Err("[line 1] Error: invalid types for addition"),
    );
    assert_prints(r#"print "a" + "b";"#, Ok(vec!["ab"]));
    assert_prints("print !true;", Ok(vec!["false"]));
    assert_prints("print !nil;", Ok(vec!["true"]));
    assert_prints("print 1 + (2 + 3);", Ok(vec!["6"]));
    assert_prints(r#"print "a" + "b" + "c";"#, Ok(vec!["abc"]));
    assert_prints("var v; print v;", Ok(vec!["nil"]));
    assert_prints("var v = 3; print v;", Ok(vec!["3"]));
    assert_prints("var v = 3; var v = 4; print v;", Ok(vec!["4"]));
}
