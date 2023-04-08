#![recursion_limit="256"]

mod lexer;
mod parser;
mod semantic_analyzer;

fn main() {
    // parser::main()
    semantic_analyzer::main()
}
