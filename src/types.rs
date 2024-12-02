use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilterError {
    pub message: String,

    // Specific location that caused error
    pub start: usize,           // Inclusive
    pub start_line: usize,      // Inclusive
    pub start_col: usize,       // Inclusive
    pub end: usize,             // Not inclusive
    pub end_line: usize,        // Inclusive
    pub end_col: usize,         // Not inclusive

    // Relevant range being considered when error occurs
    pub range_start: usize,     // Inclusive
    pub range_end: usize        // Not inclusive
}

impl FilterError {
    pub fn new(message: String, range_start: usize, range_end: usize, start: usize, start_line: usize, start_col: usize, end: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            message,
            range_start,
            range_end,
            start,
            start_line,
            start_col,
            end,
            end_line,
            end_col
        }
    }

    pub fn new_oneline_context(message: String, line: usize, range_start: usize, range_end: usize, start: usize, start_col: usize, end: usize, end_col: usize) -> Self {
        Self::new(message, range_start, range_end, start, line, start_col, end, line, end_col)
    }
    
    pub fn new_oneline(message: String, line: usize, start: usize, start_col: usize, end: usize, end_col: usize) -> Self {
        Self::new(message, start, end, start, line, start_col, end, line, end_col)
    }

    pub fn new_onechar(message: String, line: usize, start: usize, start_col: usize) -> Self {
        Self::new_oneline(message, line, start, start_col, start + 1, start_col + 1)
    }
}