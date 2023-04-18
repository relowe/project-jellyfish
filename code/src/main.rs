#![recursion_limit="256"]
// #![allow(warnings)]

mod lexer;
mod parser;
mod semantic_analyzer;
mod interpreter;
mod library_handler;

use std::env;

fn main() {
    //parser::main()
    env::set_var("RUST_BACKTRACE", "1");
    // semantic_analyzer::main()
    interpreter::main()
}
