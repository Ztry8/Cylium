#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypesCheck {
    String,
    Boolean,
    Int,
    Float,
    Array(Box<TypesCheck>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnType {
    Void,
    String,
    Boolean,
    Int,
    Float,
    Array(Box<ReturnType>),
}
