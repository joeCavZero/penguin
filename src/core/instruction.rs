use crate::utils::*;

#[derive(Debug, Clone)]
pub enum PengInstruction {
    PushConst(usize),

    PushLocal(usize),   // ...| ---> ...|v| , v := *<usize>
    StoreLocal(usize),  // ...|v| ---> ...| , *<usize> := v

    StoreValue,   // ...|ref|v ---> ...| , *ref := v

    PushString(PengNamePoolPtr),

    CreateObjectType,   // ...|t| ---> ...|tobj| , create obj of type with default fields

    Convert,    //
    CheckType,

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

    Call {  // ...|func|g0..gn|p0..pn| ---> ...|ret?|
        generics: usize,
        params: usize,
    },

    GetIndex,       // ...|vec|index| ---> ...|val|
    GetIndexRef,    // ...|vec|index| ---> ...|val ref|
    
    GetConstAttribute(PengNamePoolPtr),      // ...|obj| ---> ...|val
    GetConstAttributeRef(PengNamePoolPtr),   // ...|obj| ---> ...|val ref|
    GetConstMember(PengNamePoolPtr),     // ...|mod| ---> ...|member|
    GetConstMemberRef(PengNamePoolPtr),  // ...|mod| ---> ...|member ref|
    
    GetAttribute,     // ...|obj|attr| ---> ...|val
    GetAttributeRef,  // ...|obj|attr| ---> ...|val ref|

    Jump(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),

    Return, // ...|v| ---> returns v to frame
    // raise is rust side function
}