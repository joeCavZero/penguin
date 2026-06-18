#[derive(Debug, Clone)]
pub enum PengType {
    Nil,
    Int,
    Uint,
    Float32,
    Float64,
    Byte,
    Bool,
    String,

    Object,
    Vector,
    Type,
    Module,

    Function,
    Operator,
    Thread,

    Any,

    Union(Vec<PengType>),
}