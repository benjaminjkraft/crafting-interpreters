use crate::ast;
use crate::environment::Environment;
use crate::error::LoxError;
use crate::scanner;
use crate::unwind::Unwinder;
use std::cell::RefCell;
use std::collections::HashMap;
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
            Literal::Number(v) => write!(f, "{v}"),
            Literal::Bool(v) => write!(f, "{v}"),
            Literal::String(v) => write!(f, "{v}"),
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
    Class(Rc<RefCell<Class<'ast, 'src>>>),
    Instance(Rc<RefCell<Instance<'ast, 'src>>>),
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

#[derive(Debug)]
pub struct Class<'ast, 'src> {
    pub name: &'ast scanner::Token<'src>,
}

impl fmt::Display for Class<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", &self.name.lexeme)
    }
}

#[derive(Debug)]
pub struct Instance<'ast, 'src> {
    pub class_: Rc<RefCell<Class<'ast, 'src>>>,
    pub fields: HashMap<String, Object<'ast, 'src>>,
}

impl<'ast, 'src> Instance<'ast, 'src> {
    pub fn get(
        &self,
        name: &scanner::Token<'src>,
    ) -> Result<Object<'ast, 'src>, Unwinder<'ast, 'src>> {
        match self.fields.get(name.lexeme) {
            Some(obj) => Ok(obj.clone()),
            None => Unwinder::err(name, &format!("Undefined property '{}'.", name.lexeme)),
        }
    }

    pub fn set(&mut self, name: &scanner::Token<'src>, value: Object<'ast, 'src>) {
        self.fields.insert(name.lexeme.to_string(), value);
    }
}

impl fmt::Display for Instance<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<instance of {}>", &self.class_.borrow().name.lexeme)
    }
}

impl<'ast, 'src: 'ast> fmt::Display for Object<'ast, 'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Literal(v) => write!(f, "{v}"),
            Object::BuiltinFunction(v) => v.fmt(f),
            Object::Function { declaration, .. } => {
                write!(f, "<function {}>", declaration.name.lexeme)
            }
            Object::Class(c) => c.borrow().fmt(f),
            Object::Instance(i) => i.borrow().fmt(f),
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
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Object::Literal(l), Object::Literal(r)) => l == r,
            (Object::Literal(_), _) | (_, Object::Literal(_)) => false,
            (Object::BuiltinFunction(l), Object::BuiltinFunction(r)) => {
                l.arity == r.arity && Rc::ptr_eq(&l.function, &r.function) && l.name == r.name
            }
            (Object::BuiltinFunction(_), _) | (_, Object::BuiltinFunction(_)) => false,
            (Object::Function { declaration: l, .. }, Object::Function { declaration: r, .. }) => {
                ptr::eq(*l, *r)
            }
            (Object::Function { .. }, _) | (_, Object::Function { .. }) => false,
            (Object::Class(l), Object::Class(r)) => Rc::ptr_eq(l, r),
            (Object::Class(_), _) | (_, Object::Class(_)) => false,
            (Object::Instance(l), Object::Instance(r)) => Rc::ptr_eq(l, r),
            // (Object::Instance(l), _) | (_, Object::Instance(r)) => false,
        }
    }
}
