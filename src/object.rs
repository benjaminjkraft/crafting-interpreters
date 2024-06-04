use std::fmt;

#[derive(Debug)]
pub enum Object {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Nil,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Int(v) => write!(f, "{}", v),
            Object::Float(v) => write!(f, "{}", v),
            Object::Bool(v) => write!(f, "{}", v),
            Object::String(v) => write!(f, "{}", v),
            Object::Nil => write!(f, "nil"),
        }
    }
}
