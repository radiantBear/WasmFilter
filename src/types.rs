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