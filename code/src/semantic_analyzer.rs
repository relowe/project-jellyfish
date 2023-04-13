#![allow(dead_code)]

use std::{env, process};
use std::collections::HashMap;
use crate::lexer::{TokenType};
use crate::parser::{ParseTree, Parser, ParseType};

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

        st.basic_types.push("number".to_string());
        st.basic_types.push("text".to_string());
        st.basic_types.push("nothing".to_string());

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

        for (key, value) in &struct_keys {
            if !self.basic_types.contains(&value.basic_type) {
                return false;
            }
        }

        self.struct_args.insert(struct_id, struct_keys);
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

pub fn unwrap_id_tree(tree: &ParseTree) -> String {
    match &tree.token.token_type {
        TokenType::ID(id) => id.to_string(),
        _ => "".to_string(),
    }
}

pub fn unwrap_type_tree(tree: &ParseTree) -> String {
    match &tree.token.token_type {
        TokenType::ID(id) => id.to_string(),
        TokenType::TEXTTYPE => "text".to_string(),
        TokenType::NUMTYPE => "number".to_string(),
        TokenType::NOTHING => "nothing".to_string(),
        _ => "invalid".to_string(),
    }
}

pub fn unwrap_lit_tree(tree: &ParseTree) -> String {
    match &tree.token.token_type {
        TokenType::NUMBER(x) => "number".to_string(),
        TokenType::TEXT(x) => "text".to_string(),
        _ => "invalid".to_string(),
    }
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    expected_return_type: Option<String>,
    expected_resolve_type: Option<SymbolType>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            expected_return_type: None,
            expected_resolve_type: None,
        }
    }

    // CODE tree
    pub fn analyze(&mut self, tree: &ParseTree) -> Result<(), &'static str> {
        // DEF (could be None)
        if tree.children[0].is_some() {
            self.analyze_definitions(tree.children[0].as_ref().unwrap())?;
        }

        // BODY (program section)
        self.analyze_body(tree.children[1].as_ref().unwrap())?;
        Ok(())
    }

    // DEFINITION tree
    fn analyze_definitions(&mut self, tree: &ParseTree) -> Result<(), &'static str> {
        // struct, global, functions

        // STRUCT DEFS
        if tree.children[0].is_some() {
            self.analyze_struct_defs(tree.children[0].as_ref().unwrap())?;
        }

        // GLOBAL DEFS
        if tree.children[1].is_some() {
            self.analyze_global_defs(tree.children[1].as_ref().unwrap())?;
        }

        // FUNCTION DEFS
        if tree.children[2].is_some() {
            self.analyze_function_defs(tree.children[2].as_ref().unwrap())?;
        }

        Ok(())
    }

    fn analyze_struct_defs(&mut self, tree: &ParseTree) -> Result<(), &'static str> {
        // Add all strucutre names to the basic types
        for struct_def_tree in &tree.children {
            let id = unwrap_id_tree(struct_def_tree.as_ref().unwrap().children[0].as_ref().unwrap());

            self.symbol_table.add_type(id);
        }

        // Add all structure objects
        for struct_def_tree in &tree.children {
            let id = unwrap_id_tree(struct_def_tree.as_ref().unwrap().children[0].as_ref().unwrap());

            let mut struct_keys: HashMap<String, SymbolType> = HashMap::new();

            for struct_arg in &struct_def_tree.as_ref().unwrap().children[1].as_ref().unwrap().children {
                let key_name = unwrap_id_tree(struct_arg.as_ref().unwrap().children[0].as_ref().unwrap());
                let key_type = self.analyze_type(struct_arg.as_ref().unwrap().children[1].as_ref().unwrap())?;

                struct_keys.insert(key_name, key_type);
            }

            self.symbol_table.add_struct_keys(id, struct_keys);
        }
        
        Ok(())
    }

    fn analyze_global_defs(&mut self, tree: &ParseTree) -> Result<(), &'static str> {
        for child in &tree.children {
            self.analyze_assignment(child.as_ref().unwrap())?;
        }
        
        Ok(())
    }

    fn analyze_function_defs(&mut self, tree: &ParseTree) -> Result<(), &'static str> {
        //tree.print();

        // Loop through each child, add its function object to symbol table
        for child in &tree.children {
            let fun_def = child.as_ref().unwrap();

            let function_id = unwrap_id_tree(fun_def.children[0].as_ref().unwrap());

            let mut params: Vec<SymbolType> = Vec::new();

            for param in &fun_def.children[1].as_ref().unwrap().children {
                //param.as_ref().unwrap().print();
                //param.as_ref().unwrap().children[1].as_ref().unwrap().print();

                params.push(self.analyze_type(param.as_ref().unwrap().children[1].as_ref().unwrap())?);
            }

            //println!{"return type tree"}
            //fun_def.children[2].as_ref().unwrap().print();

            let ret_type = unwrap_type_tree(fun_def.children[2].as_ref().unwrap());

            let fn_obj = FunctionObject {
                params: params,
                return_type: ret_type,
            };

            println!{"Adding function {} of type {:?}", function_id, fn_obj};

            self.symbol_table.add_function(function_id, fn_obj);
        }

        // Loop through each child, and check the function body
        for child in &tree.children {
            let fun_def = child.as_ref().unwrap();

            self.symbol_table.scope_in();
            for param in &fun_def.children[1].as_ref().unwrap().children {
                let param_name = unwrap_id_tree(param.as_ref().unwrap().children[0].as_ref().unwrap());
                let param_type = self.analyze_type(param.as_ref().unwrap().children[1].as_ref().unwrap())?;
                self.symbol_table.add_symbol(param_name, param_type);
            }

            self.expected_return_type = Some(unwrap_type_tree(fun_def.children[2].as_ref().unwrap()));
            self.analyze_body(fun_def.children[3].as_ref().unwrap())?;
            self.expected_return_type = None;
            self.symbol_table.scope_out();
        }
        
        Ok(())
    }

    fn analyze_body(&mut self, tree: &ParseTree) -> Result<(), &'static str> {
        Ok(())
    }

    fn analyze_assignment(&mut self, tree: &ParseTree) -> Result<(), &'static str> {
        //tree.print();
        let left_type: SymbolType;
        if tree.children[0].as_ref().unwrap().parse_type == ParseType::VARDEF {
            left_type = self.analyze_vardef(tree.children[0].as_ref().unwrap())?;
        }
        else {
            left_type = self.symbol_table.find_symbol(unwrap_id_tree(tree.children[0].as_ref().unwrap()));
        }

        self.expected_resolve_type = Some(left_type.clone());

        let right_type = self.analyze_resolvable(tree.children[1].as_ref().unwrap())?;

        self.expected_resolve_type = None;

        //println!("Comparing types \n\t{:?}\n\t{:?}", left_type, right_type);

        if left_type != right_type {
            return Err("Type mismatch");
        }

        Ok(())
    }

    fn analyze_type(&mut self, tree: &ParseTree) -> Result<SymbolType, &'static str> {
        //tree.print();
        
        // Handle TYPE tree (most basic one)
        if tree.parse_type == ParseType::TYPE {
            let basic_type = unwrap_type_tree(&tree);
            return Ok(SymbolType{
                basic_type: basic_type,
                is_array: false,
                is_pointer: false,
            });
        }

        // Handle ARRAYDEF tree
        if tree.parse_type == ParseType::ARRAYDEF {
            let basic_type = unwrap_type_tree(tree.children[1].as_ref().unwrap());
            return Ok(SymbolType{
                basic_type: basic_type,
                is_array: true,
                is_pointer: false,
            });
        }

        // Handle POINTER tree
        if tree.parse_type == ParseType::POINTER {
            if tree.children[0].as_ref().unwrap().parse_type == ParseType::ARRAYDEF {
                let basic_type = unwrap_type_tree(tree.children[0].as_ref().unwrap().children[1].as_ref().unwrap());
                return Ok(SymbolType{
                    basic_type: basic_type,
                    is_array: true,
                    is_pointer: true,
                });
            }

            let basic_type = unwrap_type_tree(tree.children[0].as_ref().unwrap());
            return Ok(SymbolType{
                basic_type: basic_type,
                is_array: false,
                is_pointer: true,
            });
        }

        Err("error")
    }

    fn analyze_resolvable(&mut self, tree: &ParseTree) -> Result<SymbolType, &'static str> {
        if tree.parse_type == ParseType::LIT {
            return Ok(SymbolType{
                basic_type: unwrap_lit_tree(&tree),
                is_pointer: false,
                is_array: false,
            });
        }
        
        Err("error")
    }

    fn analyze_vardef(&mut self, tree: &ParseTree) -> Result<SymbolType, &'static str> {
        //tree.print();
        let sym_type = self.analyze_type(tree.children[1].as_ref().unwrap())?;


        let id = unwrap_id_tree(tree.children[0].as_ref().unwrap());

        println!{"Adding symbol {} of type {:?}", id, sym_type};
        self.symbol_table.add_symbol(id, sym_type.clone());
        
        Ok(sym_type)
    }
}


fn symbol_table_test() {
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


    let mut sa = SemanticAnalyzer::new();

    sa.analyze(tree.as_ref().expect("error")).expect("Something errored");
    println!("{:?}", sa.symbol_table);
}
