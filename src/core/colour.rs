#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PengColour {
    Red,   // not passed
    Black, // passed
}

#[derive(Debug, Clone)]
pub struct PengColoured<T> {
    pub colour: PengColour,
    pub value: T,
}

impl<T> PengColoured<T> {
    pub fn new(value: T) -> Self {
        Self {
            colour: PengColour::Black,
            value: value,
        }
    }

    pub fn paint(&mut self, colour: PengColour) {
        self.colour = colour
    }
}
