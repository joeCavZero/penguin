#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PengBinaryBuildOptions {
    pub strip_positions: bool,
    pub keep_debug_names: bool,
}

impl PengBinaryBuildOptions {
    pub fn default() -> Self {
        Self {
            strip_positions: false,
            keep_debug_names: false,
        }
    }
}
