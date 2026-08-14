use crate::core::*;
use crate::generator::generate_ast_using;
use crate::parser::PengAST;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PengBinaryHeapId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PengBinaryEntryKind {
    Program,
    Script,
}

#[derive(Debug, Clone)]
pub enum PengBinaryRef {
    Heap(PengBinaryHeapId),
    External(String),
}

#[derive(Debug, Clone)]
pub enum PengBinaryCell {
    Nil,
    Int(i64),
    Uint(u64),
    Float32(f32),
    Float64(f64),
    Byte(u8),
    Bool(bool),
    Reference(PengBinaryRef),
}

#[derive(Debug, Clone)]
pub enum PengBinaryBindedCell {
    Mutable(PengBinaryCell),
    Immutable(PengBinaryCell),
}

#[derive(Debug, Clone)]
pub enum PengBinaryType {
    Nil,
    Int,
    Uint,
    Float32,
    Float64,
    Byte,
    Bool,
    String,
    Object,
    Vector(Box<PengBinaryType>),
    Type,
    Module,
    Function,
    Operator,
    Thread,
    Custom(Vec<(String, PengBinaryBindedCell)>),
    Any,
}

#[derive(Debug, Clone)]
pub enum PengBinaryOpcode {
    PushConst(usize),
    MakeImmutable,
    PushLocal(usize),
    ReserveLocal(usize),
    StoreLocal(usize),
    PushHeap(PengBinaryRef),
    PushHeapRef(PengBinaryRef),
    StoreHeap,
    PushString(String),
    CreateEmptyObject,
    CreateEmptyModule,
    CreateVector(usize),
    CreateSuperType(usize),
    CreateTypedObject,
    Convert,
    Duplicate,
    Pop,
    Swap,
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Remainder,
    Negate,
    Concat,
    And,
    Or,
    Not,
    Equals,
    NotEquals,
    GreaterThan,
    GreaterEqualsThan,
    LessThan,
    LessEqualsThan,
    OperationCall,
    TryOperationCall,
    FunctionCall(usize),
    FunctionCallSpread(usize),
    TryFunctionCall(usize),
    TryFunctionCallSpread(usize),
    GetIndex,
    SetIndex,
    GetAttribute(String),
    SetAttribute(String),
    GetMember(String),
    SetMember(String),
    Jump(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),
    Return,
}

#[derive(Debug, Clone)]
pub struct PengBinaryBytecode {
    pub bytecode: Vec<PengBinaryOpcode>,
    pub positions: Vec<PengPosition>,
    pub consts: Vec<PengBinaryValue>,
    pub using_values: Vec<PengBinaryRef>,
}

#[derive(Debug, Clone)]
pub enum PengBinaryValue {
    Cell(PengBinaryCell),
    String(String),
    Object(Vec<(String, PengBinaryBindedCell)>),
    Vector(Vec<PengBinaryBindedCell>),
    Type(PengBinaryType),
    Module(Vec<(String, PengBinaryBindedCell)>),
    Function {
        code: PengBinaryBytecode,
        params: PengBytecodeFunctionParams,
    },
    Operation(PengBinaryBytecode),
}

#[derive(Debug, Clone)]
pub struct PengBinaryHeapValue {
    pub id: PengBinaryHeapId,
    pub value: PengBinaryValue,
}

#[derive(Debug, Clone)]
pub struct PengBinaryGlobal {
    pub name: String,
    pub mutable: bool,
    pub value: PengBinaryRef,
}

#[derive(Debug, Clone)]
pub struct PengBinaryFile {
    pub header: super::PengBinaryHeader,
    pub entry_kind: PengBinaryEntryKind,
    pub heap: Vec<PengBinaryHeapValue>,
    pub globals: Vec<PengBinaryGlobal>,
    pub init: Option<PengBinaryRef>,
}

fn invalid(message: String) -> PengError {
    PengError::InvalidState(format!("Penguin binary: {message}"))
}

struct Collector<'a> {
    env: &'a PengEnv,
    options: &'a super::PengBinaryBuildOptions,
    ids: HashMap<PengHeapPtr, PengBinaryHeapId>,
    external: HashMap<PengHeapPtr, String>,
    values: Vec<Option<PengBinaryValue>>,
}

impl<'a> Collector<'a> {
    fn name(&self, ptr: PengNamePoolPtr) -> Result<String, PengError> {
        match self.env.get_pooled_name(ptr) {
            Some(v) => Ok(v.clone()),
            None => Err(invalid(format!("name pool pointer {} is missing", ptr.0))),
        }
    }
    fn reference(&mut self, ptr: PengHeapPtr) -> Result<PengBinaryRef, PengError> {
        if let Some(name) = self.external.get(&ptr) {
            return Ok(PengBinaryRef::External(name.clone()));
        }
        if let Some(id) = self.ids.get(&ptr) {
            return Ok(PengBinaryRef::Heap(*id));
        }
        let id = PengBinaryHeapId(self.values.len() as u32);
        self.ids.insert(ptr, id);
        self.values.push(None);
        let value = match self.env.get_heap(ptr) {
            Some(v) => v.clone(),
            None => return Err(invalid(format!("heap pointer {} is missing", ptr.0))),
        };
        let encoded = match self.value(&value) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        self.values[id.0 as usize] = Some(encoded);
        Ok(PengBinaryRef::Heap(id))
    }
    fn cell(&mut self, cell: &PengCell) -> Result<PengBinaryCell, PengError> {
        match cell {
            PengCell::Nil => Ok(PengBinaryCell::Nil),
            PengCell::Int(v) => Ok(PengBinaryCell::Int(*v as i64)),
            PengCell::Uint(v) => Ok(PengBinaryCell::Uint(*v as u64)),
            PengCell::Float32(v) => Ok(PengBinaryCell::Float32(*v)),
            PengCell::Float64(v) => Ok(PengBinaryCell::Float64(*v)),
            PengCell::Byte(v) => Ok(PengBinaryCell::Byte(*v)),
            PengCell::Bool(v) => Ok(PengBinaryCell::Bool(*v)),
            PengCell::Reference(v) => match self.reference(*v) {
                Ok(r) => Ok(PengBinaryCell::Reference(r)),
                Err(e) => Err(e),
            },
        }
    }
    fn binded(&mut self, value: &PengBindedCell) -> Result<PengBinaryBindedCell, PengError> {
        match value {
            PengBinded::Mutable(v) => match self.cell(v) {
                Ok(v) => Ok(PengBinaryBindedCell::Mutable(v)),
                Err(e) => Err(e),
            },
            PengBinded::Immutable(v) => match self.cell(v) {
                Ok(v) => Ok(PengBinaryBindedCell::Immutable(v)),
                Err(e) => Err(e),
            },
        }
    }
    fn fields(
        &mut self,
        fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    ) -> Result<Vec<(String, PengBinaryBindedCell)>, PengError> {
        let mut source: Vec<_> = fields.iter().collect();
        source.sort_by_key(|(name, _)| self.env.get_pooled_name(**name).cloned());
        let mut out = Vec::with_capacity(source.len());
        for (name, value) in source {
            let name = match self.name(*name) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            let value = match self.binded(value) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            out.push((name, value));
        }
        Ok(out)
    }
    fn typing(&mut self, value: &PengType) -> Result<PengBinaryType, PengError> {
        match value {
            PengType::Nil => Ok(PengBinaryType::Nil),
            PengType::Int => Ok(PengBinaryType::Int),
            PengType::Uint => Ok(PengBinaryType::Uint),
            PengType::Float32 => Ok(PengBinaryType::Float32),
            PengType::Float64 => Ok(PengBinaryType::Float64),
            PengType::Byte => Ok(PengBinaryType::Byte),
            PengType::Bool => Ok(PengBinaryType::Bool),
            PengType::String => Ok(PengBinaryType::String),
            PengType::Object => Ok(PengBinaryType::Object),
            PengType::Vector(v) => match self.typing(v) {
                Ok(v) => Ok(PengBinaryType::Vector(Box::new(v))),
                Err(e) => Err(e),
            },
            PengType::Type => Ok(PengBinaryType::Type),
            PengType::Module => Ok(PengBinaryType::Module),
            PengType::Function => Ok(PengBinaryType::Function),
            PengType::Operator => Ok(PengBinaryType::Operator),
            PengType::Thread => Ok(PengBinaryType::Thread),
            PengType::Any => Ok(PengBinaryType::Any),
            PengType::Custom(v) => match self.fields(&v.fields) {
                Ok(v) => Ok(PengBinaryType::Custom(v)),
                Err(e) => Err(e),
            },
        }
    }
    fn instruction(&mut self, i: &PengInstruction) -> Result<PengBinaryOpcode, PengError> {
        use PengBinaryOpcode as O;
        Ok(match i {
            PengInstruction::PushConst(v) => O::PushConst(*v),
            PengInstruction::MakeImmutable => O::MakeImmutable,
            PengInstruction::PushLocal(v) => O::PushLocal(*v),
            PengInstruction::ReserveLocal(v) => O::ReserveLocal(*v),
            PengInstruction::StoreLocal(v) => O::StoreLocal(*v),
            PengInstruction::PushHeap(v) => O::PushHeap(match self.reference(*v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
            PengInstruction::PushHeapRef(v) => O::PushHeapRef(match self.reference(*v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
            PengInstruction::StoreHeap => O::StoreHeap,
            PengInstruction::PushString(v) => O::PushString(match self.name(*v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
            PengInstruction::CreateEmptyObject => O::CreateEmptyObject,
            PengInstruction::CreateEmptyModule => O::CreateEmptyModule,
            PengInstruction::CreateVector(v) => O::CreateVector(*v),
            PengInstruction::CreateSuperType(v) => O::CreateSuperType(*v),
            PengInstruction::CreateTypedObject => O::CreateTypedObject,
            PengInstruction::Convert => O::Convert,
            PengInstruction::Duplicate => O::Duplicate,
            PengInstruction::Pop => O::Pop,
            PengInstruction::Swap => O::Swap,
            PengInstruction::Add => O::Add,
            PengInstruction::Subtract => O::Subtract,
            PengInstruction::Multiply => O::Multiply,
            PengInstruction::Divide => O::Divide,
            PengInstruction::Power => O::Power,
            PengInstruction::Remainder => O::Remainder,
            PengInstruction::Negate => O::Negate,
            PengInstruction::Concat => O::Concat,
            PengInstruction::And => O::And,
            PengInstruction::Or => O::Or,
            PengInstruction::Not => O::Not,
            PengInstruction::Equals => O::Equals,
            PengInstruction::NotEquals => O::NotEquals,
            PengInstruction::GreaterThan => O::GreaterThan,
            PengInstruction::GreaterEqualsThan => O::GreaterEqualsThan,
            PengInstruction::LessThan => O::LessThan,
            PengInstruction::LessEqualsThan => O::LessEqualsThan,
            PengInstruction::OperationCall => O::OperationCall,
            PengInstruction::TryOperationCall => O::TryOperationCall,
            PengInstruction::FunctionCall(v) => O::FunctionCall(*v),
            PengInstruction::FunctionCallSpread(v) => O::FunctionCallSpread(*v),
            PengInstruction::TryFunctionCall(v) => O::TryFunctionCall(*v),
            PengInstruction::TryFunctionCallSpread(v) => O::TryFunctionCallSpread(*v),
            PengInstruction::GetIndex => O::GetIndex,
            PengInstruction::SetIndex => O::SetIndex,
            PengInstruction::GetAttribute(v) => O::GetAttribute(match self.name(*v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
            PengInstruction::SetAttribute(v) => O::SetAttribute(match self.name(*v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
            PengInstruction::GetMember(v) => O::GetMember(match self.name(*v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
            PengInstruction::SetMember(v) => O::SetMember(match self.name(*v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
            PengInstruction::Jump(v) => O::Jump(*v),
            PengInstruction::JumpIfTrue(v) => O::JumpIfTrue(*v),
            PengInstruction::JumpIfFalse(v) => O::JumpIfFalse(*v),
            PengInstruction::Return => O::Return,
        })
    }
    fn code(
        &mut self,
        code: &[PengInstruction],
        positions: &[PengPosition],
        consts: &[PengValue],
        using: &[PengHeapPtr],
    ) -> Result<PengBinaryBytecode, PengError> {
        let mut bytecode = Vec::with_capacity(code.len());
        for i in code {
            bytecode.push(match self.instruction(i) {
                Ok(v) => v,
                Err(e) => return Err(e),
            });
        }
        let mut constants = Vec::with_capacity(consts.len());
        for v in consts {
            constants.push(match self.value(v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            });
        }
        let mut using_values = Vec::with_capacity(using.len());
        for v in using {
            using_values.push(match self.reference(*v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            });
        }
        Ok(PengBinaryBytecode {
            bytecode,
            positions: if self.options.strip_positions {
                Vec::new()
            } else {
                positions.to_vec()
            },
            consts: constants,
            using_values,
        })
    }
    fn value(&mut self, value: &PengValue) -> Result<PengBinaryValue, PengError> {
        match value {
            PengValue::Cell(v) => match self.cell(v) {
                Ok(v) => Ok(PengBinaryValue::Cell(v)),
                Err(e) => Err(e),
            },
            PengValue::Box(PengBox::String(v)) => Ok(PengBinaryValue::String(v.clone())),
            PengValue::Box(PengBox::Object(v)) => match self.fields(&v.fields) {
                Ok(v) => Ok(PengBinaryValue::Object(v)),
                Err(e) => Err(e),
            },
            PengValue::Box(PengBox::Vector(v)) => {
                let mut out = Vec::new();
                for x in &v.values {
                    out.push(match self.binded(x) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    });
                }
                Ok(PengBinaryValue::Vector(out))
            }
            PengValue::Box(PengBox::Type(v)) => match self.typing(v) {
                Ok(v) => Ok(PengBinaryValue::Type(v)),
                Err(e) => Err(e),
            },
            PengValue::Box(PengBox::Module(v)) => match self.fields(&v.members) {
                Ok(v) => Ok(PengBinaryValue::Module(v)),
                Err(e) => Err(e),
            },
            PengValue::Box(PengBox::Function(PengFunction::Bytecode(v))) => {
                match self.code(&v.bytecode, &v.positions, &v.consts, &v.using_values) {
                    Ok(code) => Ok(PengBinaryValue::Function {
                        code,
                        params: v.params.clone(),
                    }),
                    Err(e) => Err(e),
                }
            }
            PengValue::Box(PengBox::Function(PengFunction::Native(_))) => Err(invalid(
                "native function is not an external named value".to_string(),
            )),
            PengValue::Box(PengBox::Operation(PengOperation::Bytecode(v))) => {
                match self.code(&v.bytecode, &v.positions, &v.consts, &v.using_values) {
                    Ok(v) => Ok(PengBinaryValue::Operation(v)),
                    Err(e) => Err(e),
                }
            }
            PengValue::Box(PengBox::Operation(PengOperation::Native(_))) => Err(invalid(
                "native operation is not an external named value".to_string(),
            )),
            PengValue::Box(PengBox::Thread(_)) => {
                Err(invalid("runtime threads cannot be serialized".to_string()))
            }
        }
    }
}

pub fn peng_binary_from_unit(
    env: &PengEnv,
    unit: &PengUnit,
    options: &super::PengBinaryBuildOptions,
) -> Result<PengBinaryFile, PengError> {
    peng_binary_from_unit_kind(env, unit, PengBinaryEntryKind::Program, options)
}

pub fn peng_binary_compile_ast_using_env(
    env: &mut PengEnv,
    ast: &PengAST,
    using_unit: &PengUnit,
    options: &super::PengBinaryBuildOptions,
) -> Result<PengBinaryFile, PengError> {
    let kind = match ast {
        PengAST::Script(_) => PengBinaryEntryKind::Script,
        PengAST::Program(_) => PengBinaryEntryKind::Program,
    };
    let unit = match generate_ast_using(env, ast, using_unit) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    peng_binary_from_unit_using(env, &unit, using_unit, kind, options)
}

pub fn peng_binary_from_unit_kind(
    env: &PengEnv,
    unit: &PengUnit,
    entry_kind: PengBinaryEntryKind,
    options: &super::PengBinaryBuildOptions,
) -> Result<PengBinaryFile, PengError> {
    let using = PengUnit::library();
    peng_binary_from_unit_using(env, unit, &using, entry_kind, options)
}

pub fn peng_binary_from_unit_using(
    env: &PengEnv,
    unit: &PengUnit,
    using_unit: &PengUnit,
    entry_kind: PengBinaryEntryKind,
    options: &super::PengBinaryBuildOptions,
) -> Result<PengBinaryFile, PengError> {
    let mut external = HashMap::new();
    for (name, value) in using_unit.globals() {
        if let Some(text) = env.get_pooled_name(*name) {
            external.insert(*value.value(), text.clone());
        }
    }
    for (name, value) in unit.globals() {
        let is_native = matches!(
            env.get_heap(*value.value()),
            Some(PengValue::Box(PengBox::Function(PengFunction::Native(_))))
                | Some(PengValue::Box(PengBox::Operation(PengOperation::Native(_))))
        );
        if is_native {
            if let Some(text) = env.get_pooled_name(*name) {
                external.insert(*value.value(), text.clone());
            }
        }
    }
    let mut c = Collector {
        env,
        options,
        ids: HashMap::new(),
        external,
        values: Vec::new(),
    };
    let mut source: Vec<_> = unit.globals().iter().collect();
    source.sort_by_key(|(n, _)| env.get_pooled_name(**n).cloned());
    let mut globals = Vec::new();
    for (name, value) in source {
        let name = match c.name(*name) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let reference = match c.reference(*value.value()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        globals.push(PengBinaryGlobal {
            name,
            mutable: matches!(value, PengBinded::Mutable(_)),
            value: reference,
        });
    }
    let init = match unit.init() {
        Some(v) => Some(match c.reference(v) {
            Ok(v) => v,
            Err(e) => return Err(e),
        }),
        None => None,
    };
    let mut heap = Vec::new();
    for (index, value) in c.values.into_iter().enumerate() {
        match value {
            Some(v) => heap.push(PengBinaryHeapValue {
                id: PengBinaryHeapId(index as u32),
                value: v,
            }),
            None => return Err(invalid("incomplete heap graph".to_string())),
        }
    }
    Ok(PengBinaryFile {
        header: super::PengBinaryHeader::new(),
        entry_kind,
        heap,
        globals,
        init,
    })
}

struct Loader<'a> {
    env: &'a mut PengEnv,
    ptrs: Vec<PengHeapPtr>,
    using: &'a PengUnit,
}
impl<'a> Loader<'a> {
    fn reference(&mut self, r: &PengBinaryRef) -> Result<PengHeapPtr, PengError> {
        match r {
            PengBinaryRef::Heap(id) => match self.ptrs.get(id.0 as usize) {
                Some(v) => Ok(*v),
                None => Err(invalid("heap id out of range".to_string())),
            },
            PengBinaryRef::External(name) => {
                let n = self.env.ensure_pooled_name_ptr(name.clone());
                match self.using.get_global_ptr(n) {
                    Some(v) => Ok(v),
                    None => Err(invalid(format!("unresolved external global: {name}"))),
                }
            }
        }
    }
    fn cell(&mut self, v: &PengBinaryCell) -> Result<PengCell, PengError> {
        match v {
            PengBinaryCell::Nil => Ok(PengCell::Nil),
            PengBinaryCell::Int(v) => match isize::try_from(*v) {
                Ok(v) => Ok(PengCell::Int(v)),
                Err(_) => Err(invalid("int does not fit this architecture".to_string())),
            },
            PengBinaryCell::Uint(v) => match usize::try_from(*v) {
                Ok(v) => Ok(PengCell::Uint(v)),
                Err(_) => Err(invalid("uint does not fit this architecture".to_string())),
            },
            PengBinaryCell::Float32(v) => Ok(PengCell::Float32(*v)),
            PengBinaryCell::Float64(v) => Ok(PengCell::Float64(*v)),
            PengBinaryCell::Byte(v) => Ok(PengCell::Byte(*v)),
            PengBinaryCell::Bool(v) => Ok(PengCell::Bool(*v)),
            PengBinaryCell::Reference(v) => match self.reference(v) {
                Ok(v) => Ok(PengCell::Reference(v)),
                Err(e) => Err(e),
            },
        }
    }
    fn binded(&mut self, v: &PengBinaryBindedCell) -> Result<PengBindedCell, PengError> {
        match v {
            PengBinaryBindedCell::Mutable(v) => match self.cell(v) {
                Ok(v) => Ok(PengBinded::Mutable(v)),
                Err(e) => Err(e),
            },
            PengBinaryBindedCell::Immutable(v) => match self.cell(v) {
                Ok(v) => Ok(PengBinded::Immutable(v)),
                Err(e) => Err(e),
            },
        }
    }
    fn fields(
        &mut self,
        v: &[(String, PengBinaryBindedCell)],
    ) -> Result<HashMap<PengNamePoolPtr, PengBindedCell>, PengError> {
        let mut out = HashMap::new();
        for (name, value) in v {
            let n = self.env.ensure_pooled_name_ptr(name.clone());
            let value = match self.binded(value) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            out.insert(n, value);
        }
        Ok(out)
    }
    fn typing(&mut self, v: &PengBinaryType) -> Result<PengType, PengError> {
        Ok(match v {
            PengBinaryType::Nil => PengType::Nil,
            PengBinaryType::Int => PengType::Int,
            PengBinaryType::Uint => PengType::Uint,
            PengBinaryType::Float32 => PengType::Float32,
            PengBinaryType::Float64 => PengType::Float64,
            PengBinaryType::Byte => PengType::Byte,
            PengBinaryType::Bool => PengType::Bool,
            PengBinaryType::String => PengType::String,
            PengBinaryType::Object => PengType::Object,
            PengBinaryType::Vector(v) => PengType::Vector(Box::new(match self.typing(v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            })),
            PengBinaryType::Type => PengType::Type,
            PengBinaryType::Module => PengType::Module,
            PengBinaryType::Function => PengType::Function,
            PengBinaryType::Operator => PengType::Operator,
            PengBinaryType::Thread => PengType::Thread,
            PengBinaryType::Any => PengType::Any,
            PengBinaryType::Custom(v) => PengType::Custom(PengCustomType {
                fields: match self.fields(v) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                },
            }),
        })
    }
    fn instruction(&mut self, i: &PengBinaryOpcode) -> Result<PengInstruction, PengError> {
        use PengBinaryOpcode as O;
        Ok(match i {
            O::PushConst(v) => PengInstruction::PushConst(*v),
            O::MakeImmutable => PengInstruction::MakeImmutable,
            O::PushLocal(v) => PengInstruction::PushLocal(*v),
            O::ReserveLocal(v) => PengInstruction::ReserveLocal(*v),
            O::StoreLocal(v) => PengInstruction::StoreLocal(*v),
            O::PushHeap(v) => PengInstruction::PushHeap(match self.reference(v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
            O::PushHeapRef(v) => PengInstruction::PushHeapRef(match self.reference(v) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
            O::StoreHeap => PengInstruction::StoreHeap,
            O::PushString(v) => {
                PengInstruction::PushString(self.env.ensure_pooled_name_ptr(v.clone()))
            }
            O::CreateEmptyObject => PengInstruction::CreateEmptyObject,
            O::CreateEmptyModule => PengInstruction::CreateEmptyModule,
            O::CreateVector(v) => PengInstruction::CreateVector(*v),
            O::CreateSuperType(v) => PengInstruction::CreateSuperType(*v),
            O::CreateTypedObject => PengInstruction::CreateTypedObject,
            O::Convert => PengInstruction::Convert,
            O::Duplicate => PengInstruction::Duplicate,
            O::Pop => PengInstruction::Pop,
            O::Swap => PengInstruction::Swap,
            O::Add => PengInstruction::Add,
            O::Subtract => PengInstruction::Subtract,
            O::Multiply => PengInstruction::Multiply,
            O::Divide => PengInstruction::Divide,
            O::Power => PengInstruction::Power,
            O::Remainder => PengInstruction::Remainder,
            O::Negate => PengInstruction::Negate,
            O::Concat => PengInstruction::Concat,
            O::And => PengInstruction::And,
            O::Or => PengInstruction::Or,
            O::Not => PengInstruction::Not,
            O::Equals => PengInstruction::Equals,
            O::NotEquals => PengInstruction::NotEquals,
            O::GreaterThan => PengInstruction::GreaterThan,
            O::GreaterEqualsThan => PengInstruction::GreaterEqualsThan,
            O::LessThan => PengInstruction::LessThan,
            O::LessEqualsThan => PengInstruction::LessEqualsThan,
            O::OperationCall => PengInstruction::OperationCall,
            O::TryOperationCall => PengInstruction::TryOperationCall,
            O::FunctionCall(v) => PengInstruction::FunctionCall(*v),
            O::FunctionCallSpread(v) => PengInstruction::FunctionCallSpread(*v),
            O::TryFunctionCall(v) => PengInstruction::TryFunctionCall(*v),
            O::TryFunctionCallSpread(v) => PengInstruction::TryFunctionCallSpread(*v),
            O::GetIndex => PengInstruction::GetIndex,
            O::SetIndex => PengInstruction::SetIndex,
            O::GetAttribute(v) => {
                PengInstruction::GetAttribute(self.env.ensure_pooled_name_ptr(v.clone()))
            }
            O::SetAttribute(v) => {
                PengInstruction::SetAttribute(self.env.ensure_pooled_name_ptr(v.clone()))
            }
            O::GetMember(v) => {
                PengInstruction::GetMember(self.env.ensure_pooled_name_ptr(v.clone()))
            }
            O::SetMember(v) => {
                PengInstruction::SetMember(self.env.ensure_pooled_name_ptr(v.clone()))
            }
            O::Jump(v) => PengInstruction::Jump(*v),
            O::JumpIfTrue(v) => PengInstruction::JumpIfTrue(*v),
            O::JumpIfFalse(v) => PengInstruction::JumpIfFalse(*v),
            O::Return => PengInstruction::Return,
        })
    }
    fn code(
        &mut self,
        v: &PengBinaryBytecode,
    ) -> Result<
        (
            Vec<PengInstruction>,
            Vec<PengPosition>,
            Vec<PengValue>,
            Vec<PengHeapPtr>,
        ),
        PengError,
    > {
        let mut code = Vec::new();
        for i in &v.bytecode {
            code.push(match self.instruction(i) {
                Ok(v) => v,
                Err(e) => return Err(e),
            })
        }
        let mut consts = Vec::new();
        for x in &v.consts {
            consts.push(match self.value(x) {
                Ok(v) => v,
                Err(e) => return Err(e),
            })
        }
        let mut using = Vec::new();
        for x in &v.using_values {
            using.push(match self.reference(x) {
                Ok(v) => v,
                Err(e) => return Err(e),
            })
        }
        let positions = if v.positions.is_empty() {
            vec![PengPosition::new(0, 0, None); code.len()]
        } else {
            v.positions.clone()
        };
        Ok((code, positions, consts, using))
    }
    fn value(&mut self, v: &PengBinaryValue) -> Result<PengValue, PengError> {
        match v {
            PengBinaryValue::Cell(v) => match self.cell(v) {
                Ok(v) => Ok(PengValue::Cell(v)),
                Err(e) => Err(e),
            },
            PengBinaryValue::String(v) => Ok(PengValue::Box(PengBox::String(v.clone()))),
            PengBinaryValue::Object(v) => match self.fields(v) {
                Ok(v) => Ok(PengValue::Box(PengBox::Object(PengObject { fields: v }))),
                Err(e) => Err(e),
            },
            PengBinaryValue::Module(v) => match self.fields(v) {
                Ok(v) => Ok(PengValue::Box(PengBox::Module(PengModule { members: v }))),
                Err(e) => Err(e),
            },
            PengBinaryValue::Vector(v) => {
                let mut out = Vec::new();
                for x in v {
                    out.push(match self.binded(x) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    })
                }
                Ok(PengValue::Box(PengBox::Vector(PengVector { values: out })))
            }
            PengBinaryValue::Type(v) => match self.typing(v) {
                Ok(v) => Ok(PengValue::Box(PengBox::Type(v))),
                Err(e) => Err(e),
            },
            PengBinaryValue::Function { code, params } => {
                let (code, positions, consts, using_values) = match self.code(code) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                Ok(PengValue::Box(PengBox::Function(PengFunction::Bytecode(
                    PengBytecodeFunction {
                        bytecode: code,
                        positions,
                        consts,
                        using_values,
                        params: params.clone(),
                    },
                ))))
            }
            PengBinaryValue::Operation(code) => {
                let (bytecode, positions, consts, using_values) = match self.code(code) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                Ok(PengValue::Box(PengBox::Operation(PengOperation::Bytecode(
                    PengBytecodeOperation {
                        bytecode,
                        positions,
                        consts,
                        using_values,
                    },
                ))))
            }
        }
    }
}

pub fn peng_binary_to_unit(
    env: &mut PengEnv,
    file: &PengBinaryFile,
    using_unit: &PengUnit,
) -> Result<PengUnit, PengError> {
    match super::peng_binary_validate_file(file) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let mut ptrs = Vec::with_capacity(file.heap.len());

    for _ in &file.heap {
        ptrs.push(env.create_heap_value(PengValue::Cell(PengCell::Nil)));
    }

    let mut loader = Loader {
        env,
        ptrs,
        using: using_unit,
    };

    for item in &file.heap {
        let value = match loader.value(&item.value) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let ptr = loader.ptrs[item.id.0 as usize];

        match loader.env.assign_heap(ptr, value) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    let mut globals = HashMap::new();

    for g in &file.globals {
        let name = loader.env.ensure_pooled_name_ptr(g.name.clone());

        let ptr = match loader.reference(&g.value) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        globals.insert(
            name,
            if g.mutable {
                PengBinded::Mutable(ptr)
            } else {
                PengBinded::Immutable(ptr)
            },
        );
    }

    let init = match &file.init {
        Some(v) => Some(match loader.reference(v) {
            Ok(v) => v,
            Err(e) => return Err(e),
        }),

        None => None,
    };

    match init {
        Some(v) => Ok(PengUnit::new(
            v,
            globals,
            HashMap::new(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )),

        None => {
            let mut unit = PengUnit::library();

            for (n, v) in globals {
                unit.insert_global(n, v);
            }

            Ok(unit)
        }
    }
}
