use crate::error;
use crate::error::LoxError;
use crate::object::Object;
use crate::scanner;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment<'ast, 'src: 'ast> {
    values: HashMap<String, Object<'ast, 'src>>,
    enclosing: Option<Rc<RefCell<Environment<'ast, 'src>>>>,
}

impl<'ast, 'src: 'ast> Environment<'ast, 'src> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn child(inner: Rc<RefCell<Environment<'ast, 'src>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(inner),
        }
    }

    pub fn define(&mut self, name: &'src str, value: Object<'ast, 'src>) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &scanner::Token<'src>) -> Result<Object<'ast, 'src>, LoxError> {
        match (self.values.get(name.lexeme), &self.enclosing) {
            (Some(obj), _) => Ok(obj.clone()),
            (None, Some(enclosing)) => enclosing.borrow().get(name),
            (None, None) => undefined(name),
        }
    }

    pub fn assign(
        &mut self,
        name: &scanner::Token<'src>,
        value: Object<'ast, 'src>,
    ) -> Result<(), LoxError> {
        if self.values.contains_key(name.lexeme) {
            self.define(name.lexeme, value);
            Ok(())
        } else {
            match &self.enclosing {
                Some(enclosing) => enclosing.borrow_mut().assign(name, value),
                None => undefined(name),
            }
        }
    }
}

fn undefined<'ast, 'src: 'ast, T>(name: &scanner::Token<'src>) -> Result<T, LoxError> {
    error::runtime_error(name, &format!("Undefined variable '{}'.", name.lexeme))
}
