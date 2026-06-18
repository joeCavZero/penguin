use crate::cell::*;
use crate::utils::*;
use crate::typing::*;

#[derive(Debug, Clone)]
pub enum PengInstruction {
    PushGeneric(usize),
    PushEnvParam(usize),
    PushParam(usize),

    PushCell(PengCell), 
    PushString(PengNamePoolPtr),    

    CreateObjectType(PengValuePtr), // create obj of type with default fields

    Convert(PengType), // only uses int, uint, f32, f64, byte, bool string

    Duplicate(usize), // duplicate <usize> cells from top
    Pop(usize), // pops <usize> cells


    Add,    // ...|v1|v2| ---> ...|v1+v2|
    Subtract,
    Multiply,
    Divide,
    Power,
    Remainder,
    Negate, // ...|v| ---> ...|-v|

    Concat, // ...|s1|s2| ---> ...|s1..s2|

    And,
    Or,

    Not,    // ...|true| ---> ...|false|
    Equals, // ...|v1|v2| ---> ...| v1==v2 |
    NotEquals,  // ...|v1|v2| ---> ...| v1!=v2 |
    GreaterThan,
    GreaterEqualsThan,
    LessThan,
    LessEqualsThan,

    Call {  // ...|func| ---> calls func x generics, y env_params, z params
        generics: usize,
        env_params: usize,
        params: usize,
    },

    GetIndex,   // ...|vec|index| ---> ...|get|
    SetIndex,   // ...|vec|index|v| ---> ...|
    
    GetAttribute,   // ...|obj|attr| ---> ...|val
    SetAttribute,   // ...|obj|attr|val| ---> ...|

    GetMember(PengNamePoolPtr), // ...|mod| ---> ...|member|
    SetMember(PengNamePoolPtr), // ...|mod|member| ---> ...|

    Jump(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),

    Return, // ...|v| ---> returns v to frame
    // raise is rust side function
}