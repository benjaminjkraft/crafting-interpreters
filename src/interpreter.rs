use crate::ast::*;
use crate::error::{runtime_error, LoxError};
use crate::object::Object;
use crate::parser;
use crate::scanner;
use crate::scanner::TokenType;

pub struct Interpreter {}

pub fn evaluate_source(interpreter: &mut Interpreter, source: &str) -> Result<Object, LoxError> {
    let tokens = scanner::scan_tokens(source)?;
    let expr = parser::parse(tokens)?;
    interpreter.evaluate(&expr)
}

impl<'a> Interpreter {
    pub fn evaluate(&mut self, expr: &Expr<'a>) -> Result<Object, LoxError> {
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

impl<'a> Visitor<'a, Result<Object, LoxError>> for Interpreter {
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
}

#[cfg(test)]
fn assert_evaluates_to(source: &str, expected: Result<Object, &str>) {
    let mut interpreter = Interpreter {};
    let actual = evaluate_source(&mut interpreter, source);
    match (actual, expected) {
        (Ok(a), Ok(e)) => assert_eq!(a, e),
        (Ok(a), Err(e)) => assert!(false, "Expected error {}, got {}", e, a),
        (Err(a), Err(e)) => assert_eq!(a.to_string(), e),
        (Err(a), Ok(e)) => assert!(false, "Expected {}, got error {}", e, a),
    }
}

#[test]
fn test_evaluate_expr() {
    assert_evaluates_to("1 + 2", Ok(Object::Number(3.0)));
    assert_evaluates_to("1 == 1", Ok(Object::Bool(true)));
    assert_evaluates_to("1 == true", Ok(Object::Bool(false)));
    assert_evaluates_to("1/0 == 1/0", Ok(Object::Bool(true)));
    assert_evaluates_to("0/0 == 0/0", Ok(Object::Bool(false)));
    assert_evaluates_to("false == nil", Ok(Object::Bool(false)));
    assert_evaluates_to("1 >= 1", Ok(Object::Bool(true)));
    assert_evaluates_to("1 > 1", Ok(Object::Bool(false)));
    assert_evaluates_to(
        "true > false",
        Err("[line 1] Error: invalid types for comparison"),
    );
    assert_evaluates_to(
        r#"1 + "a""#,
        Err("[line 1] Error: invalid types for addition"),
    );
    assert_evaluates_to(r#""a" + "b""#, Ok(Object::String("ab".to_string())));
    assert_evaluates_to("!true", Ok(Object::Bool(false)));
    assert_evaluates_to("!nil", Ok(Object::Bool(true)));
    assert_evaluates_to("1 + (2 + 3)", Ok(Object::Number(6.0)));
    assert_evaluates_to(r#""a" + "b" + "c""#, Ok(Object::String("abc".to_string())));
}
