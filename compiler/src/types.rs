#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypesCheck {
    String,
    Boolean,
    Int,
    Float,
    Struct(String),
    Array(Box<TypesCheck>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnType {
    Void,
    String,
    Boolean,
    Int,
    Float,
    Struct(String),
    Array(Box<ReturnType>),
}
