#![recursion_limit="256"]

mod lexer;
mod parser;
mod semantic_analyzer;

fn main() {
    semantic_analyzer::main()
}
