use crate::core::typing::*;

#[derive(Debug, Clone)]
pub struct PengUnion {
    pub unions: Vec<PengType>,
}

impl PengUnion {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.unions.len() == rhs.unions.len()
            && self
                .unions
                .iter()
                .zip(&rhs.unions)
                .all(|(left, right)| left.equals(right))
    }
}
