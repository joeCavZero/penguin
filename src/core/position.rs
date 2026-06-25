#[derive(Debug, Clone)]
pub struct PengPosition {
    pub line: usize,
    pub column: Option<usize>,
}

impl PengPosition {
    pub fn new( line: usize, column: Option<usize>) -> Self {
        Self{ line, column }
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        self.line == rhs.line
        && self.column == rhs.column
    }
}
