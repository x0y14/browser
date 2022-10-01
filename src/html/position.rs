#[derive(Debug, Clone)]
pub struct Position {
    pub line_no: u32,
    pub at_line: u32,
    pub at_whole: u32,
}

impl Position {
    pub fn new(line_no: u32, at_line: u32, at_whole: u32) -> Position {
        Position {line_no, at_line, at_whole}
    }
}