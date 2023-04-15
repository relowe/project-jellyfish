#![recursion_limit="256"]
#![allow(warnings)]

mod lexer;
mod parser;
mod semantic_analyzer;

use std::env;

fn main() {
    //parser::main()
    env::set_var("RUST_BACKTRACE", "1");
    semantic_analyzer::main()
}
