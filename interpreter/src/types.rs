#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypesCheck {
    String,
    Boolean,
    Number,
    Float,
}

#[derive(Debug, Clone)]
pub enum Types {
    Scalar(Scalar),
    String(String),
    Void,
}

#[derive(Debug, Clone, Copy)]
pub enum Scalar {
    Boolean(bool),
    Number(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnType {
    Void,
    String,
    Boolean,
    Number,
    Float,
}
