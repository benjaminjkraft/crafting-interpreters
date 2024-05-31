#[derive(Debug)]
pub enum Object {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Nil,
}
