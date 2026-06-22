use crate::utils::*;

#[derive(Debug, Clone)]
pub enum PengInstruction {
    PushConst(usize),

    PushLocal(usize),   // ...| ---> ...|v| , v := *<usize>
    StoreLocal(usize),  // ...|v| ---> ...| , *<usize> := v

    PushValue(PengValuePtr), // ...| ---> ...|v| , v := *<PengValuePtr>
    PushValueRef(PengValuePtr), // ...| ---> ...|ref|
    StoreValue,   // ...|ref|v ---> ...| , *ref := v

    PushString(PengNamePoolPtr),

    CreateObjectType,   // ...|t| ---> ...|tobj| , create obj of type with default fields
    CreateSuperType(usize), // creates a new type with usize supers (on stack)
    CreateUnion(usize), // creates a new union based on usize types (on stack)

    Convert,
    CheckType,

    Duplicate, // duplicate
    Pop, // pop

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

    OperationCall,

    FunctionCall {  // ...|func|g0..gn|p0..pn| ---> ...|ret?|
        generics: usize,
        params: usize,
    },

    TryFunctionCall { // ...|func|g0..gn|p0..pn| ---> ...|value|bool ok|
        generics: usize,
        params: usize,
    },

    GetIndex,       // ...|vec|index| ---> ...|val|
    GetIndexRef,    // ...|vec|index| ---> ...|val ref|
    
    GetConstAttribute(PengNamePoolPtr),      // ...|obj| ---> ...|val
    GetConstAttributeRef(PengNamePoolPtr),   // ...|obj| ---> ...|val ref|
    GetConstMember(PengNamePoolPtr),     // ...|mod| ---> ...|member|
    GetConstMemberRef(PengNamePoolPtr),  // ...|mod| ---> ...|member ref|

    Jump(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),

    Return, // ...|v| ---> returns v to frame
    // raise is rust side function
}

impl PengInstruction {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::PushConst(left), Self::PushConst(right))
            | (Self::PushLocal(left), Self::PushLocal(right))
            | (Self::StoreLocal(left), Self::StoreLocal(right))
            | (Self::PushValue(left), Self::PushValue(right))
            | (Self::PushValueRef(left), Self::PushValueRef(right))
            | (Self::PushString(left), Self::PushString(right))
            | (Self::CreateSuperType(left), Self::CreateSuperType(right))
            | (Self::CreateUnion(left), Self::CreateUnion(right))
            | (Self::GetConstAttribute(left), Self::GetConstAttribute(right))
            | (Self::GetConstAttributeRef(left), Self::GetConstAttributeRef(right))
            | (Self::GetConstMember(left), Self::GetConstMember(right))
            | (Self::GetConstMemberRef(left), Self::GetConstMemberRef(right))
            | (Self::Jump(left), Self::Jump(right))
            | (Self::JumpIfTrue(left), Self::JumpIfTrue(right))
            | (Self::JumpIfFalse(left), Self::JumpIfFalse(right)) => left == right,
            (
                Self::FunctionCall {
                    generics: left_generics,
                    params: left_params,
                },
                Self::FunctionCall {
                    generics: right_generics,
                    params: right_params,
                },
            )
            | (
                Self::TryFunctionCall {
                    generics: left_generics,
                    params: left_params,
                },
                Self::TryFunctionCall {
                    generics: right_generics,
                    params: right_params,
                },
            ) => left_generics == right_generics && left_params == right_params,
            _ => std::mem::discriminant(self) == std::mem::discriminant(rhs),
        }
    }
}
