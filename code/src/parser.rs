#![allow(dead_code)]

use std::{env};
use crate::lexer;
use std::mem;

static NULL_TOKEN: lexer::Token = lexer::Token {
    row: 0,
    col: 0,
    token_type: lexer::TokenType::INVALID,
    lexeme: None
};

static ID_TYPE: lexer::TokenType = lexer::TokenType::ID(String::new());
static TEXT_TYPE: lexer::TokenType = lexer::TokenType::TEXT(String::new());
static NUMBER_TYPE: lexer::TokenType = lexer::TokenType::NUMBER(0.0);

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
    IDS,           // a collection of names (for variable declaration)
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

impl ParseTree {
    pub fn print(&self) {
        let len = self.children.len();
        let mut idx = 0;

        if len == 0 {
            println!{"{:?} ({:?})", self.parse_type, self.token};
            return;
        }

        for child in self.children.iter() {
            if idx == len / 2 {
                println!{"{:?} ({:?})", self.parse_type, self.token};
            }
            
            match child {
                Some(tree) => tree.print_tabbed(1),
                _ => (),
            };

            idx += 1;
        }
    }

    fn print_tabbed(&self, tab: usize) {
        let len = self.children.len();
        let mut idx = 0;

        if len == 0 {
            for n in 0..tab {
                print!{"| "};
            }
            println!{"{:?} ({:?})", self.parse_type, self.token};
            return;
        }

        for child in self.children.iter() {
            if idx == len / 2 {
                for n in 0..tab {
                    print!{"| "};
                }
                println!{"{:?} ({:?})", self.parse_type, self.token};
            }
            
            match child {
                Some(tree) => tree.print_tabbed(tab + 1),
                _ => (),
            };

            idx += 1;
        }
    }
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

    // Get the current token in the lexer
    pub fn curr_token(&self) -> lexer::Token {
        self.lexer.curr_token.clone()
    }

    fn has(&self, token_type: &lexer::TokenType) -> bool {
        // self.lexer.curr_token.token_type == token_type
        mem::discriminant(&self.curr_token().token_type) == mem::discriminant(token_type)
    }

    // need to make it kill program...
    fn must_be(&self, token_type: &lexer::TokenType) {
        if !self.has(token_type) {
            log!("Parse Error on Line: {}, Column: {}", self.curr_token().row, self.curr_token().col);
            log!("Expected: {:?}", token_type);
            log!("Got: {:?}", self.curr_token().token_type);
        }
    }

    // Helper function to call 'must_be' and 'next'
    fn eat(&mut self, token_type: &lexer::TokenType) -> Result<lexer::Token, &'static str> {
        self.must_be(token_type);
        self.next()
    }

    pub fn parse(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.code()
    }

    fn code(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.next()?;
        let mut parse_tree = ParseTree{
            parse_type: ParseType::CODE,
            token: NULL_TOKEN.clone(),
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
        let mut parse_tree = ParseTree{
            parse_type: ParseType::DEFINITIONS,
            token: NULL_TOKEN.clone(),
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
        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTDEFS,
            token: NULL_TOKEN.clone(),
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
        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTDEF,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        self.eat(&lexer::TokenType::STRUCT)?;

        parse_tree.children.push(self.id()?);

        parse_tree.children.push(self.structure_args()?);

        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::STRUCT)?;

        Ok(Some(parse_tree))
    }
    
    fn id(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.must_be(&ID_TYPE);

        let mut parse_tree = ParseTree{
            parse_type: ParseType::ID,
            token: self.curr_token(),
            children: Vec::new()
        };

        self.next()?;
        Ok(Some(parse_tree))
    }

    fn structure_args(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTARGS,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        self.must_be(&ID_TYPE);

        while self.has(&ID_TYPE) {
            parse_tree.children.push(self.structure_arg()?);
        }

        Ok(Some(parse_tree))
    }

    fn structure_arg(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTARG,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        parse_tree.children.push(self.id()?);

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

    fn global_defs(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::GLOBALDEFS,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        while self.has(&ID_TYPE) {
            parse_tree.children.push(self.variable_def()?);
        }

        Ok(Some(parse_tree))
    }

    // Return either a VARDEF tree, or an ASSIGN tree
    fn variable_def(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::VARDEF,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        parse_tree.children.push(self.ids()?);

        self.eat(&lexer::TokenType::COLON)?;

        parse_tree.children.push(self.var_type()?);

        if self.has(&lexer::TokenType::EQ) {
            let mut assign_tree = ParseTree {
                parse_type: ParseType::ASSIGN,
                token: NULL_TOKEN.clone(),
                children: Vec::new()
            };

            assign_tree.children.push(Some(parse_tree));
            assign_tree.children.push(self.resolvable()?);
            return Ok(Some(assign_tree));
        }
        
        Ok(Some(parse_tree))
    }

    // Returns either a singular ID tree, or an IDS tree
    fn ids(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let id = self.id()?;

        if !self.has(&lexer::TokenType::COMMA) {
            return Ok(id);
        }

        let mut parse_tree = ParseTree{
            parse_type: ParseType::IDS,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        parse_tree.children.push(id);

        while self.has(&lexer::TokenType::COMMA) {
            self.next()?;

            if self.has(&ID_TYPE) {
                parse_tree.children.push(self.id()?);
            }
            else {
                break;
            }
        }

        Ok(Some(parse_tree))
    }

    //TODO
    fn resolvable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        Ok(Some(parse_tree))
    }

    //TODO
    fn var_type(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        Ok(Some(parse_tree))
    }

    //TODO
    fn function_defs(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        Ok(Some(parse_tree))
    }

    //TODO
    fn program(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        Ok(Some(parse_tree))
    }

    fn foo(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: NULL_TOKEN.clone(),
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

    let tree = p.parse().expect("Error");
    match tree {
        Some(t) => t.print(),
        _ => ()
    };
    // p.next().expect("Boo hoo!");
    // p.must_be(&lexer::TokenType::DEF);
    
    // 
    // while !p.is_done() {
    //     println!("{:?}", p.next());
    // }
}