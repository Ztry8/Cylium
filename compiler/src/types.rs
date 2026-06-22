#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypesCheck {
    String,
    Boolean,
    Number,
    Float,
    Array(Box<TypesCheck>),
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
