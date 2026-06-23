#[derive(Debug, Clone)]
pub enum PengStated<T> {
    Initialized(T),
    Uninitialized,
}