use std::env;
mod lexer;

#[derive(Debug, Clone)]
pub enum ParseType {
    BLOCK,
    ADD,
    SUB,
    //... TODO
}

#[derive(Debug, Clone)]
pub struct ParseTree {
    parse_type: ParseType,
    token: lexer::Token,
    children: Vec<ParseTree>,
}

#[derive(Debug)]
pub struct Parser {
    lexer: lexer::Lexer,
}

impl Parser {

    pub fn new(text: String) -> Result<Self, &'static str> {
        let lexer = lexer::Lexer::new(text)?;
        Ok(Parser {
            lexer: lexer,
        })
    }

    pub fn from_file(file: String) -> Result<Self, &'static str> {
        let lexer = lexer::Lexer::from_file(file)?;
        Ok(Parser {
            lexer: lexer,
        })
    }

    pub fn is_done(&self) -> bool {
        self.lexer.is_done()
    }

    pub fn next(&mut self) -> Result<lexer::Token, &'static str> {
        self.lexer.next()
    }
}


pub fn main() {
    let args: Vec<String> = env::args().collect();

    let mut p: Parser;
    if args.len() > 1 {
        let fname = &args[1];
        p = Parser::from_file(fname.to_string()).expect("Could not create lexer");
    }
    else {
        p = Parser::new("hello, world + 123 - 11.4491 = 12333 test.file \n hello again".to_string()).expect("Could not create lexer");
    }
    
    while !p.is_done() {
        println!("{:?}", p.next());
    }
}