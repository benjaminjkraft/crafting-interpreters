use crate::ast;
use crate::environment::Environment;
use crate::error::LoxError;
use std::cell::RefCell;
use std::fmt;
use std::ptr;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    Bool(bool),
    // TODO(benkraft): Immutable strings could something something.
    String(String),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Number(v) => write!(f, "{}", v),
            Literal::Bool(v) => write!(f, "{}", v),
            Literal::String(v) => write!(f, "{}", v),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

impl Literal {
    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Nil => false,
            Literal::Bool(b) => *b,
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Object<'ast, 'src: 'ast> {
    Literal(Literal),
    BuiltinFunction(Function<'ast, 'src>),
    Function {
        declaration: &'ast ast::FunctionStmt<'src>,
        closure: Rc<RefCell<Environment<'ast, 'src>>>,
    },
}

#[derive(Clone)]
pub struct Function<'ast, 'src> {
    pub arity: usize,
    pub function:
        Rc<RefCell<dyn FnMut(Vec<Object<'ast, 'src>>) -> Result<Object<'ast, 'src>, LoxError>>>,
    pub name: String,
}

impl fmt::Debug for Function<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {} (arity {})>", &self.name, &self.arity)
    }
}

impl fmt::Display for Function<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {}>", &self.name)
    }
}

impl<'ast, 'src: 'ast> fmt::Display for Object<'ast, 'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Literal(v) => write!(f, "{}", v),
            Object::BuiltinFunction(v) => v.fmt(f),
            Object::Function {
                declaration,
                closure: _,
            } => write!(f, "<function {}>", declaration.name.lexeme),
        }
    }
}

impl<'ast, 'src: 'ast> Object<'ast, 'src> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Literal(v) => v.is_truthy(),
            _ => true,
        }
    }
}

impl<'ast, 'src: 'ast> PartialEq for Object<'ast, 'src> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Literal(l), Object::Literal(r)) => l == r,
            (Object::Literal(_), _) | (_, Object::Literal(_)) => false,
            (Object::BuiltinFunction(l), Object::BuiltinFunction(r)) => {
                l.arity == r.arity && Rc::ptr_eq(&l.function, &r.function) && l.name == r.name
            }
            (Object::BuiltinFunction(_), _) | (_, Object::BuiltinFunction(_)) => false,
            (
                Object::Function {
                    declaration: l,
                    closure: _,
                },
                Object::Function {
                    declaration: r,
                    closure: _,
                },
            ) => ptr::eq(l, r),
            // (Object::Function(_), _) | (_, Object::Function(_)) => false,
        }
    }
}
