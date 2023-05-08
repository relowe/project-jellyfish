#![recursion_limit="256"]
// #![allow(warnings)]

mod lexer;
mod parser;
mod semantic_analyzer;
mod interpreter;
mod library_handler;

use std::env;

// Cargo requires a main funtion to start execution.
// For the intepreter, ew just pull in each module file
//  and call the interpreter's main function.
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    //lexer::main()
    //parser::main()
    //semantic_analyzer::main()
    interpreter::main()
}
