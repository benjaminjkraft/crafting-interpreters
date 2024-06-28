use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Number(f64),
    Bool(bool),
    // TODO(benkraft): Immutable strings could something something.
    String(String),
    Nil,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Number(v) => write!(f, "{}", v),
            Object::Bool(v) => write!(f, "{}", v),
            Object::String(v) => write!(f, "{}", v),
            Object::Nil => write!(f, "nil"),
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

    pub fn is_equal(&self, other: &Self) -> bool {
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
            // (Object::Number(_), _) | (_, Object::Number(_)) => false,
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            Object::Nil => "nil".to_string(),
            Object::Bool(b) => b.to_string(),
            Object::String(s) => s.clone(),
            Object::Number(n) => n.to_string(),
        }
    }
}
