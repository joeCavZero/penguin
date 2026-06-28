#[derive(Debug, Clone)]
pub struct PengOptimizerConfig {
    pub constant_folding: bool,
    pub boolean_simplification: bool,
    pub dead_code: bool,
    pub empty_blocks: bool,
    pub constant_propagation: bool,
}

impl PengOptimizerConfig {
    pub fn none() -> Self {
        Self {
            constant_folding: false,
            boolean_simplification: false,
            dead_code: false,
            empty_blocks: false,
            constant_propagation: false,
        }
    }

    pub fn safe() -> Self {
        Self {
            constant_folding: true,
            boolean_simplification: true,
            dead_code: true,
            empty_blocks: true,
            constant_propagation: false,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            constant_folding: true,
            boolean_simplification: true,
            dead_code: true,
            empty_blocks: true,
            constant_propagation: true,
        }
    }
}