#![allow(dead_code)]

use std::{env, process};
use std::collections::HashMap;
use crate::lexer::{Token, TokenType};
use crate::parser::{ParseTree, ParseType};
use crate::semantic_analyzer::{SemanticAnalyzer};

pub enum EvalType {
    NUMBER,
    TEXT,
    NONE,
    CONDITION,
}

#[derive(Clone, Eq, Debug)]
pub struct EvalResult {
    eval_type: Evaltype,
    value: Any,
}

pub struct Interpreter {
    semantic_analyzer: SemanticAnalyzer,
    memory: (),
}

impl Interpreter {
    pub fn new(tree: &ParseTree) -> Self {
        sa = SemanticAnalyzer::new();
        sa.analyze(tree).expect("Semantic Analyzer errored");
        Interpreter {
            semantic_analyzer: sa,
            memory: (),
        }
    }

    pub fn run(&mut self, tree: &ParseTree) -> Result<(), String> {
        Ok(())
    }


}

pub fn main() {
    let args: Vec<String> = env::args().collect();

    // create parser
    let mut p: Parser;
    // if argument, open file
    if args.len() > 1 {
        let fname = &args[1];
        p = Parser::from_file(fname.to_string()).expect("Could not create lexer");
    }
    else {
        p = Parser::new("
    program
        print(\"Hello world!\")
    end program
    ".to_string()).expect("Could not create lexer");
    }

    let tree = p.parse().expect("Error");

    tree.clone().expect("error").print();
    println!("\n\n\n\n");


    let mut int = Interpreter::new(p);

    
}