#[derive(Debug, Clone)]
pub enum PengPosition {
    File {
        file_id: usize,
        line: usize,
        column: Option<usize>,
    },
    Source {
        line: usize,
        column: Option<usize>,
    },
}

impl PengPosition {
    pub fn new_file(file_id: usize, line: usize, column: Option<usize>) -> Self {
        Self::File { file_id, line, column }
    }

    pub fn new_source(line: usize, column: Option<usize>) -> Self {
        Self::Source { line, column }
    }
}