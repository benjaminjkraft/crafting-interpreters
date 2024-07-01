use crate::ast;
use crate::error::LoxError;
use std::cell::RefCell;
use std::fmt;
use std::ptr;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Object<'a> {
    Number(f64),
    Bool(bool),
    // TODO(benkraft): Immutable strings could something something.
    String(String),
    Nil,
    BuiltinFunction(Function<'a>),
    Function(&'a ast::FunctionStmt<'a>),
}

#[derive(Clone)]
pub struct Function<'a> {
    pub arity: usize,
    pub function: Rc<RefCell<dyn FnMut(Vec<Object<'a>>) -> Result<Object<'a>, LoxError>>>,
    pub name: String,
}

impl fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {} (arity {})>", &self.name, &self.arity)
    }
}

impl fmt::Display for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {}>", &self.name)
    }
}

impl<'a> fmt::Display for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Number(v) => write!(f, "{}", v),
            Object::Bool(v) => write!(f, "{}", v),
            Object::String(v) => write!(f, "{}", v),
            Object::Nil => write!(f, "nil"),
            Object::BuiltinFunction(v) => v.fmt(f),
            Object::Function(stmt) => write!(f, "<function {}>", stmt.name.lexeme),
        }
    }
}

impl<'a> Object<'a> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Nil => false,
            Object::Bool(b) => *b,
            _ => true,
        }
    }
}

impl<'a> PartialEq for Object<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Nil, Object::Nil) => true,
            (Object::Nil, _) | (_, Object::Nil) => false,
            (Object::Bool(l), Object::Bool(r)) => l == r,
            (Object::Bool(_), _) | (_, Object::Bool(_)) => false,
            (Object::String(l), Object::String(r)) => l == r,
            (Object::String(_), _) | (_, Object::String(_)) => false,
            // Note: matching IEEE semantics rather than Java .equals semantics, because clox does
            // that anyway and I can't be bothered to match Java's nonsense.
            (Object::Number(l), Object::Number(r)) => l == r,
            (Object::Number(_), _) | (_, Object::Number(_)) => false,
            (Object::BuiltinFunction(l), Object::BuiltinFunction(r)) => {
                l.arity == r.arity && Rc::ptr_eq(&l.function, &r.function) && l.name == r.name
            }
            (Object::BuiltinFunction(_), _) | (_, Object::BuiltinFunction(_)) => false,
            (Object::Function(l), Object::Function(r)) => ptr::eq(l, r),
            // (Object::Function(_), _) | (_, Object::Function(_)) => false,
        }
    }
}
