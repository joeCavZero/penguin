#[derive(Debug, Clone)]
pub enum PengBinded<T> {
    Mutable(T),
    Immutable(T),
}