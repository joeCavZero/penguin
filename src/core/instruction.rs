use crate::utils::*;

#[derive(Debug, Clone)]
pub enum PengInstruction {
    PushConst(usize),

    PushLocal(usize),   // ...| ---> ...|v| , v := *<usize>
    StoreLocal(usize),  // ...|v| ---> ...| , *<usize> := v

    PushHeap(PengHeapPtr), // ...| ---> ...|v| , v := *<PengValuePtr>
    PushHeapRef(PengHeapPtr), // ...| ---> ...|ref|
    StoreHeap,   // ...|ref|v ---> ...| , *ref := v

    PushString(PengNamePoolPtr),

    CreateEmptyObject,
    CreateEmptyModule,
    CreateVector(usize), // ...|v0|v1|...|vn| ---> ...|vector|
    CreateSuperType(usize), // creates a new type with usize supers (on stack)
    CreateUnion(usize), // creates a new union based on usize types (on stack)
    CreateTypedObject,

    Convert,

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

    FunctionCall(usize),  // ...|func|p0..pn| ---> ...|ret?|

    TryFunctionCall(usize),  // ...|func|p0..pn| ---> ...|value|bool ok|

    GetIndex,
    SetIndex,

    GetConstAttribute(PengNamePoolPtr),
    SetConstAttribute(PengNamePoolPtr),

    GetConstMember(PengNamePoolPtr),
    SetConstMember(PengNamePoolPtr),

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
            | (Self::PushHeap(left), Self::PushHeap(right))
            | (Self::PushHeapRef(left), Self::PushHeapRef(right))
            | (Self::PushString(left), Self::PushString(right))
            | (Self::CreateSuperType(left), Self::CreateSuperType(right))
            | (Self::CreateVector(left), Self::CreateVector(right))
            | (Self::CreateUnion(left), Self::CreateUnion(right))
            | (Self::GetConstAttribute(left), Self::GetConstAttribute(right))
            | (Self::SetConstAttribute(left), Self::SetConstAttribute(right))
            | (Self::GetConstMember(left), Self::GetConstMember(right))
            | (Self::SetConstMember(left), Self::SetConstMember(right))
            | (Self::FunctionCall(left), Self::FunctionCall(right))
            | (Self::TryFunctionCall(left), Self::TryFunctionCall(right))
            | (Self::Jump(left), Self::Jump(right))
            | (Self::JumpIfTrue(left), Self::JumpIfTrue(right))
            | (Self::JumpIfFalse(left), Self::JumpIfFalse(right)) => left == right,
            _ => std::mem::discriminant(self) == std::mem::discriminant(rhs),
        }
    }
}
