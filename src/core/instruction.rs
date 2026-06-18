use crate::utils::*;

#[derive(Debug, Clone)]
pub enum PengInstruction {
    PushGeneric(usize),
    PushEnvParam(usize),
    PushParam(usize),
    CreateObject,
    CreateObjectType(PengValuePtr),
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Remainder,
    Negate,
}