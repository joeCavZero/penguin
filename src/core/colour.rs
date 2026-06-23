#[derive(Debug,Clone)]
pub enum PengColour {
    White,
    Gray,
    Black,
}

#[derive(Debug,Clone)]
pub struct PengColoured<T> {
    pub colour: PengColour,
    pub value: T,
}

impl<T> PengColoured<T> {
    pub fn new(value: T) -> Self {
        Self {
            colour: PengColour::White,
            value: value,
        }
    }
}