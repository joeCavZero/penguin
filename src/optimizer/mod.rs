pub mod optimizer;
pub mod config;
pub mod constant_folding;
pub mod boolean_simplification;
pub mod dead_code;
pub mod empty_blocks;
pub mod constant_propagation;

pub use optimizer::*;
pub use config::*;