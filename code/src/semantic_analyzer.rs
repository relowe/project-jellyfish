#![allow(dead_code)]

use std::{env, process};
use std::collections::HashMap;
use crate::parser;
use crate::lexer::{Token, TokenType};
use crate::parser::{ParseTree, ParseType};

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolData {
    TEXT(String),
    NUMBER(f64),
    FUNCTION(Function),
    STRUCT(SymbolTable),
    ARRAY(),
    POINTER,
    NOTHING,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub parameters: Vec<String>,
    pub code: parser::ParseTree,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    name: String,
    data: SymbolData,
    is_pointer: bool,
    is_array: bool,
}

impl Symbol {
    pub fn new(name: String, data: SymbolData, is_pointer: bool, is_array: bool) -> Result<Self, &'static str> {
        Ok(Symbol {
            name: name,
            data: data,
            is_pointer: is_pointer,
            is_array: is_array,
        })
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct SymbolTable {
    table: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Result<Self, &'static str> {
        Ok(SymbolTable {
            table: HashMap::new(),
        })
    }

    pub fn add(&mut self, symbol: Symbol) {
        self.table.insert(symbol.name.clone(), symbol);
    }


    // Roughly how to add a funciton to the Symbol table,
    // please implement in the analyzer code.
    // Keep in mind it was written as a function of this impl,
    // not from the outside
    /*pub fn add_function(&mut self, name: String, parameters: Vec<String>, code: ParseTree) {
        let function = Function {
            parameters: parameters,
            code: code,
        };
        let symbol = Symbol {
            var_type: VarType::FUNCTION(function),
        };
        self.table.insert(name, symbol);
    }*/
    
    pub fn get(&self, name: String) -> Result<Option<&Symbol>, &'static str> {
        Ok(self.table.get(&name))
    }
}


// Define a struct called SemanticAnalyzer that has a HashMap called symbol_table, which maps strings to Symbols
pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, Symbol>,
}
// Implement a constructor function for the struct that returns a new instance of SemanticAnalyzer
impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: HashMap::new(),
        }
    }
    
    // Implement a method called analyze that takes a ParseTree as input and returns a Result type with no value if successful or a String if an error occurs
    pub fn analyze(&mut self, parse_tree: ParseTree) -> Result<(), String> {
        match parse_tree.parse_type {
            ParseType::CODE => {
                for statement in parse_tree.children {
                    if statement.is_some() {
                        self.analyze_statement(statement.unwrap())?;
                    }
                }
            }
            _ => return Err("Invalid parse tree".to_string()),
        }
        Ok(())
    }

    // Define a method called analyze_statement that takes a ParseTree as input and returns a Result type with no value if successful or a String if an error occurs
    fn analyze_statement(&mut self, statement: ParseTree) -> Result<(), String> {
        match statement {
            ParseTree::parse_type::Assignment(identifier, expression) => {
                let symbol = self.get_symbol(&identifier)?;
                let expression_type = self.analyze_expression(expression)?;
                if symbol.data != expression_type {
                    return Err(format!(
                        "Type mismatch: expected {:?}, found {:?}",
                        symbol.data, expression_type
                    ));
                }
            }
            _ => return Err("Invalid statement".to_string()),
        }
        Ok(())
    }

    // Define a method called analyze_expression that takes a ParseTree as input and returns a Result type with a SymbolData if successful or a String if an error occurs
    // TODO add support for more types, figure out issue with abiguous associated types?
    fn analyze_expression(&mut self, expression: ParseTree) -> Result<SymbolData, String> {
        match expression {
            ParseTree::parse_type::BINOP(op, left, right) => {
                let left_type = self.analyze_expression(*left)?;
                let right_type = self.analyze_expression(*right)?;
                if left_type != right_type {
                    return Err(format!(
                        "Type mismatch: expected {:?}, found {:?}",
                        left_type, right_type
                    ));
                }
                match op {
                    TokenType::ADD => Ok(left_type),
                    _ => Err("Invalid operator".to_string()),
                }
            }
            ParseTree::parse_type::NUMBER_TYPE(value) => Ok(SymbolData::NUMBER(value)),
            ParseTree::parse_type::ID(identifier) => {
                let symbol = self.get_symbol(&identifier)?;
                Ok(symbol.data.clone())
            }
            _ => Err("Invalid expression".to_string()),
        }
    }
    

    fn get_symbol(&self, identifier: &str) -> Result<&Symbol, String> {
        match self.symbol_table.get(identifier) {
            Some(symbol) => Ok(symbol),
            None => Err(format!("Undefined symbol: {}", identifier)),
        }
    }
}


pub fn main() {
    let mut table: SymbolTable;

    table = SymbolTable::new().expect("Error in getting symbol table");

    table.add(Symbol {
        name: String::from("x"),
        data: SymbolData::NUMBER(2.0),
        is_pointer: false,
        is_array: false,
    });
    println!("{:?}", table)
}