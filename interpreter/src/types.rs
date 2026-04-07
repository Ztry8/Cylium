#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypesCheck {
    String,
    Boolean,
    Number,
    Float,
    Array(Box<TypesCheck>),
}

#[derive(Debug, Clone)]
pub enum Types {
    Scalar(Scalar),
    String(String),
    Array(Vec<Scalar>),
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
    Array(Box<ReturnType>),
}
