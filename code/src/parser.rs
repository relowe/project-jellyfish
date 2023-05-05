#![allow(dead_code)]

use std::{env, process};
use crate::lexer;
use std::mem;

// This null token can be used as a token placeholder
//  elsewhere througout the code
static NULL_TOKEN: lexer::Token = lexer::Token {
    row: 0,
    col: 0,
    token_type: lexer::TokenType::INVALID,
    lexeme: None
};

// For the token types with values, we need a standard
//  token with a value to compare to
static ID_TYPE: lexer::TokenType = lexer::TokenType::ID(String::new());
static TEXT_TYPE: lexer::TokenType = lexer::TokenType::TEXT(String::new());
static NUMBER_TYPE: lexer::TokenType = lexer::TokenType::NUMBER(0.0);


// Handle error reporting through web assembly
// For right now we just print the error, but later
//  on this would be passed to JavaScript code
macro_rules! log {
    ($($t:tt)*) => (println!("{}",  &format_args!($ ( $t ) *).to_string() ))
}

// These types represents nodes in the parse tree
// Each node is something some action that the
//  interpreter will perform, or an object that
//  the interpreter will work with.
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
    REPEATFOREVER, // repeat forever
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
    POINTER,    // type for pointers specifically
    CHANGEABLE, // changeable modifier for types
    LINK,       // link modifier for types
    LINKLIT,    // a literal used to denote that a reference is being linked
    ARRAYLIT,   // array literal
    STRUCTLIT,  // structure literal
    ARRAYDEF,   // define an array variable with (optional) bounds and type
    BOUNDS,     // bounds for an array
    BOUND,      // a single bound for an array
    GETINDEX,      // return the index of an array
    GETSTRUCT,     // get the value of a structure key
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

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree {
    pub parse_type: ParseType,
    pub token: lexer::Token,
    pub children: Vec<Option<ParseTree>>,
}

impl ParseTree {
    // A function to recursively print the entire parse
    //  tree and its children.
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

    // A helper function to print the parse tree,
    //  making sure all nodes are properly indented
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
    in_loop_block: i32,
    in_if_block: i32,
}

impl Parser {
    // Construct a lexer for the parser from a String
    pub fn new(text: String) -> Result<Self, &'static str> {
        let lexer = lexer::Lexer::new(text)?;
        Ok(Parser {
            lexer: lexer,
            in_fun_def: false,
            in_loop_block: 0,
            in_if_block: 0,
        })
    }

    // Construct a lexer for the parser from a file
    pub fn from_file(file: String) -> Result<Self, &'static str> {
        let lexer = lexer::Lexer::from_file(file)?;
        Ok(Parser {
            lexer: lexer,
            in_fun_def: false,
            in_loop_block: 0,
            in_if_block: 0,
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

    // Return a boolean on if the current token matches
    //  the expected token
    fn has(&self, token_type: &lexer::TokenType) -> bool {
        /* self.lexer.curr_token.token_type == token_type */

        // Need to use Rust's discriminant function to
        //  analyze the token types that have values
        //  associated with them
        mem::discriminant(&self.curr_token().token_type) == mem::discriminant(token_type)
    }

    // Test if the current token matches the expected token
    // If not, log the error and close the program
    // For now, we just exit, but this can be handled
    //  differently in WebAssembly
    fn must_be(&self, token_type: &lexer::TokenType) -> bool {
        if !self.has(token_type) {
            log!("Parse Error on Line: {}, Column: {}", self.curr_token().row, self.curr_token().col);
            log!("Expected: {:?}", token_type);
            log!("Got: {:?}", self.curr_token().token_type);
            process::exit(0);
        }
        true
    }

    // Helper function to call 'must_be' and 'next' together
    fn eat(&mut self, token_type: &lexer::TokenType) -> Result<lexer::Token, &'static str> {
        self.must_be(token_type);
        self.next()
    }

    // Run the parser
    pub fn parse(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.code()
    }

    // < CODE >
    fn code(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.next()?;
        let mut parse_tree = ParseTree{
            parse_type: ParseType::CODE,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // [DEF < definitions >]?
        if self.has(&lexer::TokenType::DEF) {
            self.next()?;
            parse_tree.children.push(self.definitions()?);
        }
        else {
            parse_tree.children.push(None);
        }

        // < program >
        parse_tree.children.push(self.program()?);

        Ok(Some(parse_tree))
    }

    // < definitions >
    fn definitions(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::DEFINITIONS,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // Will check for STRUCT token and return None if there isn't one
        // < structure-defs >
        parse_tree.children.push(self.structure_defs()?);

        // Will check for ID token and return None if there isn't one
        // < global-defs >
        parse_tree.children.push(self.global_defs()?);

        // Will check for FUN token and return None if there isn't one
        // < function-defs >
        parse_tree.children.push(self.function_defs()?);

        // END DEF
        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::DEF)?;

        Ok(Some(parse_tree))
    }

    // < structure-defs >
    fn structure_defs(&mut self) -> Result<Option<ParseTree>, &'static str> {
        if !self.has(&lexer::TokenType::STRUCT) {
            return Ok(None);
        }

        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTDEFS,
            token: self.curr_token(), 
            children: Vec::new()
        };
        
        // [< structure-def >]*
        while self.has(&lexer::TokenType::STRUCT) {
            parse_tree.children.push(self.structure_def()?);
        }

        Ok(Some(parse_tree))
    }

    // < structure-def >
    fn structure_def(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTDEF,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // STRUCT
        self.eat(&lexer::TokenType::STRUCT)?;

        // ID
        parse_tree.children.push(self.id()?);

        // < structure-arg >
        // Catch the requirement, but we don't process it here
        self.must_be(&ID_TYPE);

        // < structure-args >
        parse_tree.children.push(self.structure_args()?);

        // END STRUCT
        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::STRUCT)?;

        Ok(Some(parse_tree))
    }
    
    // ID
    fn id(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // ID
        self.must_be(&ID_TYPE);

        let parse_tree = ParseTree{
            parse_type: ParseType::ID,
            token: self.curr_token(),
            children: Vec::new()
        };

        self.next()?;
        Ok(Some(parse_tree))
    }

    // < structure-args >
    fn structure_args(&mut self) -> Result<Option<ParseTree>, &'static str> {   
        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTARGS,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // [< structure-arg >]*
        while self.has(&ID_TYPE) {
            parse_tree.children.push(self.structure_arg()?);
        }

        Ok(Some(parse_tree))
    }

    // < structure-arg >
    fn structure_arg(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTARG,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // ID
        parse_tree.children.push(self.id()?);

        // COLON
        self.eat(&lexer::TokenType::COLON)?;

        // < variable-type >
        parse_tree.children.push(self.var_type()?);

        // < var-def-equal >
        // But ran internally
        // [EQ]?
        if self.has(&lexer::TokenType::EQ) {
            self.next()?;

            // < resolvable >
            parse_tree.children.push(self.resolvable()?);
        }
        else {
            parse_tree.children.push(None);
        }
        
        Ok(Some(parse_tree))
    }

    // < global-defs >
    fn global_defs(&mut self) -> Result<Option<ParseTree>, &'static str> {
        if !self.has(&ID_TYPE) {
            return Ok(None);
        }
        
        let mut parse_tree = ParseTree{
            parse_type: ParseType::GLOBALDEFS,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // [< global-def >]*
        // Here we use a boolean to signify if
        //  assign_or_var_def should also look for
        //  resolvable' at the end, this helps
        //  to not repeat code
        while self.has(&ID_TYPE) {
            parse_tree.children.push(self.assign_or_var_def(false)?);
        }

        Ok(Some(parse_tree))
    }

    // Returns either a singular ID tree, or an IDS tree
    fn ids(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // ID
        let id = self.id()?;

        // This catches just the single ID
        if !self.has(&lexer::TokenType::COMMA) {
            return Ok(id);
        }

        let mut parse_tree = ParseTree{
            parse_type: ParseType::IDS,
            token: self.curr_token(), 
            children: Vec::new()
        };

        parse_tree.children.push(id);

        // < ids' >
        while self.has(&lexer::TokenType::COMMA) {
            self.next()?;

            // ID
            if self.has(&ID_TYPE) {
                parse_tree.children.push(self.id()?);
            }
            else {
                break;
            }
        }

        Ok(Some(parse_tree))
    }

    // < function-defs >
    fn function_defs(&mut self) -> Result<Option<ParseTree>, &'static str> {
        if !self.has(&lexer::TokenType::FUN) {
            return Ok(None);
        }
        
        let mut parse_tree = ParseTree{
            parse_type: ParseType::FUNDEFS,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // [< function-def >]*
        while self.has(&lexer::TokenType::FUN) {
            parse_tree.children.push(self.function_def()?);
        }

        Ok(Some(parse_tree))
    }

    // < function-def >
    fn function_def(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // FUN
        self.eat(&lexer::TokenType::FUN)?;

        let mut parse_tree = ParseTree{
            parse_type: ParseType::FUNDEF,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // ID
        parse_tree.children.push(self.id()?);
        
        // LPAREN
        self.eat(&lexer::TokenType::LPAREN)?;

        // < parameters >
        parse_tree.children.push(self.params()?);

        // RPAREN RETURNS
        self.eat(&lexer::TokenType::RPAREN)?;
        self.eat(&lexer::TokenType::RETURNS)?;

        // < fun-return-type >
        parse_tree.children.push(self.fun_return_type()?);

        // We cheat having to double up on our BNF by
        //  setting a flag for when we enter a function
        //  definition
        // Otherwise "RETURN" statements wouldn't be
        //  caught properly
        self.in_fun_def = true;
        // < fun-statements >
        parse_tree.children.push(self.statements()?);
        self.in_fun_def = false;

        // ENF FUN
        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::FUN)?;

        Ok(Some(parse_tree))
    }

    // < fun-return-type >
    fn fun_return_type(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // < basic-type >
        if !self.has(&lexer::TokenType::NOTHING) {
            return self.fun_arg_type();
        }
        
        // NOTHING
        let parse_tree = ParseTree{
            parse_type: ParseType::TYPE,
            token: self.curr_token(),
            children: Vec::new()
        };

        self.next()?;

        Ok(Some(parse_tree))
    }

    // < parameters >
    fn params(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::PARAMS,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // [< parameter >]*
        while self.has(&ID_TYPE) {
            parse_tree.children.push(self.param()?);

            if self.has(&lexer::TokenType::COMMA) {
                self.next()?;
            }
            else {
                break;
            }
        }

        Ok(Some(parse_tree))
    }

    // < parameter >
    fn param(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::PARAM,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // ID COLON
        parse_tree.children.push(self.id()?);
        self.eat(&lexer::TokenType::COLON)?;

        // [CHANGEABLE]?
        let type_tree: Option<ParseTree>;
        if self.has(&lexer::TokenType::CHANGEABLE) {
            let mut changeable_tree = ParseTree {
                parse_type: ParseType::POINTER,
                token: self.curr_token(),
                children: Vec::new(),
            };
            self.next()?;

            // < fun-arg-type >
            changeable_tree.children.push(self.fun_arg_type()?);

            type_tree = Some(changeable_tree);
        }
        // < fun-arg-type >
        else {
            type_tree = self.fun_arg_type()?;
        }

        parse_tree.children.push(type_tree);

        Ok(Some(parse_tree))
    }

    // < program >
    fn program(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // PROGRAM
        self.eat(&lexer::TokenType::PROGRAM)?;
        
        // < statements >
        let body = self.statements()?;

        // END PROGRAM
        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::PROGRAM)?;

        Ok(body)
    }

    // < statements >
    fn statements(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::BLOCK,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // [< statement >]*
        // Also catches when certain end conditions are reached
        //  like the "END" of a loop, or if we hit EOF early
        while !(self.has(&lexer::TokenType::END) || 
              self.has(&lexer::TokenType::EOF)) {
            if self.in_if_block > 0 && self.has(&lexer::TokenType::ELSE) {
                break;
            }

            // < fun-statement > (for return specifically)
            if self.in_fun_def && self.has(&lexer::TokenType::RETURN) {
                parse_tree.children.push(self.return_statement()?);
            }
            // < loop-statements > (for BREAK)
            else if self.in_loop_block > 0 && self.has(&lexer::TokenType::BREAK) {
                let break_tree = ParseTree {
                    parse_type: ParseType::BREAK,
                    token: self.curr_token(),
                    children: Vec::new(),
                };
                self.next()?;
                parse_tree.children.push(Some(break_tree));
            }
            // < loop-statements > (for CONTINUE)
            else if self.in_loop_block > 0 && self.has(&lexer::TokenType::CONTINUE) {
                let continue_tree = ParseTree {
                    parse_type: ParseType::CONTINUE,
                    token: self.curr_token(),
                    children: Vec::new(),
                };
                self.next()?;
                parse_tree.children.push(Some(continue_tree));
            }
            // < statement >
            else {
                parse_tree.children.push(self.statement()?);
            }
        }

        Ok(Some(parse_tree))
    }

    // < statement >
    // This just runs a match statement with the first token
    fn statement(&mut self) -> Result<Option<ParseTree>, &'static str> {
        match self.curr_token().token_type {
            lexer::TokenType::ID(_) => self.assign_or_var_def(true),
            lexer::TokenType::WHILE => self.while_block(),
            lexer::TokenType::IF => self.if_block(),
            lexer::TokenType::REPEAT => self.repeat(),
            lexer::TokenType::UNLINK => self.unlink(),
            lexer::TokenType::QUIT => self.quit(),
            _ => self.resolvable(),
        }
    }

    // RETURN < return-value >
    fn return_statement(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::RETURN,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // RETURN
        self.eat(&lexer::TokenType::RETURN)?;

        // < return-value > ==> NOTHING
        if self.has(&lexer::TokenType::NOTHING) {
            self.next()?;
            parse_tree.children.push(None);
        }

        // < return-value > ==> < resolvable >
        else {
            parse_tree.children.push(self.resolvable()?);
        }

        Ok(Some(parse_tree))
    }

    // < asg or var-def or res > and < asg or var-def >
    fn assign_or_var_def(&mut self, go_to_resolvable: bool) -> Result<Option<ParseTree>, &'static str> {
        let mut id_tree = self.ids()?;

        // handle special < var-def > for multiple ids
        if id_tree.clone().unwrap().parse_type == ParseType::IDS {
            self.eat(&lexer::TokenType::COLON)?;
            let mut parse_tree = ParseTree {
                parse_type: ParseType::VARDEFS,
                token: self.curr_token(), 
                children: Vec::new(),
            };
            parse_tree.children.push(id_tree);
            parse_tree.children.push(self.var_type()?);
            // don't allow for multiple assignment, only type def
            return Ok(Some(parse_tree));
        }

        // < var-def >
        if self.has(&lexer::TokenType::COLON) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::VARDEF,
                token: self.curr_token(), 
                children: Vec::new(),
            };
            self.next()?;

            parse_tree.children.push(id_tree);
            parse_tree.children.push(self.var_type()?);

            // < var-def-equal >
            if self.has(&lexer::TokenType::EQ) {
                let mut assign_tree = ParseTree {
                    parse_type: ParseType::ASSIGN,
                    token: self.curr_token(), 
                    children: Vec::new(),
                };
                self.next()?;

                assign_tree.children.push(Some(parse_tree));

                assign_tree.children.push(self.resolvable()?);
                return Ok(Some(assign_tree));
            }

            return Ok(Some(parse_tree));
        }

        // < assignment >
        id_tree = self.reference2(id_tree)?;
        if self.has(&lexer::TokenType::EQ) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::ASSIGN,
                token: self.curr_token(), 
                children: Vec::new(),
            };
            self.next()?;
            parse_tree.children.push(id_tree);
            parse_tree.children.push(self.resolvable()?);
            return Ok(Some(parse_tree));
        }

        // < resolvable' >
        if go_to_resolvable {
            self.resolvable2(id_tree)
        }
        else {
            self.must_be(&lexer::TokenType::INVALID);
            Ok(None)
        }
    }

    // < while >
    fn while_block(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // WHILE
        self.eat(&lexer::TokenType::WHILE)?;
        
        let mut parse_tree = ParseTree{
            parse_type: ParseType::WHILE,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // < condition >
        parse_tree.children.push(self.condition()?);

        // < loop-statements >
        // Using a flag to mark when we are in a loop
        //  helps reduce the BNF by not requireing
        //  more special function for BREAK and CONTINUE
        self.in_loop_block += 1;
        parse_tree.children.push(self.statements()?);
        self.in_loop_block -= 1;

        // END WHILE
        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::WHILE)?;

        Ok(Some(parse_tree))
    }

    // < if >
    fn if_block(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // IF
        self.eat(&lexer::TokenType::IF)?;
        
        let mut parse_tree = ParseTree{
            parse_type: ParseType::IF,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // < condition > THEN
        parse_tree.children.push(self.condition()?);
        self.eat(&lexer::TokenType::THEN)?;

        // < statements >
        // Using a flag to mark when we are in an if
        //  statement to make up for the < if2 elif >
        //  non-terminal
        self.in_if_block += 1;
        parse_tree.children.push(self.statements()?);
        self.in_if_block -= 1;

        // < if2 >
        parse_tree.children.push(self.if_block_2()?);

        Ok(Some(parse_tree))
    }

    // < if2 > and < if2 elif > put together
    fn if_block_2(&mut self) -> Result<Option<ParseTree>, &'static str> {
        
        // END IF
        if self.has(&lexer::TokenType::END) {
            self.eat(&lexer::TokenType::END)?;
            self.eat(&lexer::TokenType::IF)?;
            return Ok(None);
        }

        // ELSE
        self.eat(&lexer::TokenType::ELSE)?;

        // < if2 elif > is handled by < if >
        //  thanks to the in_if_block flag
        if self.has(&lexer::TokenType::IF) {
            return self.if_block();
        }

        // we don't need to set self.in_if_block here
        // because we shouldn't be seeing another else
        // if this is final else
        // < statements >
        let else_block = self.statements()?;

        // END IF
        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::IF)?;

        Ok(else_block)
    }

    // < condition >
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

    // < logic-andable >
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

    // < logic-paren >
    fn logic_paren(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // LPAREN < condition > RPAREN
        if self.has(&lexer::TokenType::LPAREN) {
            self.next()?;
            let condition_tree = self.condition();
            self.eat(&lexer::TokenType::RPAREN)?;
            return condition_tree;
        }

        // < comparable >
        self.comparable()
    }

    // < comparable >
    fn comparable(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut left: Option<ParseTree> = Some( ParseTree {
            parse_type: ParseType::INVALID,
            token: self.curr_token(), 
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
                // < resolvable >
                left = self.resolvable2(id_tree)?;
            }
        }

        let mut parse_tree: ParseTree = ParseTree {
            parse_type: ParseType::INVALID,
            token: self.curr_token(), 
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

    // < repeat >
    fn repeat(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // REPEAT
        self.eat(&lexer::TokenType::REPEAT)?;

        let mut parse_tree;
        
        // FOREVER
        if self.has(&lexer::TokenType::FOREVER) {
            self.eat(&lexer::TokenType::FOREVER)?;

            parse_tree = ParseTree {
                parse_type: ParseType::REPEATFOREVER,
                token: self.curr_token(), 
                children: Vec::new(),
            };
        }

        // FOR ALL ID IN < resolvable >
        else if self.has(&lexer::TokenType::FOR) {
            self.eat(&lexer::TokenType::FOR)?;
            self.eat(&lexer::TokenType::ALL)?;

            parse_tree = ParseTree {
                parse_type: ParseType::REPEATFOR,
                token: self.curr_token(), 
                children: Vec::new(),
            };

            // ID
            parse_tree.children.push(self.id()?);

            // IN
            self.eat(&lexer::TokenType::IN)?;

            // < resolvable >
            parse_tree.children.push(self.resolvable()?);
        }

        // < resolvable > TIMES
        else {
            parse_tree = ParseTree {
                parse_type: ParseType::REPEAT,
                token: self.curr_token(), 
                children: Vec::new(),
            };

            // < resolvable >
            parse_tree.children.push(self.resolvable()?);

            // TIMES
            self.eat(&lexer::TokenType::TIMES)?;
        }

        // < loop-statements >
        // Using a flag to mark when we are in a loop
        //  helps reduce the BNF by not requireing
        //  more special function for BREAK and CONTINUE
        self.in_loop_block += 1;
        parse_tree.children.push(self.statements()?);
        self.in_loop_block -= 1;

        // END REPEAT
        self.eat(&lexer::TokenType::END)?;
        self.eat(&lexer::TokenType::REPEAT)?;

        Ok(Some(parse_tree))
    }

    // < unlink >
    fn unlink(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // UNLINK
        self.eat(&lexer::TokenType::UNLINK)?;

        let mut parse_tree = ParseTree{
            parse_type: ParseType::UNLINK,
            token: self.curr_token(), 
            children: Vec::new()
        };

        // < reference >
        parse_tree.children.push(self.reference()?);

        Ok(Some(parse_tree))
    }

    // QUIT
    fn quit(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // QUIT
        self.eat(&lexer::TokenType::QUIT)?;

        let parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: self.curr_token(), 
            children: Vec::new()
        };

        Ok(Some(parse_tree))
    }

    
    fn reference(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let id_tree = self.id()?;
        self.reference2(id_tree)
    }

    fn reference2(&mut self, id_tree: Option<ParseTree>) -> Result<Option<ParseTree>, &'static str> {
        if self.has(&lexer::TokenType::LBRACKET) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::GETINDEX,
                token: self.curr_token(), 
                children: Vec::new(),
            };
            self.next()?;
            parse_tree.children.push(id_tree);
            parse_tree.children.push(self.index()?);
            self.eat(&lexer::TokenType::RBRACKET)?;

            return self.reference2(Some(parse_tree));
        }
        else if self.has(&lexer::TokenType::PERIOD) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::GETSTRUCT,
                token: self.curr_token(), 
                children: Vec::new(),
            };
            self.next()?;
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
            token: self.curr_token(), 
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
            self.eat(&lexer::TokenType::RPAREN)?;
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
            return self.struct_lit();
        }
        else if self.has(&lexer::TokenType::LBRACKET) {
            return self.array_lit();
        }
        else {
            //println!{"==============================="};
            //println!("{:?}\nI don't know how we got here", self.curr_token());
            //println!{"==============================="};
            self.must_be(&NUMBER_TYPE);
            return Ok(Some(ParseTree{
                parse_type: ParseType::INVALID,
                token: self.curr_token(),
                children: Vec::new(),
            }));
        }
    }

    fn ref_or_call(&mut self, id_tree: Option<ParseTree>) -> Result<Option<ParseTree>, &'static str> {
        // < call >
        if self.has(&lexer::TokenType::LPAREN) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::CALL,
                token: self.curr_token(), 
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

    fn array_lit(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::ARRAYLIT,
            token: self.curr_token(), 
            children: Vec::new()
        };

        self.eat(&lexer::TokenType::LBRACKET)?;

        while !self.has(&lexer::TokenType::RBRACKET) {
            parse_tree.children.push(self.resolvable()?);

            if self.has(&lexer::TokenType::COMMA) {
                self.next()?;
            }
            else {
                break;
            }
        }

        self.eat(&lexer::TokenType::RBRACKET)?;

        Ok(Some(parse_tree))
    }

    fn struct_lit(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::STRUCTLIT,
            token: self.curr_token(), 
            children: Vec::new()
        };

        self.eat(&lexer::TokenType::LCURLY)?;

        while !self.has(&lexer::TokenType::RCURLY) {
            // LINK TO < ref-or-nothing >
            if self.has(&lexer::TokenType::LINK) {
                let mut child = ParseTree {
                    parse_type: ParseType::LINKLIT,
                    token: self.curr_token(),
                    children: Vec::new(),
                };
                self.eat(&lexer::TokenType::LINK)?;
                self.eat(&lexer::TokenType::TO)?;
                
                // < ref-or-nothing >
                if self.has(&lexer::TokenType::NOTHING) {
                    self.next()?;
                    child.children.push(None);
                }
                else {
                    child.children.push(self.reference()?);
                }

                parse_tree.children.push(Some(child));
            }
            // < resolvable >
            else {
                parse_tree.children.push(self.resolvable()?);
            }

            if self.has(&lexer::TokenType::COMMA) {
                self.next()?;
            }
            else {
                break;
            }
        }

        self.eat(&lexer::TokenType::RCURLY)?;

        Ok(Some(parse_tree))
    }

    fn fun_arg_type(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // ARRAY < type or bounds >
        if self.has(&lexer::TokenType::ARRAY) {
            let mut parse_tree = ParseTree{
                parse_type: ParseType::ARRAYDEF,
                token: self.curr_token(),
                children: Vec::new(),
            };
            self.next()?;


            // FOR NOW, WE ARE GOING TO EXPECT BOUNDS
            // IF NOT PROVIDING BOUNDS, ONE WOULD NEED
            // TO DYNAMICALLY FIND AND SET THE PROPER
            // SIZE IN MEMORY
            // TO UNDO THIS, REMOVE THE FOLLOWING LINE
            self.must_be(&lexer::TokenType::LBRACKET);

            // < type or bounds >
            if self.has(&lexer::TokenType::LBRACKET) {
                parse_tree.children.push(self.bounds()?);
            }
            else {
                parse_tree.children.push(None);
            }

            self.eat(&lexer::TokenType::OF)?;

            parse_tree.children.push(self.basic_type()?);

            return Ok(Some(parse_tree));
        }

        self.basic_type()
    }

    fn var_type(&mut self) -> Result<Option<ParseTree>, &'static str> {
        // LINK TO < basic-type >
        if self.has(&lexer::TokenType::LINK) {
            let mut parse_tree = ParseTree {
                parse_type: ParseType::POINTER,
                token: self.curr_token(),
                children: Vec::new(),
            };

            self.eat(&lexer::TokenType::LINK)?;
            self.eat(&lexer::TokenType::TO)?;

            parse_tree.children.push(self.basic_type()?);

            return Ok(Some(parse_tree));
        }
        // ARRAY < type or bounds >
        else if self.has(&lexer::TokenType::ARRAY) {
            let mut parse_tree = ParseTree{
                parse_type: ParseType::ARRAYDEF,
                token: self.curr_token(),
                children: Vec::new(),
            };
            self.next()?;

            // < type or bounds >
            
            // FOR NOW, WE ARE GOING TO EXPECT BOUNDS
            // IF NOT PROVIDING BOUNDS, ONE WOULD NEED
            // TO DYNAMICALLY FIND AND SET THE PROPER
            // SIZE IN MEMORY
            // TO UNDO THIS, REMOVE THE FOLLOWING LINE
            self.must_be(&lexer::TokenType::LBRACKET);

            if self.has(&lexer::TokenType::LBRACKET) {
                parse_tree.children.push(self.bounds()?);
            }
            else {
                parse_tree.children.push(None);
            }

            self.eat(&lexer::TokenType::OF)?;

            parse_tree.children.push(self.basic_type()?);

            return Ok(Some(parse_tree));
        }

        self.basic_type()
    }

    fn basic_type(&mut self) -> Result<Option<ParseTree>, &'static str> {
        
        let mut parse_tree = ParseTree {
            parse_type: ParseType::INVALID,
            token: self.curr_token(),
            children: Vec::new(),
        };

        if self.has(&lexer::TokenType::NUMTYPE) ||
           self.has(&lexer::TokenType::TEXTTYPE) || 
           self.must_be(&ID_TYPE) {
            parse_tree = ParseTree {
                parse_type: ParseType::TYPE,
                token: self.curr_token(),
                children: Vec::new(),
            };

            self.next()?;
        }

        Ok(Some(parse_tree))
    }

    //TODO
    fn bounds(&mut self) -> Result<Option<ParseTree>, &'static str> {
        self.eat(&lexer::TokenType::LBRACKET)?;

        let mut parse_tree = ParseTree {
            parse_type: ParseType::BOUNDS,
            token: self.curr_token(), 
            children: Vec::new(),
        };

        while !self.has(&lexer::TokenType::RBRACKET) {
            parse_tree.children.push(self.bound()?);

            if self.has(&lexer::TokenType::COMMA) {
                self.next()?;
            }
            else {
                break;
            }
        }

        self.eat(&lexer::TokenType::RBRACKET)?;

        Ok(Some(parse_tree))
    }

    fn bound(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let mut parse_tree = ParseTree{
            parse_type: ParseType::BOUND,
            token: self.curr_token(), 
            children: Vec::new()
        };

        let tree = self.resolvable()?;

        if self.has(&lexer::TokenType::TO) {
            self.next()?;

            parse_tree.children.push(tree);
            parse_tree.children.push(self.resolvable()?);
        }
        else {
            parse_tree.children.push(None);
            parse_tree.children.push(tree);
        }

        Ok(Some(parse_tree))
    }

    fn foo(&mut self) -> Result<Option<ParseTree>, &'static str> {
        let parse_tree = ParseTree{
            parse_type: ParseType::QUIT,
            token: self.curr_token(), 
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
        p = Parser::new("
    program
        print(\"Hello world!\")
    end program
    ".to_string()).expect("Could not create lexer");
    }

    let tree = p.parse().expect("Error");
    match tree {
        Some(t) => t.print(),
        _ => ()
    };
}