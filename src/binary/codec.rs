use super::*;
use crate::core::*;

fn binary_invalid_state_error(s: &str) -> PengError {
    PengError::InvalidState(format!("invalid Penguin binary: {s}"))
}
struct W {
    b: Vec<u8>,
}
impl W {
    fn u8(&mut self, v: u8) {
        self.b.push(v)
    }
    fn u16(&mut self, v: u16) {
        self.b.extend(v.to_le_bytes())
    }
    fn u32(&mut self, v: u32) {
        self.b.extend(v.to_le_bytes())
    }
    fn u64(&mut self, v: u64) {
        self.b.extend(v.to_le_bytes())
    }
    fn text(&mut self, v: &str) {
        self.u64(v.len() as u64);
        self.b.extend(v.as_bytes())
    }
    fn len(&mut self, v: usize) {
        self.u64(v as u64)
    }
}
struct R<'a> {
    b: &'a [u8],
    p: usize,
}
impl<'a> R<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], PengError> {
        if n > self.b.len().saturating_sub(self.p) {
            return Err(binary_invalid_state_error("unexpected end of file"));
        }
        let v = &self.b[self.p..self.p + n];
        self.p += n;
        Ok(v)
    }
    fn u8(&mut self) -> Result<u8, PengError> {
        match self.take(1) {
            Ok(v) => Ok(v[0]),
            Err(e) => Err(e),
        }
    }
    fn u16(&mut self) -> Result<u16, PengError> {
        match self.take(2) {
            Ok(v) => Ok(u16::from_le_bytes([v[0], v[1]])),
            Err(e) => Err(e),
        }
    }
    fn u32(&mut self) -> Result<u32, PengError> {
        match self.take(4) {
            Ok(v) => Ok(u32::from_le_bytes([v[0], v[1], v[2], v[3]])),
            Err(e) => Err(e),
        }
    }
    fn u64(&mut self) -> Result<u64, PengError> {
        match self.take(8) {
            Ok(v) => Ok(u64::from_le_bytes([
                v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7],
            ])),
            Err(e) => Err(e),
        }
    }
    fn len(&mut self) -> Result<usize, PengError> {
        match self.u64() {
            Ok(v) => match usize::try_from(v) {
                Ok(v) => Ok(v),
                Err(_) => Err(binary_invalid_state_error("length exceeds architecture")),
            },
            Err(e) => Err(e),
        }
    }
    fn text(&mut self) -> Result<String, PengError> {
        let n = match self.len() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        match self.take(n) {
            Ok(v) => match std::str::from_utf8(v) {
                Ok(v) => Ok(v.to_string()),
                Err(_) => Err(binary_invalid_state_error("invalid UTF-8")),
            },
            Err(e) => Err(e),
        }
    }
}

fn wr_ref(w: &mut W, v: &PengBinaryRef) {
    match v {
        PengBinaryRef::Heap(v) => {
            w.u8(0);
            w.u32(v.0)
        }
        PengBinaryRef::External(v) => {
            w.u8(1);
            w.text(v)
        }
    }
}
fn rd_ref(r: &mut R) -> Result<PengBinaryRef, PengError> {
    match r.u8() {
        Ok(0) => match r.u32() {
            Ok(v) => Ok(PengBinaryRef::Heap(PengBinaryHeapId(v))),
            Err(e) => Err(e),
        },
        Ok(1) => match r.text() {
            Ok(v) => Ok(PengBinaryRef::External(v)),
            Err(e) => Err(e),
        },
        Ok(_) => Err(binary_invalid_state_error("invalid reference tag")),
        Err(e) => Err(e),
    }
}
fn wr_cell(w: &mut W, v: &PengBinaryCell) {
    match v {
        PengBinaryCell::Nil => w.u8(0),
        PengBinaryCell::Int(v) => {
            w.u8(1);
            w.u64(*v as u64)
        }
        PengBinaryCell::Uint(v) => {
            w.u8(2);
            w.u64(*v)
        }
        PengBinaryCell::Float32(v) => {
            w.u8(3);
            w.u32(v.to_bits())
        }
        PengBinaryCell::Float64(v) => {
            w.u8(4);
            w.u64(v.to_bits())
        }
        PengBinaryCell::Byte(v) => {
            w.u8(5);
            w.u8(*v)
        }
        PengBinaryCell::Bool(v) => {
            w.u8(6);
            w.u8(*v as u8)
        }
        PengBinaryCell::Reference(v) => {
            w.u8(7);
            wr_ref(w, v)
        }
    }
}
fn rd_cell(r: &mut R) -> Result<PengBinaryCell, PengError> {
    match r.u8() {
        Ok(0) => Ok(PengBinaryCell::Nil),
        Ok(1) => match r.u64() {
            Ok(v) => Ok(PengBinaryCell::Int(v as i64)),
            Err(e) => Err(e),
        },
        Ok(2) => match r.u64() {
            Ok(v) => Ok(PengBinaryCell::Uint(v)),
            Err(e) => Err(e),
        },
        Ok(3) => match r.u32() {
            Ok(v) => Ok(PengBinaryCell::Float32(f32::from_bits(v))),
            Err(e) => Err(e),
        },
        Ok(4) => match r.u64() {
            Ok(v) => Ok(PengBinaryCell::Float64(f64::from_bits(v))),
            Err(e) => Err(e),
        },
        Ok(5) => match r.u8() {
            Ok(v) => Ok(PengBinaryCell::Byte(v)),
            Err(e) => Err(e),
        },
        Ok(6) => match r.u8() {
            Ok(0) => Ok(PengBinaryCell::Bool(false)),
            Ok(1) => Ok(PengBinaryCell::Bool(true)),
            Ok(_) => Err(binary_invalid_state_error("invalid bool")),
            Err(e) => Err(e),
        },
        Ok(7) => match rd_ref(r) {
            Ok(v) => Ok(PengBinaryCell::Reference(v)),
            Err(e) => Err(e),
        },
        Ok(_) => Err(binary_invalid_state_error("invalid cell tag")),
        Err(e) => Err(e),
    }
}
fn wr_bound(w: &mut W, v: &PengBinaryBindedCell) {
    match v {
        PengBinaryBindedCell::Mutable(v) => {
            w.u8(0);
            wr_cell(w, v)
        }
        PengBinaryBindedCell::Immutable(v) => {
            w.u8(1);
            wr_cell(w, v)
        }
    }
}
fn rd_bound(r: &mut R) -> Result<PengBinaryBindedCell, PengError> {
    let t = match r.u8() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let v = match rd_cell(r) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    match t {
        0 => Ok(PengBinaryBindedCell::Mutable(v)),
        1 => Ok(PengBinaryBindedCell::Immutable(v)),
        _ => Err(binary_invalid_state_error("invalid binding tag")),
    }
}
fn wr_fields(w: &mut W, v: &[(String, PengBinaryBindedCell)]) {
    w.len(v.len());
    for (n, x) in v {
        w.text(n);
        wr_bound(w, x)
    }
}
fn rd_fields(r: &mut R) -> Result<Vec<(String, PengBinaryBindedCell)>, PengError> {
    let n = match r.len() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut o = Vec::with_capacity(n);
    for _ in 0..n {
        let k = match r.text() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let v = match rd_bound(r) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        o.push((k, v))
    }
    Ok(o)
}
fn wr_type(w: &mut W, v: &PengBinaryType) {
    let t = match v {
        PengBinaryType::Nil => 0,
        PengBinaryType::Int => 1,
        PengBinaryType::Uint => 2,
        PengBinaryType::Float32 => 3,
        PengBinaryType::Float64 => 4,
        PengBinaryType::Byte => 5,
        PengBinaryType::Bool => 6,
        PengBinaryType::String => 7,
        PengBinaryType::Object => 8,
        PengBinaryType::Vector(_) => 9,
        PengBinaryType::Type => 10,
        PengBinaryType::Module => 11,
        PengBinaryType::Function => 12,
        PengBinaryType::Operator => 13,
        PengBinaryType::Thread => 14,
        PengBinaryType::Custom(_) => 15,
        PengBinaryType::Any => 16,
    };
    w.u8(t);
    match v {
        PengBinaryType::Vector(x) => wr_type(w, x),
        PengBinaryType::Custom(x) => wr_fields(w, x),
        _ => {}
    }
}
fn rd_type(r: &mut R) -> Result<PengBinaryType, PengError> {
    match r.u8() {
        Ok(0) => Ok(PengBinaryType::Nil),
        Ok(1) => Ok(PengBinaryType::Int),
        Ok(2) => Ok(PengBinaryType::Uint),
        Ok(3) => Ok(PengBinaryType::Float32),
        Ok(4) => Ok(PengBinaryType::Float64),
        Ok(5) => Ok(PengBinaryType::Byte),
        Ok(6) => Ok(PengBinaryType::Bool),
        Ok(7) => Ok(PengBinaryType::String),
        Ok(8) => Ok(PengBinaryType::Object),
        Ok(9) => match rd_type(r) {
            Ok(v) => Ok(PengBinaryType::Vector(Box::new(v))),
            Err(e) => Err(e),
        },
        Ok(10) => Ok(PengBinaryType::Type),
        Ok(11) => Ok(PengBinaryType::Module),
        Ok(12) => Ok(PengBinaryType::Function),
        Ok(13) => Ok(PengBinaryType::Operator),
        Ok(14) => Ok(PengBinaryType::Thread),
        Ok(15) => match rd_fields(r) {
            Ok(v) => Ok(PengBinaryType::Custom(v)),
            Err(e) => Err(e),
        },
        Ok(16) => Ok(PengBinaryType::Any),
        Ok(_) => Err(binary_invalid_state_error("invalid type tag")),
        Err(e) => Err(e),
    }
}

fn wr_instr(w: &mut W, i: &PengBinaryOpcode) {
    use PengBinaryOpcode as O;
    let (t, num, text, rf) = match i {
        O::PushConst(v) => (0, Some(*v), None, None),
        O::MakeImmutable => (1, None, None, None),
        O::PushLocal(v) => (2, Some(*v), None, None),
        O::ReserveLocal(v) => (3, Some(*v), None, None),
        O::StoreLocal(v) => (4, Some(*v), None, None),
        O::PushHeap(v) => (5, None, None, Some(v)),
        O::PushHeapRef(v) => (6, None, None, Some(v)),
        O::StoreHeap => (7, None, None, None),
        O::PushString(v) => (8, None, Some(v), None),
        O::CreateEmptyObject => (9, None, None, None),
        O::CreateEmptyModule => (10, None, None, None),
        O::CreateVector(v) => (11, Some(*v), None, None),
        O::CreateSuperType(v) => (12, Some(*v), None, None),
        O::CreateTypedObject => (14, None, None, None),
        O::Convert => (15, None, None, None),
        O::Duplicate => (16, None, None, None),
        O::Pop => (17, None, None, None),
        O::Swap => (18, None, None, None),
        O::Add => (19, None, None, None),
        O::Subtract => (20, None, None, None),
        O::Multiply => (21, None, None, None),
        O::Divide => (22, None, None, None),
        O::Power => (23, None, None, None),
        O::Remainder => (24, None, None, None),
        O::Negate => (25, None, None, None),
        O::Concat => (26, None, None, None),
        O::And => (27, None, None, None),
        O::Or => (28, None, None, None),
        O::Not => (29, None, None, None),
        O::Equals => (30, None, None, None),
        O::NotEquals => (31, None, None, None),
        O::GreaterThan => (32, None, None, None),
        O::GreaterEqualsThan => (33, None, None, None),
        O::LessThan => (34, None, None, None),
        O::LessEqualsThan => (35, None, None, None),
        O::OperationCall => (36, None, None, None),
        O::TryOperationCall => (37, None, None, None),
        O::FunctionCall(v) => (38, Some(*v), None, None),
        O::FunctionCallSpread(v) => (39, Some(*v), None, None),
        O::TryFunctionCall(v) => (40, Some(*v), None, None),
        O::TryFunctionCallSpread(v) => (41, Some(*v), None, None),
        O::GetIndex => (42, None, None, None),
        O::SetIndex => (43, None, None, None),
        O::GetAttribute(v) => (44, None, Some(v), None),
        O::SetAttribute(v) => (45, None, Some(v), None),
        O::GetMember(v) => (46, None, Some(v), None),
        O::SetMember(v) => (47, None, Some(v), None),
        O::Jump(v) => (48, Some(*v), None, None),
        O::JumpIfTrue(v) => (49, Some(*v), None, None),
        O::JumpIfFalse(v) => (50, Some(*v), None, None),
        O::Return => (51, None, None, None),
    };
    w.u8(t);
    if let Some(v) = num {
        w.u64(v as u64)
    }
    if let Some(v) = text {
        w.text(v)
    }
    if let Some(v) = rf {
        wr_ref(w, v)
    }
}
fn rd_instr(r: &mut R) -> Result<PengBinaryOpcode, PengError> {
    use PengBinaryOpcode as O;
    let t = match r.u8() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    if matches!(t, 0 | 2 | 3 | 4 | 11 | 12 | 38 | 39 | 40 | 41 | 48 | 49 | 50) {
        let v = match r.len() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        return Ok(match t {
            0 => O::PushConst(v),
            2 => O::PushLocal(v),
            3 => O::ReserveLocal(v),
            4 => O::StoreLocal(v),
            11 => O::CreateVector(v),
            12 => O::CreateSuperType(v),
            38 => O::FunctionCall(v),
            39 => O::FunctionCallSpread(v),
            40 => O::TryFunctionCall(v),
            41 => O::TryFunctionCallSpread(v),
            48 => O::Jump(v),
            49 => O::JumpIfTrue(v),
            _ => O::JumpIfFalse(v),
        });
    }
    if t == 5 || t == 6 {
        let v = match rd_ref(r) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        return Ok(if t == 5 {
            O::PushHeap(v)
        } else {
            O::PushHeapRef(v)
        });
    }
    if matches!(t, 8 | 44 | 45 | 46 | 47) {
        let v = match r.text() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        return Ok(match t {
            8 => O::PushString(v),
            44 => O::GetAttribute(v),
            45 => O::SetAttribute(v),
            46 => O::GetMember(v),
            _ => O::SetMember(v),
        });
    }
    match t {
        1 => Ok(O::MakeImmutable),
        7 => Ok(O::StoreHeap),
        9 => Ok(O::CreateEmptyObject),
        10 => Ok(O::CreateEmptyModule),
        14 => Ok(O::CreateTypedObject),
        15 => Ok(O::Convert),
        16 => Ok(O::Duplicate),
        17 => Ok(O::Pop),
        18 => Ok(O::Swap),
        19 => Ok(O::Add),
        20 => Ok(O::Subtract),
        21 => Ok(O::Multiply),
        22 => Ok(O::Divide),
        23 => Ok(O::Power),
        24 => Ok(O::Remainder),
        25 => Ok(O::Negate),
        26 => Ok(O::Concat),
        27 => Ok(O::And),
        28 => Ok(O::Or),
        29 => Ok(O::Not),
        30 => Ok(O::Equals),
        31 => Ok(O::NotEquals),
        32 => Ok(O::GreaterThan),
        33 => Ok(O::GreaterEqualsThan),
        34 => Ok(O::LessThan),
        35 => Ok(O::LessEqualsThan),
        36 => Ok(O::OperationCall),
        37 => Ok(O::TryOperationCall),
        42 => Ok(O::GetIndex),
        43 => Ok(O::SetIndex),
        51 => Ok(O::Return),
        _ => Err(binary_invalid_state_error("invalid instruction tag")),
    }
}

fn wr_pos(w: &mut W, v: &PengPosition) {
    w.u64(v.id as u64);
    w.u64(v.line as u64);
    match v.column {
        Some(x) => {
            w.u8(1);
            w.u64(x as u64)
        }
        None => w.u8(0),
    }
}
fn rd_pos(r: &mut R) -> Result<PengPosition, PengError> {
    let id = match r.len() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let line = match r.len() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let column = match r.u8() {
        Ok(0) => None,
        Ok(1) => Some(match r.len() {
            Ok(v) => v,
            Err(e) => return Err(e),
        }),
        Ok(_) => return Err(binary_invalid_state_error("invalid option tag")),
        Err(e) => return Err(e),
    };
    Ok(PengPosition::new(id, line, column))
}
fn wr_code(w: &mut W, v: &PengBinaryBytecode) {
    w.len(v.bytecode.len());
    for x in &v.bytecode {
        wr_instr(w, x)
    }
    w.len(v.positions.len());
    for x in &v.positions {
        wr_pos(w, x)
    }
    w.len(v.consts.len());
    for x in &v.consts {
        wr_value(w, x)
    }
    w.len(v.using_values.len());
    for x in &v.using_values {
        wr_ref(w, x)
    }
}
fn rd_code(r: &mut R) -> Result<PengBinaryBytecode, PengError> {
    let n = match r.len() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut bytecode = Vec::with_capacity(n);
    for _ in 0..n {
        bytecode.push(match rd_instr(r) {
            Ok(v) => v,
            Err(e) => return Err(e),
        })
    }
    let n = match r.len() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut positions = Vec::with_capacity(n);
    for _ in 0..n {
        positions.push(match rd_pos(r) {
            Ok(v) => v,
            Err(e) => return Err(e),
        })
    }
    let n = match r.len() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut consts = Vec::with_capacity(n);
    for _ in 0..n {
        consts.push(match rd_value(r) {
            Ok(v) => v,
            Err(e) => return Err(e),
        })
    }
    let n = match r.len() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut using_values = Vec::with_capacity(n);
    for _ in 0..n {
        using_values.push(match rd_ref(r) {
            Ok(v) => v,
            Err(e) => return Err(e),
        })
    }
    Ok(PengBinaryBytecode {
        bytecode,
        positions,
        consts,
        using_values,
    })
}
fn wr_value(w: &mut W, v: &PengBinaryValue) {
    match v {
        PengBinaryValue::Cell(x) => {
            w.u8(0);
            wr_cell(w, x)
        }
        PengBinaryValue::String(x) => {
            w.u8(1);
            w.text(x)
        }
        PengBinaryValue::Object(x) => {
            w.u8(2);
            wr_fields(w, x)
        }
        PengBinaryValue::Vector(x) => {
            w.u8(3);
            w.len(x.len());
            for v in x {
                wr_bound(w, v)
            }
        }
        PengBinaryValue::Type(x) => {
            w.u8(4);
            wr_type(w, x)
        }
        PengBinaryValue::Module(x) => {
            w.u8(5);
            wr_fields(w, x)
        }
        PengBinaryValue::Function { code, params } => {
            w.u8(6);
            match params {
                PengBytecodeFunctionParams::Fixed(v) => {
                    w.u8(0);
                    w.u64(*v as u64)
                }
                PengBytecodeFunctionParams::Variadic(v) => {
                    w.u8(1);
                    w.u64(*v as u64)
                }
            }
            wr_code(w, code)
        }
        PengBinaryValue::Operation(x) => {
            w.u8(7);
            wr_code(w, x)
        }
    }
}
fn rd_value(r: &mut R) -> Result<PengBinaryValue, PengError> {
    match r.u8() {
        Ok(0) => match rd_cell(r) {
            Ok(v) => Ok(PengBinaryValue::Cell(v)),
            Err(e) => Err(e),
        },
        Ok(1) => match r.text() {
            Ok(v) => Ok(PengBinaryValue::String(v)),
            Err(e) => Err(e),
        },
        Ok(2) => match rd_fields(r) {
            Ok(v) => Ok(PengBinaryValue::Object(v)),
            Err(e) => Err(e),
        },
        Ok(3) => {
            let n = match r.len() {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            let mut o = Vec::with_capacity(n);
            for _ in 0..n {
                o.push(match rd_bound(r) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                })
            }
            Ok(PengBinaryValue::Vector(o))
        }
        Ok(4) => match rd_type(r) {
            Ok(v) => Ok(PengBinaryValue::Type(v)),
            Err(e) => Err(e),
        },
        Ok(5) => match rd_fields(r) {
            Ok(v) => Ok(PengBinaryValue::Module(v)),
            Err(e) => Err(e),
        },
        Ok(6) => {
            let t = match r.u8() {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            let n = match r.len() {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            let params = match t {
                0 => PengBytecodeFunctionParams::Fixed(n),
                1 => PengBytecodeFunctionParams::Variadic(n),
                _ => return Err(binary_invalid_state_error("invalid function params tag")),
            };
            match rd_code(r) {
                Ok(code) => Ok(PengBinaryValue::Function { code, params }),
                Err(e) => Err(e),
            }
        }
        Ok(7) => match rd_code(r) {
            Ok(v) => Ok(PengBinaryValue::Operation(v)),
            Err(e) => Err(e),
        },
        Ok(_) => Err(binary_invalid_state_error("invalid value tag")),
        Err(e) => Err(e),
    }
}

pub fn peng_binary_encode(file: &PengBinaryFile) -> Result<Vec<u8>, PengError> {
    match peng_binary_validate_file(file) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }
    let mut w = W { b: Vec::new() };
    w.b.extend(file.header.magic);
    w.u16(file.header.format_version);
    w.u8(file.header.arch_bits);
    w.u8(match file.header.endian {
        PengBinaryEndian::Little => 0,
        PengBinaryEndian::Big => 1,
    });
    w.u32(file.header.flags);
    w.u8(match file.entry_kind {
        PengBinaryEntryKind::Program => 0,
        PengBinaryEntryKind::Script => 1,
    });
    w.len(file.heap.len());
    for x in &file.heap {
        w.u32(x.id.0);
        wr_value(&mut w, &x.value)
    }
    w.len(file.globals.len());
    for x in &file.globals {
        w.text(&x.name);
        w.u8(x.mutable as u8);
        wr_ref(&mut w, &x.value)
    }
    match &file.init {
        Some(v) => {
            w.u8(1);
            wr_ref(&mut w, v)
        }
        None => w.u8(0),
    }
    Ok(w.b)
}
pub fn peng_binary_decode(bytes: &[u8]) -> Result<PengBinaryFile, PengError> {
    let mut r = R { b: bytes, p: 0 };
    let magic = match r.take(4) {
        Ok(v) => [v[0], v[1], v[2], v[3]],
        Err(e) => return Err(e),
    };
    let format_version = match r.u16() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let arch_bits = match r.u8() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let endian = match r.u8() {
        Ok(0) => PengBinaryEndian::Little,
        Ok(1) => PengBinaryEndian::Big,
        Ok(_) => return Err(binary_invalid_state_error("invalid endian tag")),
        Err(e) => return Err(e),
    };
    let flags = match r.u32() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let entry_kind = match r.u8() {
        Ok(0) => PengBinaryEntryKind::Program,
        Ok(1) => PengBinaryEntryKind::Script,
        Ok(_) => return Err(binary_invalid_state_error("invalid entry kind")),
        Err(e) => return Err(e),
    };
    let n = match r.len() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut heap = Vec::with_capacity(n);
    for _ in 0..n {
        let id = match r.u32() {
            Ok(v) => PengBinaryHeapId(v),
            Err(e) => return Err(e),
        };
        let value = match rd_value(&mut r) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        heap.push(PengBinaryHeapValue { id, value })
    }
    let n = match r.len() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut globals = Vec::with_capacity(n);
    for _ in 0..n {
        let name = match r.text() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let mutable = match r.u8() {
            Ok(0) => false,
            Ok(1) => true,
            Ok(_) => return Err(binary_invalid_state_error("invalid bool")),
            Err(e) => return Err(e),
        };
        let value = match rd_ref(&mut r) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        globals.push(PengBinaryGlobal {
            name,
            mutable,
            value,
        })
    }
    let init = match r.u8() {
        Ok(0) => None,
        Ok(1) => Some(match rd_ref(&mut r) {
            Ok(v) => v,
            Err(e) => return Err(e),
        }),
        Ok(_) => return Err(binary_invalid_state_error("invalid option tag")),
        Err(e) => return Err(e),
    };
    if r.p != bytes.len() {
        return Err(binary_invalid_state_error("trailing bytes"));
    }
    let file = PengBinaryFile {
        header: PengBinaryHeader {
            magic,
            format_version,
            arch_bits,
            endian,
            flags,
        },
        entry_kind,
        heap,
        globals,
        init,
    };
    match peng_binary_validate_file(&file) {
        Ok(()) => Ok(file),
        Err(e) => Err(e),
    }
}
fn valid_ref(v: &PengBinaryRef, n: usize) -> bool {
    match v {
        PengBinaryRef::Heap(v) => (v.0 as usize) < n,
        PengBinaryRef::External(s) => !s.is_empty(),
    }
}
fn validate_code(v: &PengBinaryBytecode, n: usize) -> Result<(), PengError> {
    if !v.positions.is_empty() && v.positions.len() != v.bytecode.len() {
        return Err(binary_invalid_state_error(
            "positions length differs from bytecode length",
        ));
    }
    for i in &v.bytecode {
        match i {
            PengBinaryOpcode::PushConst(x) if *x >= v.consts.len() => {
                return Err(binary_invalid_state_error("constant index out of range"));
            }
            PengBinaryOpcode::Jump(x)
            | PengBinaryOpcode::JumpIfTrue(x)
            | PengBinaryOpcode::JumpIfFalse(x)
                if *x > v.bytecode.len() =>
            {
                return Err(binary_invalid_state_error("jump target out of range"));
            }
            PengBinaryOpcode::PushHeap(x) | PengBinaryOpcode::PushHeapRef(x)
                if !valid_ref(x, n) =>
            {
                return Err(binary_invalid_state_error(
                    "instruction heap id out of range",
                ));
            }
            _ => {}
        }
    }
    for x in &v.using_values {
        if !valid_ref(x, n) {
            return Err(binary_invalid_state_error(
                "using value heap id out of range",
            ));
        }
    }
    Ok(())
}
fn validate_value(v: &PengBinaryValue, n: usize) -> Result<(), PengError> {
    match v {
        PengBinaryValue::Function { code, .. } | PengBinaryValue::Operation(code) => {
            validate_code(code, n)
        }
        _ => Ok(()),
    }
}
pub fn peng_binary_validate_file(file: &PengBinaryFile) -> Result<(), PengError> {
    if file.header.magic != PENG_BINARY_MAGIC {
        return Err(binary_invalid_state_error("invalid magic"));
    }
    if file.header.format_version != PENG_BINARY_FORMAT_VERSION {
        return Err(binary_invalid_state_error("unsupported format version"));
    }
    if file.header.endian != PengBinaryEndian::Little {
        return Err(binary_invalid_state_error("unsupported endian"));
    }
    for (index, x) in file.heap.iter().enumerate() {
        if x.id.0 as usize != index {
            return Err(binary_invalid_state_error(
                "heap ids must be contiguous and ordered",
            ));
        }
        match validate_value(&x.value, file.heap.len()) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }
    for x in &file.globals {
        if x.name.is_empty() || !valid_ref(&x.value, file.heap.len()) {
            return Err(binary_invalid_state_error("invalid global"));
        }
    }
    if let Some(x) = &file.init {
        if !valid_ref(x, file.heap.len()) {
            return Err(binary_invalid_state_error("invalid init reference"));
        }
    }
    Ok(())
}
