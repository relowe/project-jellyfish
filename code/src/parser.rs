#![allow(dead_code)]

use std::{env};
use crate::lexer;
use std::mem;

const NULL_TOKEN: lexer::Token = lexer::Token {
    row: 0,
    col: 0,
    token_type: &lexer::TokenType::INVALID,
    lexeme: None
};

macro_rules! log {
    ($($t:tt)*) => (println!("{}",  &format_args!($ ( $t ) *).to_string() ))
}

#[derive(Debug, Clone)]
pub enum ParseType {
    CODE,       // the start of all trees
    DEFINITIONS,// the definitions section of the program
    GLOBALDEFS, // the definition section for global variables
    FUNDEFS,    // the definition section for functions
    BLOCK,      // a block of statements to execute
    FUNDEF,     // function definition
    VARDEF,     // variable definition
    STRUCTDEFS, // define structures
    STRUCTDEF,  // define a structure
    ASSIGN,     // assign a value to a variable
    WHILE,      // while loop
    IF,         // if statement
    REPEAT,     // repeat a number of times
    REPEATFOR,  // repeat for each value in an array
    UNLINK,     // unlink
    QUIT,       // quit
    BREAK,      // break
    CONTINUE,   // continue
    BINOP,      // binary operation (add, sub, div, ...)
    BINCOMP,    // binary comparison (=, !=, <, ...)
    RETURN,     // return
    TYPE,       // variable type
    ARRAY,      // array literal
    ARRAYBOUNDDEF, // define an array variable with bounds
    ARRAYLITDEF,   // define an array variable with a literal array
    GETINDEX,      // return the index of an array
    SETINDEX,      // set a value at an array index
    GETSTRUCT,     // get the value of a structure key
    SETSTRUCT,     // st the value of a structure key
    CALL,          // call a function
    ARGS,          // list of values (arguments)
    PARAMS,        // list of params
    PARAM,         // a single name and expected value
    ID,            // some form of name
    LIT,           // a literal value
    STRUCTARGS,    // a list of structure arguments
    STRUCTARG,     // a name, type, and default value
}

#[derive(Debug, Clone)]
pub struct ParseTree {
    parse_type: ParseType,
    token: lexer::Token,
    children: Vec<Option<ParseTree>>,
}

#[derive(Debug)]
pub struct Parser {
    lexer: lexer::Lexer,
}

impl Parser {

    // Construct a lexer for the parser from a String
    pub fn new(text: String) -> Result<Self, &'static str> {
        let lexer = lexer::Lexer::new(text)?;
        Ok(Parser {
            lexer: lexer,
        })
    }

    // Construct a lexer for the parser from a file
    pub fn from_file(file: String) -> Result<Self, &'static str> {
        let lexer = lexer::Lexer::from_file(file)?;
        Ok(Parser {
            lexer: lexer,
        })
    }

    // Checks for end of file in lexer
    pub fn is_done(&self) -> bool {
        self.lexer.is_done()
    }

    // Consume next token in lexer
    pub fn next(&mut self) -> Result<lexer::Token, &'static str> {
        self.lexer.next()
    }

    fn has(&self, token_type: &lexer::TokenType) -> bool {
        // self.lexer.curr_token.token_type == token_type
        mem::discriminant(&self.lexer.curr_token.token_type) == mem::discriminant(token_type)
    }

    // need to make it kill program...
    fn must_be(&self, token_type: &lexer::TokenType) {
        if !self.has(token_type) {
            log!("Parse Error on Line: {}, Column: {}", self.lexer.curr_token.row, self.lexer.curr_token.col);
            log!("Expected: {:?}", token_type);
            log!("Got: {:?}", self.lexer.curr_token.token_type);
        }
    }

    pub fn parse(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.code()
    }

    fn code(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.next()?;
        let parse_tree = ParseTree{
            parse_type: ParseType::CODE,
            token: NULL_TOKEN,
            children: Vec::new()
        };

        if self.has(&lexer::TokenType::DEF) {
            self.next()?;
            parse_tree.children.push(self.definitions()?);
        }
        else {
            parse_tree.children.push(None);
        }

        parse_tree.children.push(self.program()?);

        Ok(Some(parse_tree))
    }

    fn definitions(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let parse_tree = ParseTree{
            parse_type: ParseType::DEFINITIONS,
            token: NULL_TOKEN,
            children: Vec::new()
        };

        // Will check for STRUCT token and return None if there isn't one
        parse_tree.children.push(self.structure_defs()?);

        // Will check for ID token and return None if there isn't one
        parse_tree.children.push(self.global_defs()?);

        // Will check for FUN token and return None if there isn't one
        parse_tree.children.push(self.function_defs()?);

        Ok(Some(parse_tree))
    }

    fn structure_defs(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let parse_tree = ParseTree{
            parse_type: ParseType::STRUCTDEFS,
            token: NULL_TOKEN,
            children: Vec::new()
        };

        if !self.has(&lexer::TokenType::STRUCT) {
            return Ok(None);
        }
        
        while self.has(&lexer::TokenType::STRUCT) {
            parse_tree.children.push(self.structure_def()?);
        }

        Ok(Some(parse_tree))
    }

    fn structure_def(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let parse_tree = ParseTree{
            parse_type: ParseType::STRUCTDEF,
            token: NULL_TOKEN,
            children: Vec::new()
        };

        self.must_be(&lexer::TokenType::STRUCT);
        self.next()?;

        parse_tree.children.push(self.id()?);

        parse_tree.children.push(self.structure_args()?);

        self.must_be(&lexer::TokenType::END);
        self.next()?;

        self.must_be(&lexer::TokenType::STRUCT);
        self.next()?;

        Ok(Some(parse_tree))
    }
    
    fn id(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.must_be(&lexer::TokenType::ID('a'.to_string()));

        let parse_tree = ParseTree{
            parse_type: ParseType::ID,
            token: self.lexer.curr_token,
            children: Vec::new()
        };

        self.next()?;
        Ok(Some(parse_tree))
    }

    fn structure_args(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let parse_tree = ParseTree{
            parse_type: ParseType::STRUCTARGS,
            token: NULL_TOKEN,
            children: Vec::new()
        };

        self.must_be(&lexer::TokenType::ID('a'.to_string()));

        while self.has(&lexer::TokenType::ID('a'.to_string())) {
            parse_tree.children.push(self.structure_arg()?);
        }

        Ok(Some(parse_tree))
    }

    fn structure_arg(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let parse_tree = ParseTree{
            parse_type: ParseType::STRUCTARG,
            token: NULL_TOKEN,
            children: Vec::new()
        };

        self.must_be(&lexer::TokenType::ID('a'.to_string()));
        self.next()?;

        parse_tree.children.push(self.var_type()?);

        // Optional return of literal
        if self.has(&lexer::TokenType::EQ) {
            self.next()?;

            parse_tree.children.push(self.resolvable()?);
        }
        else {
            parse_tree.children.push(None);
        }
        
        Ok(Some(parse_tree))
    }

    // not done, fix later
    fn global_defs(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let parse_tree = ParseTree{
            parse_type: ParseType::GLOBALDEFS,
            token: NULL_TOKEN,
            children: Vec::new()
        };

        while self.has(&lexer::TokenType::ID('a'.to_string())) {
            parse_tree.children.push(self.var_def()?);
        }

        Ok(Some(parse_tree))
    }

    // not done, fix later
    fn assign_or_vardef(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.must_be(&lexer::TokenType::ID('a'.to_string()));

        let parse_tree = ParseTree{
            parse_type: ParseType::ASSIGN,
            token: NULL_TOKEN,
            children: Vec::new()
        };

        self.next()?;

        self.must_be(&lexer::TokenType::COLON);
        self.next()?;

        parse_tree.children.push(self.var_type()?);

        self.must_be(&lexer::TokenType::EQ);
        self.next()?;

        parse_tree.children.push(self.resolvable()?);
        
        Ok(Some(parse_tree))
    }

    fn foo(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let parse_tree = ParseTree{
            parse_type: ParseType::STRUCTDEF,
            token: NULL_TOKEN,
            children: Vec::new()
        };

        Ok(Some(parse_tree))
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
        p = Parser::new("hello, world + 123 - 11.4491 = 12333 test.file \n hello again".to_string()).expect("Could not create lexer");
    }

    println!("{:?}", p.parse().expect("Waaaah"));
    // p.next().expect("Boo hoo!");
    // p.must_be(&lexer::TokenType::DEF);
    
    // 
    // while !p.is_done() {
    //     println!("{:?}", p.next());
    // }
}