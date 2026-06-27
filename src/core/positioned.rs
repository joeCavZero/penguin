use crate::core::*;

#[derive(Debug, Clone)]
pub struct PengPositioned<T> {
    pub value: T,
    pub position: PengPosition,
}

impl<T> PengPositioned<T> {
    pub fn new(value: T, position: PengPosition) -> Self {
        Self { value, position }
    }
}
