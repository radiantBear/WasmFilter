mod utils;
pub mod lexer;
pub mod parser;
pub mod types;

use std::iter::Peekable;
use std::str::Chars;
use wasm_bindgen::prelude::*;
use crate::lexer::{BareToken, BareTokenData};
use crate::types::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, world!!!");
}

#[wasm_bindgen(getter_with_clone)]
pub struct LexData {
    pub tokens: Vec<lexer::BareTokenData>,
    pub errors: Vec<FilterError>
}

#[wasm_bindgen]
pub fn lex_filter(filter: &str) -> LexData {
    utils::set_panic_hook();

    let filter = String::from(filter);
    let mut filter = filter.chars().peekable();

    run_lex(&mut filter, 0, 0, 0)
}

fn run_lex(mut filter: &mut Peekable<Chars>, cursor: usize, line: usize, col: usize) -> LexData {
    let mut data = LexData { tokens: Vec::new(), errors: Vec::new() };

    let result = lexer::lex(&mut filter, cursor, line, col);

    match result.1 {
        None => {
            for token in result.0 {
                data.tokens.push(token.to_bare());
            };
        }

        Some(error) => {
            for token in result.0 {
                data.tokens.push(token.to_bare());
            };
            data.tokens.push(BareTokenData {
                token: BareToken::Error,
                start: error.start,
                start_line: error.start_line,
                start_col: error.start_col,
                end: error.end,
                end_line: error.end_line,
                end_col: error.end_col
            });

            // Restart lexing at the next character
            let mut result = run_lex(&mut filter, error.end, error.end_line, error.end_col);
 
            data.errors.push(error);
            data.tokens.append(&mut result.tokens);
            data.errors.append(&mut result.errors);
        }
    }

    data
}

#[wasm_bindgen]
pub fn parse_filter(filter: &str) {
    utils::set_panic_hook();

    let lexed_filter = lexer::lex(&mut String::from(filter).chars().peekable(), 0, 0, 0);
    alert(format!("{:?}", lexed_filter).to_string().as_str());

    if lexed_filter.1.is_some() {
        return 
    };
    
    let parsed_filter = parser::parse(lexed_filter.0);
    alert(format!("{:?}", parsed_filter).to_string().as_str());
}