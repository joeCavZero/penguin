pub mod boolean_simplification;
pub mod config;
pub mod constant_folding;
pub mod constant_propagation;
pub mod dead_code;
pub mod empty_blocks;
pub mod optimizer;

pub use config::*;
pub use optimizer::*;
