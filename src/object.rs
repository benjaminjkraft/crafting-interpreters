use crate::ast;
use crate::environment::Environment;
use crate::error::LoxError;
use crate::scanner;
use crate::unwind::Unwinder;
use derive_more::From;
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

#[derive(Debug, Clone, From)]
pub enum Object<'ast, 'src: 'ast> {
    Literal(Literal),
    BuiltinFunction(BuiltinFunction<'ast, 'src>),
    Function(Function<'ast, 'src>),
    Class(Rc<RefCell<Class<'ast, 'src>>>),
    Instance(Rc<RefCell<Instance<'ast, 'src>>>),
}

#[derive(Clone)]
pub struct BuiltinFunction<'ast, 'src> {
    pub arity: usize,
    pub function:
        Rc<RefCell<dyn FnMut(Vec<Object<'ast, 'src>>) -> Result<Object<'ast, 'src>, LoxError>>>,
    pub name: String,
}

impl fmt::Debug for BuiltinFunction<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {} (arity {})>", &self.name, &self.arity)
    }
}

impl fmt::Display for BuiltinFunction<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {}>", &self.name)
    }
}

#[derive(Clone)]
pub struct Function<'ast, 'src> {
    pub declaration: &'ast ast::FunctionStmt<'src>,
    pub closure: Rc<RefCell<Environment<'ast, 'src>>>,
    pub is_initializer: bool,
}

impl<'ast, 'src> Function<'ast, 'src> {
    pub fn bind(&self, instance: Rc<RefCell<Instance<'ast, 'src>>>) -> Self {
        let mut environment = Environment::child(self.closure.clone());
        environment.define("this", instance.into());
        Function {
            declaration: self.declaration,
            closure: Rc::new(RefCell::new(environment)),
            is_initializer: self.is_initializer,
        }
    }
}

impl fmt::Debug for Function<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {}>", &self.declaration.name.lexeme)
    }
}

impl fmt::Display for Function<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function {}>", &self.declaration.name.lexeme)
    }
}

#[derive(Debug)]
pub struct Class<'ast, 'src> {
    pub name: &'ast scanner::Token<'src>,
    pub superclass: Option<Rc<RefCell<Class<'ast, 'src>>>>,
    pub methods: HashMap<String, Function<'ast, 'src>>,
}

impl<'ast, 'src> Class<'ast, 'src> {
    pub fn find_method(&self, name: &str) -> Option<Function<'ast, 'src>> {
        if let Some(method) = self.methods.get(name) {
            Some(method.clone())
        } else if let Some(sup) = &self.superclass {
            sup.borrow().find_method(name)
        } else {
            None
        }
    }
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

fn instance_get_field<'ast, 'src>(
    inst: &Rc<RefCell<Instance<'ast, 'src>>>,
    name: &scanner::Token<'src>,
) -> Option<Object<'ast, 'src>> {
    inst.borrow().fields.get(name.lexeme).cloned()
}

fn instance_get_method<'ast, 'src>(
    inst: &Rc<RefCell<Instance<'ast, 'src>>>,
    name: &scanner::Token<'src>,
) -> Option<Function<'ast, 'src>> {
    inst.borrow().class_.borrow().find_method(name.lexeme)
}

// TODO: possible to refactor types to make this a method?
pub fn instance_get<'ast, 'src>(
    inst: Rc<RefCell<Instance<'ast, 'src>>>,
    name: &scanner::Token<'src>,
) -> Result<Object<'ast, 'src>, Unwinder<'ast, 'src>> {
    if let Some(obj) = instance_get_field(&inst, name) {
        Ok(obj)
    } else if let Some(method) = instance_get_method(&inst, name) {
        Ok(method.bind(inst).into())
    } else {
        Unwinder::err(name, &format!("Undefined property '{}'.", name.lexeme))
    }
}

impl<'ast, 'src> Instance<'ast, 'src> {
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
            Object::Function(v) => v.fmt(f),
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
            (Object::Function(l), Object::Function(r)) => ptr::eq(l.declaration, r.declaration),
            (Object::Function { .. }, _) | (_, Object::Function { .. }) => false,
            (Object::Class(l), Object::Class(r)) => Rc::ptr_eq(l, r),
            (Object::Class(_), _) | (_, Object::Class(_)) => false,
            (Object::Instance(l), Object::Instance(r)) => Rc::ptr_eq(l, r),
            // (Object::Instance(l), _) | (_, Object::Instance(r)) => false,
        }
    }
}
