mod utils;
mod lexer;

extern crate js_sys;
extern crate web_sys;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsObject;
use wasm_bindgen::convert::FromWasmAbi;
use std::fmt;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $ ( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen(module = "/libraries.js")]
extern "C" {
    //fn call(s: String, args: Vec<String>) -> String;
}


#[wasm_bindgen]
pub fn lex(s: String) -> String {
    utils::set_panic_hook();

    let mut lex = lexer::Lexer::new(s).expect("Could not create lexer");
    let mut s: String = String::new();
    while !lex.is_done() {
        s.push_str(&format!("{:?}\n", lex.next().expect("Could not lex line")));
    }
    s
}