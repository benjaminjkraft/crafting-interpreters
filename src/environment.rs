use crate::error;
use crate::error::LoxError;
use crate::object::Object;
use crate::scanner;
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Object>,
}

impl<'a> Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &'a str, value: Object) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&mut self, name: &scanner::Token<'a>) -> Result<Object, LoxError> {
        match self.values.get(name.lexeme) {
            Some(obj) => Ok(obj.clone()),
            None => error::runtime_error(&name, &format!("Undefined variable '{}'.", name.lexeme)),
        }
    }
}
