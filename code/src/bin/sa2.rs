#![allow(dead_code)]

use std::{env, process};
use std::collections::HashMap;
use crate::parser;

use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SymbolType {
    basic_type: String,
    is_pointer: bool,
    is_array: bool,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FunctionObject {
    params: Vec<SymbolType>,
    return_type: String,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SymbolTable {
    symbols: Vec<HashMap<String, SymbolType>>,
    basic_types: Vec<String>,
    struct_args: HashMap<String, HashMap<String, SymbolType>>,
    functions: HashMap<String, FunctionObject>,
    depth: usize,
}

impl SymbolTable {
    fn new() -> Self {
        let mut st = SymbolTable {
            symbols: Vec::new(),
            basic_types: Vec::new(),
            struct_args: HashMap::new(),
            functions: HashMap::new(),
            depth: 0,
        };

        st.symbols.push(HashMap::new());

        st.basic_types.push("NUMBER".to_string());
        st.basic_types.push("TEXT".to_string());
        st.basic_types.push("NOTHING".to_string());

        st
    }

    fn add_type(&mut self, basic_type: String) -> bool {
        if self.basic_types.contains(&basic_type) {
            // Should error on false
            return false;
        }

        self.basic_types.push(basic_type);

        true
    }

    fn add_symbol(&mut self, symbol: String, symbol_type: SymbolType) -> bool {
        if self.symbols[self.depth].contains_key(&symbol) {
            return false;
        }

        if !self.basic_types.contains(&symbol_type.basic_type) {
            return false;
        }

        self.symbols[self.depth].insert(symbol, symbol_type);

        true
    }

    /*
     * check depth currently at, then keep back tracking to find the symbol
     * symbols: Vec<HashMap<String, SymbolType>>
     */
    fn find_symbol(&mut self, symbol: String) -> SymbolType {
        for i in (0..self.depth+1).rev() {
            if self.symbols[i as usize].contains_key(&symbol) {
                return self.symbols[i as usize].get(&symbol).unwrap().clone();
            }
        }

        // do error stuff

        SymbolType {
            basic_type: "invalid".to_string(),
            is_pointer: false,
            is_array: false,
        }
    }

    /*
     * checks if the function is already defined
     * checks if all the parameters are valid (types that do exist)
     */
    fn add_function(&mut self, id: String, obj: FunctionObject) -> bool {
        if self.functions.contains_key(&id) {
            return false;
        }

        for symbol_type in &obj.params {
            if !self.basic_types.contains(&symbol_type.basic_type) {
                return false
            }
        }

        self.functions.insert(id, obj);

        true
    }

    /*
     * depth + 1
     * add new hashmap to vector
     */
    fn scope_in(&mut self) {
        self.symbols.push(HashMap::new());
        self.depth += 1;
    }

    /*
     * depth - 1
     * remove hashmap from vector
     */
    fn scope_out(&mut self) {
        self.symbols.remove(self.depth);
        self.depth -= 1;
    }

    /*
     * make sure structrue doesn't already exist
     * make sure all struct keys are valid (check symbol table)
     * struct_args: HashMap<String, HashMap<String, SymbolType>>
     */
    fn add_struct_keys(&mut self, struct_id: String, struct_keys: HashMap<String, SymbolType>) -> bool {
        if self.struct_args.contains_key(&struct_id) {
            return false;
        }

        for (key, value) in struct_keys {
            if !self.basic_types.contains(&value.basic_type) {
                return false;
            }
        }

        true
    }

    /*
     * find the structure and key, if they don't exist, error
     * struct_args: HashMap<String, HashMap<String, SymbolType>>
     */
    fn get_struct_key(&self, struct_id: String, key_id: String) -> SymbolType {
        // check for structure
        if !self.struct_args.contains_key(&struct_id) {
            // error lol
        }

        // check for key
        if !self.struct_args.get(&struct_id).unwrap().contains_key(&key_id) {
            // error lol
        }

        // return the SymbolType
        self.struct_args.get(&struct_id).unwrap().get(&key_id).unwrap().clone()
    }
}

fn main() {
    let mut st = SymbolTable::new();

    st.add_type("PERSON".to_string());

    let sym_type = SymbolType {
        basic_type: "PERSON".to_string(),
        is_pointer: false,
        is_array: false,
    };

    st.add_symbol("x".to_string(), sym_type);

    println! {"{:?}", st};
}
