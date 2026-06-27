#[derive(Debug, Clone)]
pub struct PengPosition {
    pub id: usize,
    pub line: usize,
    pub column: Option<usize>,
}

impl PengPosition {
    pub fn new(id: usize, line: usize, column: Option<usize>) -> Self {
        Self { id, line, column }
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        self.id == rhs.id && self.line == rhs.line && self.column == rhs.column
    }
}
