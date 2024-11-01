mod utils;
mod lexer;

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
pub fn parse_filter(filter: &str) -> *const lexer::Search {
    utils::set_panic_hook();

    let search: Box<Option<lexer::Search>> = Box::new(None);

    let lexed_filter = lexer::lex(String::from(filter));
    alert(format!("{:?}", lexed_filter).to_string().as_str());

    search.as_slice().as_ptr()
}

#[wasm_bindgen]
pub fn display(search: *const lexer::Search) {
    alert(format!("{:?}", search).to_string().as_str());
}