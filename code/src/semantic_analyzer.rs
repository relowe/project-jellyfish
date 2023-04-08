#![allow(dead_code)]

use std::{env, process};
use std::collections::HashMap;
use crate::parser;

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

impl SemanticAnalyzer {
    
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