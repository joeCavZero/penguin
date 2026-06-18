use crate::vector::*;
use crate::object::*;
use crate::module::*;
use crate::typing::*;
use crate::thread::*;
use crate::function::*;

#[derive(Debug, Clone)]
pub enum PengValue {
    Nil,
    Int(isize),
    Uint(usize),
    Float32(f32),
    Float64(f64),
    Byte(u8),
    Bool(bool),
    String(String),
    Object(PengObject),
    Vector(PengVector),
    Type(PengType),
    Module(PengModule),
    Thread(PengThread),
    Function(PengFunction),
}
