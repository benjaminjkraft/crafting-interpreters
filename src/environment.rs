use crate::object::Object;
use crate::scanner;
use crate::unwind::Unwinder;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// TODO: type alias for Rc<RefCell<Environment<>>>, to elide the .borrow().thing()?
pub struct Environment<'ast, 'src: 'ast> {
    values: HashMap<String, Object<'ast, 'src>>,
    enclosing: Option<Rc<RefCell<Environment<'ast, 'src>>>>,
}

impl Environment<'_, '_> {
    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        if depth == 0 {
            writeln!(f, "===================== environment =====================")?;
        }
        for (k, v) in &self.values {
            writeln!(f, "{}{} = {}", "\t".repeat(depth), k, v)?;
        }
        if let Some(next) = &self.enclosing {
            next.borrow().fmt_indented(f, depth + 1)?;
        }
        if depth == 0 {
            writeln!(f, "=======================================================")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Environment<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
    }
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

    pub fn get_at(
        &self,
        depth: usize,
        name: &scanner::Token<'src>,
    ) -> Result<Object<'ast, 'src>, Unwinder<'ast, 'src>> {
        if depth == 0 {
            self.get(name)
        } else {
            let next = self.enclosing.as_ref().unwrap().borrow();
            next.get_at(depth - 1, name)
        }
    }

    pub fn get(
        &self,
        name: &scanner::Token<'src>,
    ) -> Result<Object<'ast, 'src>, Unwinder<'ast, 'src>> {
        match (self.values.get(name.lexeme), &self.enclosing) {
            (Some(obj), _) => Ok(obj.clone()),
            (None, Some(enclosing)) => enclosing.borrow().get(name),
            (None, None) => undefined(name),
        }
    }

    pub fn assign_at(
        &mut self,
        depth: usize,
        name: &scanner::Token<'src>,
        value: Object<'ast, 'src>,
    ) -> Result<(), Unwinder<'ast, 'src>> {
        if depth == 0 {
            self.assign(name, value)
        } else {
            let mut next = self.enclosing.as_ref().unwrap().borrow_mut();
            next.assign_at(depth - 1, name, value)
        }
    }

    pub fn assign(
        &mut self,
        name: &scanner::Token<'src>,
        value: Object<'ast, 'src>,
    ) -> Result<(), Unwinder<'ast, 'src>> {
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

fn undefined<'ast, 'src: 'ast, T>(name: &scanner::Token<'src>) -> Result<T, Unwinder<'ast, 'src>> {
    Unwinder::err(name, &format!("Undefined variable '{}'.", name.lexeme))
}
