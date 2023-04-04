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

#[derive(Debug, Clone, PartialEq)]
pub enum ParseType {
    CODE,       // the start of all trees
    DEFINITIONS,// the definitions section of the program
    GLOBALDEFS, // the definition section for global variables
    FUNDEFS,    // the definition section for functions
    BLOCK,      // a block of statements to execute
    FUNDEF,     // function definition
    VARDEF,     // variable definition
    VARDEFS,    // variable definition for a list of ids
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
    ISLINKED,   // conditional for if a variable is linked
    ISNOTLINKED, // conditional for if a varialbe is unliked
    BITNOT,     // bitwise not operation
    NEG,        // negative value operation
    ABS,        // absolute value operation
    RETURN,     // return
    TYPE,       // variable type
    CHANGEABLE, // changeable modifier for types
    LINK,       // link modifier for types
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
    INDEX,         // a set of indecies to get from an array

    INVALID,       // an invalid parse instruction, just for placeholding
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
            match self.token.token_type {
                lexer::TokenType::INVALID => println!{"{:?}", self.parse_type},
                _ => println!{"{:?} ({:?})", self.parse_type, self.token},
            };
            return;
        }

        for child in self.children.iter() {
            if idx == 0 {
                match self.token.token_type {
                    lexer::TokenType::INVALID => println!{"{:?}", self.parse_type},
                    _ => println!{"{:?} ({:?})", self.parse_type, self.token},
                };
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
            for _n in 0..tab {
                print!{"| "};
            }
            match self.token.token_type {
                lexer::TokenType::INVALID => println!{"{:?}", self.parse_type},
                _ => println!{"{:?} ({:?})", self.parse_type, self.token},
            };
            return;
        }

        for child in self.children.iter() {
            if idx == 0 {
                for _n in 0..tab {
                    print!{"| "};
                }
                match self.token.token_type {
                    lexer::TokenType::INVALID => println!{"{:?}", self.parse_type},
                    _ => println!{"{:?} ({:?})", self.parse_type, self.token},
                };
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
    in_fun_def: bool,
    in_loop_block: bool,
    in_if_block: bool,
}

impl Parser {

    // Construct a lexer for the parser from a String
    pub fn new(text: String) -> Result<Self, &'static str> {
        let lexer = lexer::Lexer::new(text)?;
        Ok(Parser {
            lexer: lexer,
            in_fun_def: false,
            in_loop_block: false,
            in_if_block: false,
        })
    }

    // Construct a lexer for the parser from a file
    pub fn from_file(file: String) -> Result<Self, &'static str> {
        let lexer = lexer::Lexer::from_file(file)?;
        Ok(Parser {
            lexer: lexer,
            in_fun_def: false,
            in_loop_block: false,
            in_if_block: false,
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
    fn must_be(&self, token_type: &lexer::TokenType) -> bool {
        if !self.has(token_type) {
            log!("Parse Error on Line: {}, Column: {}", self.curr_token().row, self.curr_token().col);
            log!("Expected: {:?}", token_type);
            log!("Got: {:?}", self.curr_token().token_type);
        }

        true
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
        if !self.has(&lexer::TokenType::STRUCT) {
            return Ok(None);
        }

        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTDEFS,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };
        
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
        if !self.has(&ID_TYPE) {
            return Ok(None);
        }
        
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

    fn function_defs(&mut self) -> Result<Option<ParseTree>, &'static str> {
        if !self.has(&lexer::TokenType::FUN) {
            return Ok(None);
        }
        
        let mut parse_tree = ParseTree{
            parse_type: ParseType::FUNDEFS,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        while self.has(&lexer::TokenType::FUN) {
            parse_tree.children.push(self.function_def()?);
        }

        Ok(Some(parse_tree))
    }

    fn function_def(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.eat(&lexer::TokenType::FUN)?;

        let mut parse_tree = ParseTree{
            parse_type: ParseType::FUNDEF,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        parse_tree.children.push(self.id()?);
        
        self.eat(&lexer::TokenType::LPAREN)?;

        parse_tree.children.push(self.params()?);

        self.eat(&lexer::TokenType::RPAREN)?;
        self.eat(&lexer::TokenType::RETURNS)?;

        parse_tree.children.push(self.fun_return_type()?);

        self.in_fun_def = true;
        parse_tree.children.push(self.statements()?);
        self.in_fun_def = false;

        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::FUN)?;

        Ok(Some(parse_tree))
    }

    fn fun_return_type(&mut self) -> Result<Option<ParseTree>, &'static str> {
        if !self.has(&lexer::TokenType::NOTHING) {
            return self.basic_type();
        }
        
        let mut parse_tree = ParseTree{
            parse_type: ParseType::TYPE,
            token: self.curr_token(),
            children: Vec::new()
        };

        // TYPE expects a modifier child, so give it none
        parse_tree.children.push(None);

        self.next()?;

        Ok(Some(parse_tree))
    }

    fn params(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::PARAMS,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        while self.has(&ID_TYPE) {
            parse_tree.children.push(self.param()?);

            if self.has(&lexer::TokenType::COMMA) {
                self.next()?;
            }
        }

        Ok(Some(parse_tree))
    }

    //TODO
    fn param(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::PARAM,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        parse_tree.children.push(self.id()?);

        self.eat(&lexer::TokenType::COLON)?;


        // Get the changable modifer
        let changeable = match self.curr_token().token_type {
            lexer::TokenType::CHANGEABLE => Some(ParseTree {
                parse_type: ParseType::CHANGEABLE,
                token: self.curr_token(),
                children: Vec::new(),
            }),
            _ => None
        };

        if changeable.is_some() {
            self.next()?;
        }


        Ok(Some(parse_tree))
    }

    fn program(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.eat(&lexer::TokenType::PROGRAM)?;
        
        let body = self.statements()?;

        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::PROGRAM)?;

        Ok(body)
    }

    fn statements(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::BLOCK,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        while !self.has(&lexer::TokenType::END) && 
              !self.has(&lexer::TokenType::EOF) &&
              !(self.in_if_block && self.has(&lexer::TokenType::ELSE)) {
            if self.in_fun_def && self.has(&lexer::TokenType::RETURN) {
                parse_tree.children.push(self.return_statement()?);
                self.next()?;
            }
            else if self.in_loop_block && self.has(&lexer::TokenType::BREAK) {
                let break_tree = ParseTree {
                    parse_type: ParseType::BREAK,
                    token: self.curr_token(),
                    children: Vec::new(),
                };
                self.next()?;
                parse_tree.children.push(Some(break_tree));
            }
            else if self.in_loop_block && self.has(&lexer::TokenType::CONTINUE) {
                let continue_tree = ParseTree {
                    parse_type: ParseType::CONTINUE,
                    token: self.curr_token(),
                    children: Vec::new(),
                };
                self.next()?;
                parse_tree.children.push(Some(continue_tree));
            }
            else {
                parse_tree.children.push(self.statement()?);
            }
        }

        Ok(Some(parse_tree))
    }

    fn statement(&mut self) -> Result<Option<ParseTree>, &'static str> {
        match self.curr_token().token_type {
            lexer::TokenType::ID(_) => self.assign_or_var_def(),
            lexer::TokenType::WHILE => self.while_block(),
            lexer::TokenType::IF => self.if_block(),
            lexer::TokenType::REPEAT => self.repeat(),
            lexer::TokenType::UNLINK => self.unlink(),
            lexer::TokenType::QUIT => self.quit(),
            _ => self.resolvable(),
        }
    }

    fn basic_type(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::TYPE,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        if self.has(&NUMBER_TYPE) |
           self.has(&TEXT_TYPE) {
            self.next()?;
        }
        else {
            // we just need to eat the ID here, we already have it stored
            self.eat(&ID_TYPE)?;
        }

        Ok(Some(parse_tree))
    }

    fn return_statement(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        self.eat(&lexer::TokenType::RETURN)?;

        // Return nothing, needs keyword nothing
        if self.has(&lexer::TokenType::NOTHING) {
            self.next()?;
            parse_tree.children.push(None);
        }
        // Returns a resolvable statement
        else {
            parse_tree.children.push(self.resolvable()?);
        }

        Ok(Some(parse_tree))
    }

    //TODO
    fn assign_or_var_def(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut id_tree = self.ids()?;

        // handle special < var-def > for multiple ids
        if id_tree.clone().unwrap().parse_type == ParseType::IDS {
            self.eat(&lexer::TokenType::COLON)?;
            let mut parse_tree = ParseTree {
                parse_type: ParseType::VARDEFS,
                token: NULL_TOKEN.clone(),
                children: Vec::new(),
            };
            parse_tree.children.push(id_tree);
            parse_tree.children.push(self.var_type()?);
            // don't allow for multiple assignment, only type def
            return Ok(Some(parse_tree));
        }

        // < assignment >
        if self.has(&lexer::TokenType::EQ) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::ASSIGN,
                token: NULL_TOKEN.clone(),
                children: Vec::new(),
            };
            self.next()?;
            parse_tree.children.push(id_tree);
            parse_tree.children.push(self.resolvable()?);
            return Ok(Some(parse_tree));
        }
        // < var-def >
        else if self.has(&lexer::TokenType::COLON) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::VARDEF,
                token: NULL_TOKEN.clone(),
                children: Vec::new(),
            };
            parse_tree.children.push(id_tree);
            parse_tree.children.push(self.var_type()?);

            // < var-def-equal >
            if self.has(&lexer::TokenType::EQ) {
                let mut assign_tree = ParseTree {
                    parse_type: ParseType::ASSIGN,
                    token: NULL_TOKEN.clone(),
                    children: Vec::new(),
                };
                self.next()?;
                assign_tree.children.push(Some(parse_tree));
                assign_tree.children.push(self.resolvable()?);
                return Ok(Some(assign_tree));
            }

            return Ok(Some(parse_tree));
        }

        // < resolvable' >
        self.resolvable2(id_tree)
    }

    fn while_block(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.eat(&lexer::TokenType::WHILE)?;
        
        let mut parse_tree = ParseTree{
            parse_type: ParseType::WHILE,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        parse_tree.children.push(self.condition()?);

        self.in_loop_block = true;
        parse_tree.children.push(self.statements()?);
        self.in_loop_block = false;

        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::WHILE)?;

        Ok(Some(parse_tree))
    }

    fn if_block(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.eat(&lexer::TokenType::IF)?;
        
        let mut parse_tree = ParseTree{
            parse_type: ParseType::IF,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        parse_tree.children.push(self.condition()?);
        
        self.eat(&lexer::TokenType::THEN)?;

        self.in_if_block = true;
        parse_tree.children.push(self.statements()?);
        self.in_if_block = false;

        parse_tree.children.push(self.if_block_2()?);

        Ok(Some(parse_tree))
    }

    fn if_block_2(&mut self) -> Result<Option<ParseTree>, &'static str> {
        
        if self.has(&lexer::TokenType::END) {
            self.eat(&lexer::TokenType::END)?;
            self.eat(&lexer::TokenType::IF)?;
            return Ok(None);
        }

        self.eat(&lexer::TokenType::ELSE)?;

        if self.has(&lexer::TokenType::IF) {
            return self.if_block();
        }

        // we don't need to set self.in_if_block here
        // because we shouldn't be seeing another else
        // if this is final else
        let else_block = self.statements()?;

        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::IF)?;

        Ok(else_block)
    }

    fn condition(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < condition >
        let mut left = self.logic_andable()?;

        let mut parse_tree: ParseTree;

        // < condition' >
        while self.has(&lexer::TokenType::OR) {
            parse_tree = ParseTree {
                parse_type: ParseType::BINCOMP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            parse_tree.children.push(self.logic_andable()?);
            left = Some(parse_tree);
        }

        Ok(left)
    }

    // TODO
    fn logic_andable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < logic-paren >
        let mut left = self.logic_paren()?;

        let mut parse_tree: ParseTree;

        // < logic-andable' >
        while self.has(&lexer::TokenType::AND) {
            parse_tree = ParseTree {
                parse_type: ParseType::BINCOMP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            parse_tree.children.push(self.logic_paren()?);
            left = Some(parse_tree);
        }

        Ok(left)
    }

    fn logic_paren(&mut self) -> Result<Option<ParseTree>, &'static str> {
        if self.has(&lexer::TokenType::LPAREN) {
            self.next()?;
            let condition_tree = self.condition();
            self.eat(&lexer::TokenType::RPAREN)?;
            return condition_tree;
        }

        self.comparable()
    }

    // TODO
    fn comparable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut left: Option<ParseTree> = Some( ParseTree {
            parse_type: ParseType::INVALID,
            token: NULL_TOKEN.clone(),
            children: Vec::new(),
        });
        
        // < comparable >
        if self.has(&ID_TYPE) {
            let id_tree = self.id()?;

            // < comparable' >
            if self.has(&lexer::TokenType::IS) {
                // < linked or not >
                self.next()?;
                if self.has(&lexer::TokenType::LINKED) {
                    let mut parse_tree = ParseTree {
                        parse_type: ParseType::ISLINKED,
                        token:  self.curr_token(),
                        children: Vec::new(),
                    };
                    parse_tree.children.push(self.reference2(id_tree)?);
                    self.next()?;
                    return Ok(Some(parse_tree));
                }
                else {
                    self.eat(&lexer::TokenType::WORDNOT)?;
                    let mut parse_tree = ParseTree {
                        parse_type: ParseType::ISNOTLINKED,
                        token: self.curr_token(),
                        children: Vec::new(),
                    };
                    parse_tree.children.push(self.reference2(id_tree)?);
                    self.eat(&lexer::TokenType::LINKED)?;
                    return Ok(Some(parse_tree));
                }
            }
            else {
                left = self.resolvable2(id_tree)?;
            }
        }

        let mut parse_tree: ParseTree = ParseTree {
            parse_type: ParseType::INVALID,
            token: NULL_TOKEN.clone(),
            children: Vec::new(),
        };

        // < comparable'' >
        if self.has(&lexer::TokenType::GT) ||
           self.has(&lexer::TokenType::LT) ||
           self.has(&lexer::TokenType::GE) ||
           self.has(&lexer::TokenType::LE) ||
           self.has(&lexer::TokenType::EQ) ||
           self.must_be(&lexer::TokenType::NE) {
            parse_tree = ParseTree {
                parse_type: ParseType::BINCOMP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.resolvable()?);
        }

        Ok(Some(parse_tree))
    }

    //TODO
    fn repeat(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        Ok(Some(parse_tree))
    }

    fn unlink(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.eat(&lexer::TokenType::UNLINK)?;

        let mut parse_tree = ParseTree{
            parse_type: ParseType::UNLINK,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        parse_tree.children.push(self.reference()?);

        Ok(Some(parse_tree))
    }

    fn quit(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.eat(&lexer::TokenType::QUIT)?;

        let mut parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: NULL_TOKEN.clone(),
            children: Vec::new()
        };

        Ok(Some(parse_tree))
    }

    fn reference(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let id_tree = self.id()?;
        self.reference2(id_tree)
    }

    //TODO
    fn reference2(&mut self, id_tree: Option<ParseTree>) -> Result<Option<ParseTree>, &'static str> {
        if self.has(&lexer::TokenType::LBRACKET) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::GETINDEX,
                token: NULL_TOKEN.clone(),
                children: Vec::new(),
            };
            self.next()?;
            parse_tree.children.push(id_tree);
            parse_tree.children.push(self.index()?);
            self.eat(&lexer::TokenType::RBRACKET);

            return self.reference2(Some(parse_tree));
        }
        else if self.has(&lexer::TokenType::PERIOD) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::GETSTRUCT,
                token: NULL_TOKEN.clone(),
                children: Vec::new(),
            };
            parse_tree.children.push(id_tree);
            parse_tree.children.push(self.id()?);
            return self.reference2(Some(parse_tree));
        }

        Ok(id_tree)
    }


    fn index(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::INDEX,
            token: self.curr_token(),
            children: Vec::new()
        };

        while !self.has(&lexer::TokenType::RBRACKET) {
            parse_tree.children.push(self.resolvable()?);

            if self.has(&lexer::TokenType::COMMA) {
                self.next()?;
            }
        }

        Ok(Some(parse_tree))
    }

    fn resolvable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.bit_orable()
    }

    //TODO
    fn resolvable2(&mut self, id_tree: Option<ParseTree>) -> Result<Option<ParseTree>, &'static str> {
        // < ref-or-call >
        let mut left = self.ref_or_call(id_tree)?;

        // < factor' >
        if self.has(&lexer::TokenType::POW){
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.factor()?);
            left = Some(parse_tree);
        }

        // < term' >
        while self.has(&lexer::TokenType::MUL) ||
              self.has(&lexer::TokenType::DIV) ||
              self.has(&lexer::TokenType::MOD) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.factor()?);
            left = Some(parse_tree);
        }

        // < expression' >
        while self.has(&lexer::TokenType::ADD) ||
              self.has(&lexer::TokenType::SUB) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.term()?);
            left = Some(parse_tree);
        }

        // < bit-shiftable' >
        while self.has(&lexer::TokenType::BSL) ||
           self.has(&lexer::TokenType::BSR) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.expression()?);
            left = Some(parse_tree);
        }

        // < bit-andable' >
        while self.has(&lexer::TokenType::BAND) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.bit_shiftable()?);
            left = Some(parse_tree);
        }

        // < bit-xorable' >
        while self.has(&lexer::TokenType::BXOR) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.bit_andable()?);
            left = Some(parse_tree);
        }

        // < bit-orable' >
        while self.has(&lexer::TokenType::BOR) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.bit_xorable()?);
            left = Some(parse_tree);
        }
        
        Ok(left)
    }

    fn arg_list(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.eat(&lexer::TokenType::LPAREN)?;

        // handle the empty argument list
        if self.has(&lexer::TokenType::RPAREN) {
            self.next()?;
            return Ok(None);
        }

        let mut parse_tree = ParseTree {
            parse_type: ParseType::ARGS,
            token: NULL_TOKEN.clone(),
            children: Vec::new(),
        };

        while !self.has(&lexer::TokenType::RPAREN) {
            parse_tree.children.push(self.resolvable()?);

            if self.has(&lexer::TokenType::COMMA) {
                self.next()?;
            }
        }

        self.eat(&lexer::TokenType::RPAREN)?;

        Ok(Some(parse_tree))
    }

    fn bit_orable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < bit-orable >
        let mut left = self.bit_xorable()?;


        // < bit-orable' >
        while self.has(&lexer::TokenType::BOR) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.bit_xorable()?);
            left = Some(parse_tree);
        }

        Ok(left)
    }

    fn bit_xorable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < bit-xorable >
        let mut left = self.bit_andable()?;


        // < bit-xorable' >
        while self.has(&lexer::TokenType::BXOR) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.bit_andable()?);
            left = Some(parse_tree);
        }

        Ok(left)
    }

    fn bit_andable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < bit-andable >
        let mut left = self.bit_shiftable()?;


        // < bit-andable' >
        while self.has(&lexer::TokenType::BAND) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.bit_shiftable()?);
            left = Some(parse_tree);
        }

        Ok(left)
    }

    fn bit_shiftable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < bit-shiftable >
        let mut left = self.expression()?;


        // < bit-shiftable' >
        while self.has(&lexer::TokenType::BSL) ||
           self.has(&lexer::TokenType::BSR) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.expression()?);
            left = Some(parse_tree);
        }

        Ok(left)
    }

    fn expression(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < expression >
        let mut left = self.term()?;


        // < expression' >
        while self.has(&lexer::TokenType::ADD) ||
              self.has(&lexer::TokenType::SUB) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.term()?);
            left = Some(parse_tree);
        }

        Ok(left)
    }

    fn term(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < term >
        let mut left = self.factor()?;


        // < term' >
        while self.has(&lexer::TokenType::MUL) ||
              self.has(&lexer::TokenType::DIV) ||
              self.has(&lexer::TokenType::MOD) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.factor()?);
            left = Some(parse_tree);
        }

        Ok(left)
    }

    fn factor(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < factor >
        if self.has(&lexer::TokenType::SUB) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::NEG,
                token: self.curr_token(),
                children: Vec::new(),
            };
            self.next()?;
            parse_tree.children.push(self.factor()?);
            return Ok(Some(parse_tree));
        }

        let mut left = self.bit_notable()?;

        // < factor' >
        if self.has(&lexer::TokenType::POW){
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BINOP,
                token: self.curr_token(),
                children: Vec::new(),
            };
            parse_tree.children.push(left);
            self.next()?;
            parse_tree.children.push(self.factor()?);
            left = Some(parse_tree);
        }

        Ok(left)
    }

    fn bit_notable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < bit-notable >
        if self.has(&lexer::TokenType::BNOT) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::BITNOT,
                token: self.curr_token(),
                children: Vec::new(),
            };
            self.next()?;
            parse_tree.children.push(self.exponent()?);
            return Ok(Some(parse_tree));
        }

        self.exponent()
    }

    fn exponent(&mut self) -> Result<Option<ParseTree>, &'static str> {

        if self.has(&lexer::TokenType::ADD) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::ABS,
                token: self.curr_token(),
                children: Vec::new(),
            };
            self.next()?;
            parse_tree.children.push(self.exponent()?);
            return Ok(Some(parse_tree));
        }
        else if self.has(&lexer::TokenType::LPAREN) {
            self.next()?;
            let resolvable_tree = self.resolvable();
            self.eat(&lexer::TokenType::RPAREN);
            return resolvable_tree;
        }
        else if self.has(&NUMBER_TYPE) ||
                self.has(&TEXT_TYPE)   {
            let parse_tree = ParseTree {
                parse_type: ParseType::LIT,
                token: self.curr_token(),
                children: Vec::new()
            };
            self.next()?;
            return Ok(Some(parse_tree))
        }
        else if self.has(&ID_TYPE) {
            let id_tree = self.id()?;
            return self.ref_or_call(id_tree);
        }
        else if self.has(&lexer::TokenType::LCURLY) {
            return self.array_struct_lit();
        }
        else {
            println!("{:?}\nI don't know how we got here", self.curr_token());
            self.next();
            return Ok(None);
        }
    }

    //TODO
    fn ref_or_call(&mut self, id_tree: Option<ParseTree>) -> Result<Option<ParseTree>, &'static str> {
        // < call >
        if self.has(&lexer::TokenType::LPAREN) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::CALL,
                token: NULL_TOKEN.clone(),
                children: Vec::new(),
            };
            parse_tree.children.push(id_tree);
            // < arg-list > techinically handles the RPAREN
            // and the case of no arguments, though they should
            // be handled here
            parse_tree.children.push(self.arg_list()?);
            return Ok(Some(parse_tree));
        }

        // < reference >
        self.reference2(id_tree)
    }

    //TODO
    fn array_struct_lit(&mut self) -> Result<Option<ParseTree>, &'static str> {
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
}