use crate::error::LoxError;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Object {
    Number(f64),
    Bool(bool),
    // TODO(benkraft): Immutable strings could something something.
    String(String),
    Nil,
    BuiltinFunction(Function),
}

#[derive(Clone)]
pub struct Function {
    pub arity: usize,
    pub function: Rc<RefCell<dyn FnMut(Vec<Object>) -> Result<Object, LoxError>>>,
    pub name: String,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {} (arity {})>", &self.name, &self.arity)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {}>", &self.name)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Number(v) => write!(f, "{}", v),
            Object::Bool(v) => write!(f, "{}", v),
            Object::String(v) => write!(f, "{}", v),
            Object::Nil => write!(f, "nil"),
            Object::BuiltinFunction(v) => v.fmt(f),
        }
    }
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Nil => false,
            Object::Bool(b) => *b,
            _ => true,
        }
    }
}

impl PartialEq for Object {
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
            } // (Object::BuiltinFunction(_), _) | (_, Object::BuiltinFunction(_)) => false,
        }
    }
}
