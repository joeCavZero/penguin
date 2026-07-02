pub const PENG_BINARY_MAGIC: [u8; 4] = *b"peng";
pub const PENG_BINARY_FORMAT_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PengBinaryEndian {
    Little,
    Big,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PengBinaryHeader {
    pub magic: [u8; 4],
    pub format_version: u16,
    pub arch_bits: u8,
    pub endian: PengBinaryEndian,
    pub flags: u32,
}

impl PengBinaryHeader {
    pub fn new() -> Self {
        Self {
            magic: PENG_BINARY_MAGIC,
            format_version: PENG_BINARY_FORMAT_VERSION,
            arch_bits: usize::BITS as u8,
            endian: PengBinaryEndian::Little,
            flags: 0,
        }
    }

    pub fn default() -> Self {
        Self::new()
    }
}
