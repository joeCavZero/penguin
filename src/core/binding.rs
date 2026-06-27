#[derive(Debug, Clone)]
pub enum PengBinded<T> {
    Mutable(T),
    Immutable(T),
}

impl<T> PengBinded<T> {
    pub fn value(&self) -> &T {
        match self {
            Self::Mutable(v) => v,
            Self::Immutable(v) => v,
        }
    }

    pub fn value_mut(&mut self) -> &mut T {
        match self {
            Self::Mutable(v) => v,
            Self::Immutable(v) => v,
        }
    }
}
