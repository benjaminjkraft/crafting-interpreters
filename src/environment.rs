use crate::error;
use crate::error::LoxError;
use crate::object::Object;
use crate::scanner;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment<'a> {
    values: HashMap<String, Object<'a>>,
    enclosing: Option<Rc<RefCell<Environment<'a>>>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn child(inner: Rc<RefCell<Environment<'a>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(inner),
        }
    }

    pub fn define(&mut self, name: &'a str, value: Object<'a>) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &scanner::Token<'a>) -> Result<Object<'a>, LoxError> {
        match (self.values.get(name.lexeme), &self.enclosing) {
            (Some(obj), _) => Ok(obj.clone()),
            (None, Some(enclosing)) => enclosing.borrow().get(name),
            (None, None) => undefined(name),
        }
    }

    pub fn assign(&mut self, name: &scanner::Token<'a>, value: Object<'a>) -> Result<(), LoxError> {
        if self.values.contains_key(name.lexeme) {
            Ok(self.define(name.lexeme, value))
        } else {
            match &self.enclosing {
                Some(enclosing) => enclosing.borrow_mut().assign(name, value),
                None => undefined(name),
            }
        }
    }
}

fn undefined<'a, T>(name: &scanner::Token<'a>) -> Result<T, LoxError> {
    error::runtime_error(&name, &format!("Undefined variable '{}'.", name.lexeme))
}
