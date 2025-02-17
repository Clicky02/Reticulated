use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    line: usize,
    column: usize,
    index: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

impl Position {
    pub fn new() -> Self {
        Self {
            line: 0,
            column: 0,
            index: 0,
        }
    }

    pub fn new_at(line: usize, col: usize, index: usize) -> Self {
        Self {
            line,
            column: col,
            index,
        }
    }

    pub fn advance(&mut self, ch: Option<char>) {
        self.index += 1;

        if let Some('\n') = ch {
            self.line += 1;
            self.column = 0;
        } else {
            // TODO: Handle None?
            self.column += 1;
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

impl From<Position> for usize {
    fn from(value: Position) -> Self {
        value.index
    }
}
