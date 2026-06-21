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

    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (
                Self::File {
                    file_id: left_file_id,
                    line: left_line,
                    column: left_column,
                },
                Self::File {
                    file_id: right_file_id,
                    line: right_line,
                    column: right_column,
                },
            ) => {
                left_file_id == right_file_id
                    && left_line == right_line
                    && left_column == right_column
            }
            (
                Self::Source {
                    line: left_line,
                    column: left_column,
                },
                Self::Source {
                    line: right_line,
                    column: right_column,
                },
            ) => left_line == right_line && left_column == right_column,
            _ => false,
        }
    }
}
