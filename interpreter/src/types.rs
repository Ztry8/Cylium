#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypesCheck {
    String,
    Boolean,
    Number,
    Float,
}

#[derive(Debug, Clone)]
pub enum Types {
    // Vector(Vec<Types>),
    String(String),
    Boolean(bool),
    Number(i64),
    Float(f64),
}
