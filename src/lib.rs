mod utils;
pub mod lexer;
pub mod parser;

use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub fn lex_filter(filter: &str) -> Vec<lexer::BareTokenData> {
    utils::set_panic_hook();

    lexer::lex(String::from(filter)).iter().map(|token| token.to_bare()).collect()
}

#[wasm_bindgen]
pub fn parse_filter(filter: &str) {
    utils::set_panic_hook();

    let lexed_filter = lexer::lex(String::from(filter));
    // alert(format!("{:?}", lexed_filter).to_string().as_str());
    
    let _parsed_filter = parser::parse(lexed_filter);
    // alert(format!("{:?}", parsed_filter).to_string().as_str());
}