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
}

#[derive(Debug, Clone, Copy)]
pub enum Scalar {
    Boolean(bool),
    Number(i64),
    Float(f64),
}
