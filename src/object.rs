use std::fmt;

#[derive(Debug, Clone)]
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
