#![allow(dead_code)]
#![allow(warnings)]

use std::{env, process};
use std::collections::HashMap;
use crate::lexer::{TokenType};
use crate::parser::{ParseTree, Parser, ParseType};

#[derive(Clone, Eq, Debug)]
pub struct SymbolType {
    basic_type: String,
    is_pointer: bool,
    array_dimensions: i32,
}

impl PartialEq for SymbolType {
    fn eq(&self, other: &Self) -> bool {
        self.basic_type == other.basic_type &&
        self.array_dimensions == other.array_dimensions
    }
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

    fn add_type(&mut self, basic_type: String) -> Result<(), String> {
        if self.basic_types.contains(&basic_type) {
            return Err("Type mismatch".to_string());
        }

        self.basic_types.push(basic_type);

        Ok(())
    }

    fn add_symbol(&mut self, symbol: String, symbol_type: SymbolType) -> Result<(), String> {
        if self.symbols[self.depth].contains_key(&symbol) {
            return Err(format!("Symbol '{}' already exists", &symbol));
        }

        if !self.basic_types.contains(&symbol_type.basic_type) {
            return Err(format!("Unknown type: {}", &symbol_type.basic_type));
        }

        self.symbols[self.depth].insert(symbol, symbol_type);

        Ok(())
    }

    /*
     * check depth currently at, then keep back tracking to find the symbol
     * symbols: Vec<HashMap<String, SymbolType>>
     */
    fn find_symbol(&mut self, symbol: String) -> Result<SymbolType, String> {
        for i in (0..self.depth+1).rev() {
            if self.symbols[i as usize].contains_key(&symbol) {
                return Ok(self.symbols[i as usize].get(&symbol).unwrap().clone());
            }
        }

        Err(format!("Unknown symbol '{}'", &symbol))
    }

    /*
     * checks if the function is already defined
     * checks if all the parameters are valid (types that do exist)
     */
    fn add_function(&mut self, id: String, obj: FunctionObject) -> Result<(), String> {
        if self.functions.contains_key(&id) {
            return Err(format!("Function {} has already been defined", &id));
        }

        for symbol_type in &obj.params {
            if !self.basic_types.contains(&symbol_type.basic_type) {
                return Err(format!("Unknown type: {}", &symbol_type.basic_type));
            }
        }

        self.functions.insert(id, obj);

        Ok(())
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
    fn add_struct_keys(&mut self, struct_id: String, struct_keys: HashMap<String, SymbolType>) -> Result<(), String> {
        if self.struct_args.contains_key(&struct_id) {
            return Err(format!("Structure {} has already been defined", &struct_id));
        }

        for (key, value) in &struct_keys {
            if !self.basic_types.contains(&value.basic_type) {
                return Err(format!("Unknown type: {}", &value.basic_type));
            }
        }

        self.struct_args.insert(struct_id, struct_keys);
        Ok(())
    }

    /*
     * find the structure and key, if they don't exist, error
     * struct_args: HashMap<String, HashMap<String, SymbolType>>
     */
    fn get_struct_key(&self, struct_id: String, key_id: String) -> Result<SymbolType, String> {
        // check for structure
        if !self.struct_args.contains_key(&struct_id) {
            return Err(format!("Unknown Structure {}", &struct_id));
        }

        // check for key
        if !self.struct_args.get(&struct_id).unwrap().contains_key(&key_id) {
            return Err(format!("Unknown key {}", &key_id));
        }

        // return the SymbolType
        Ok(self.struct_args.get(&struct_id).unwrap().get(&key_id).unwrap().clone())
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
    pub fn analyze(&mut self, tree: &ParseTree) -> Result<(), String> {
        // DEF (could be None)
        if tree.children[0].is_some() {
            self.analyze_definitions(tree.children[0].as_ref().unwrap())?;
        }

        // BODY (program section)
        self.analyze_body(tree.children[1].as_ref().unwrap())?;
        Ok(())
    }

    // DEFINITION tree
    fn analyze_definitions(&mut self, tree: &ParseTree) -> Result<(), String> {
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

    fn analyze_struct_defs(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Add all strucutre names to the basic types
        for struct_def_tree in &tree.children {
            let id = unwrap_id_tree(struct_def_tree.as_ref().unwrap().children[0].as_ref().unwrap());

            self.symbol_table.add_type(id)?;
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

            self.symbol_table.add_struct_keys(id, struct_keys)?;
        }
        
        Ok(())
    }

    fn analyze_global_defs(&mut self, tree: &ParseTree) -> Result<(), String> {
        for child in &tree.children {
            if child.as_ref().unwrap().parse_type == ParseType::ASSIGN {
                self.analyze_assignment(child.as_ref().unwrap())?;
            }
            else {
                self.analyze_vardef(child.as_ref().unwrap())?;
            }
        }
        
        Ok(())
    }

    fn analyze_function_defs(&mut self, tree: &ParseTree) -> Result<(), String> {
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

            self.symbol_table.add_function(function_id, fn_obj)?;
        }

        // Loop through each child, and check the function body
        for child in &tree.children {
            let fun_def = child.as_ref().unwrap();

            self.symbol_table.scope_in();
            for param in &fun_def.children[1].as_ref().unwrap().children {
                let param_name = unwrap_id_tree(param.as_ref().unwrap().children[0].as_ref().unwrap());
                let param_type = self.analyze_type(param.as_ref().unwrap().children[1].as_ref().unwrap())?;
                self.symbol_table.add_symbol(param_name, param_type)?;
            }

            self.expected_return_type = Some(unwrap_type_tree(fun_def.children[2].as_ref().unwrap()));
            self.analyze_body(fun_def.children[3].as_ref().unwrap())?;
            self.expected_return_type = None;
            self.symbol_table.scope_out();
        }
        
        Ok(())
    }

    fn analyze_assignment(&mut self, tree: &ParseTree) -> Result<(), String> {
        //tree.print();
        let left_type: SymbolType;

        // VARDEF
        if tree.children[0].as_ref().unwrap().parse_type == ParseType::VARDEF {
            left_type = self.analyze_vardef(tree.children[0].as_ref().unwrap())?;
        }
        else {
            left_type = self.analyze_reference(tree.children[0].as_ref().unwrap())?;
        }

        self.expected_resolve_type = Some(left_type.clone());
        let right_type = self.analyze_resolvable(tree.children[1].as_ref().unwrap())?;
        self.expected_resolve_type = None;

        //println!("Comparing types \n\t{:?}\n\t{:?}", left_type, right_type);

        if left_type != right_type {
            return Err(format!{"Type mismatch between {:?} and {:?}", left_type, right_type});
        }

        Ok(())
    }

    fn analyze_type(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        //tree.print();

        let mut curr_tree = tree;
        let mut is_pointer = false;
        let mut array_dimensions = 0;

        if curr_tree.parse_type == ParseType::POINTER {
            is_pointer = true;
            curr_tree = curr_tree.children[0].as_ref().unwrap();
        }

        if curr_tree.parse_type == ParseType::ARRAYDEF {
            curr_tree.print();
            let bounds_tree = curr_tree.children[0].as_ref().unwrap();

            for bound in &bounds_tree.children {
                array_dimensions += 1;
                let mut bound_type: SymbolType;
                if bound.as_ref().unwrap().children[0].as_ref().is_some() {
                    bound_type = self.analyze_resolvable(bound.as_ref().unwrap().children[0].as_ref().unwrap())?;
                    if bound_type.basic_type != "number".to_string() {
                        return Err("Cannot set bounds of an array to a non-number!".to_string());
                    }
                }

                bound_type = self.analyze_resolvable(bound.as_ref().unwrap().children[1].as_ref().unwrap())?;
                if bound_type.basic_type != "number".to_string() {
                    return Err("Cannot set bounds of an array to a non-number!".to_string());
                }
            }

            curr_tree = curr_tree.children[1].as_ref().unwrap();
        }

        let basic_type = unwrap_type_tree(&curr_tree);
        Ok(SymbolType{
            basic_type: basic_type,
            array_dimensions: array_dimensions,
            is_pointer: is_pointer,
        })
    }

    fn analyze_resolvable(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        if tree.parse_type == ParseType::LIT {
            return Ok(SymbolType{
                basic_type: unwrap_lit_tree(&tree),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        else if tree.parse_type == ParseType::BINOP {
            // Get the types of the left and right children
            let left_type = self.analyze_resolvable(tree.children[0].as_ref().unwrap())?;
            let right_type = self.analyze_resolvable(tree.children[1].as_ref().unwrap())?;

            // Check for addition of text
            if tree.token.token_type == TokenType::ADD &&
               left_type.basic_type == "text".to_string() &&
               right_type.basic_type == "text".to_string() &&
               left_type.array_dimensions == 0 &&
               right_type.array_dimensions == 0 {
                return Ok(SymbolType{
                    basic_type: "text".to_string(),
                    is_pointer: false,
                    array_dimensions: 0,
                });
            }

            // Otherwise, they both must be numbers (non-arrays)
            if left_type.basic_type != "number".to_string() ||
               left_type.array_dimensions != 0 ||
               right_type.basic_type != "number".to_string() ||
               right_type.array_dimensions != 0 {
                return Err("Cannot perform binary operations on non-numbers".to_string());
            }

            // Return Ok
            return Ok(SymbolType{
                basic_type: "number".to_string(),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        // copy above for neg, abs, bitnot (with only one child)


        // Blake, is it not simpler to just check if they are literals?
        // Then they could't be arrays, right? If I'm wrong, just change it
        else if tree.parse_type == ParseType::NEG {
            if tree.children[0].as_ref().unwrap().parse_type != ParseType::LIT ||
                    unwrap_lit_tree(tree.children[0].as_ref().unwrap()) != "number" {
                return Err("Negative must be provided a number literal.".to_string());
            }
            return Ok(SymbolType {
                basic_type: unwrap_lit_tree(tree.children[0].as_ref().unwrap()),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        else if tree.parse_type == ParseType::ABS {
            if tree.children[0].as_ref().unwrap().parse_type != ParseType::LIT ||
                    unwrap_lit_tree(tree.children[0].as_ref().unwrap()) != "number" {
                return Err("Absolute must be provided a number literal.".to_string());
            }
            return Ok(SymbolType {
                basic_type: unwrap_lit_tree(tree.children[0].as_ref().unwrap()),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        // Blake: Is this only supposed to work on numbers? That's how I coded it.
        else if tree.parse_type == ParseType::BITNOT {
            if tree.children[0].as_ref().unwrap().parse_type != ParseType::LIT ||
                    unwrap_lit_tree(tree.children[0].as_ref().unwrap()) != "number" {
                return Err("Absolute must be provided a number literal.".to_string());
            }
            return Ok(SymbolType {
                basic_type: unwrap_lit_tree(tree.children[0].as_ref().unwrap()),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        else if tree.parse_type == ParseType::GETINDEX ||
                tree.parse_type == ParseType::GETSTRUCT ||
                tree.parse_type == ParseType::ID {
            return self.analyze_reference(tree);
        }
        
        // Blake: To Do
        else if tree.parse_type == ParseType::CALL {
            return Ok(SymbolType {
                basic_type: "call".to_string(),
                array_dimensions: 0, 
                is_pointer: false
            });
        }
        
        // Blake: To Do
        else if tree.parse_type == ParseType::ARRAYORSTRUCTLIT {
            return Ok(SymbolType {
                basic_type: "array/struct".to_string(),
                array_dimensions: 1, 
                is_pointer: false
            });
        }
        
        //Err("error".to_string())
        println!("Parse type: {:?}", &tree.parse_type);
        Ok(SymbolType{
            basic_type: "invalid".to_string(),
            array_dimensions: 1,
            is_pointer: false,
        })
    }

    /// Blake: check/reimplement please
    fn analyze_conditional(&mut self, tree: &ParseTree) -> Result<(), String> {
        match tree.parse_type {
            ParseType::BINCOMP => {
                // Get the types of the left and right children
                let left_type = self.analyze_resolvable(tree.children[0].as_ref().unwrap())?;
                let right_type = self.analyze_resolvable(tree.children[1].as_ref().unwrap())?;

                // Check for comparison of text
                if tree.token.token_type == TokenType::EQ &&
                    left_type.basic_type == "text".to_string() &&
                    right_type.basic_type == "text".to_string() &&
                    left_type.array_dimensions == 0 &&
                    right_type.array_dimensions == 0 {
                    return Ok(());
                }

                println!("Comparing types {:?} and {:?}.", left_type, right_type);

                // Otherwise, they both must be numbers (non-arrays)
                if left_type.basic_type != "number".to_string() ||
                    left_type.array_dimensions != 0 ||
                    right_type.basic_type != "number".to_string() ||
                    right_type.array_dimensions != 0 {
                    return Err("Cannot perform binary comparisons on non-numbers".to_string());
                }
                return Ok(());
            },
            ParseType::ISLINKED | ParseType::ISNOTLINKED => {
                if !self.analyze_reference(tree.children[0].as_ref().unwrap())?.is_pointer {
                    return Err("Cannot check the link status of a non-link object".to_string());
                }
                return Ok(());
            },
            _ => return Err("Must have a conditional statement".to_string()),
        }
    }

    fn analyze_reference(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        let mut ref_type: SymbolType;

        if tree.parse_type != ParseType::ID {
            ref_type = self.analyze_reference(tree.children[0].as_ref().unwrap())?;

            if tree.parse_type == ParseType::GETINDEX {
                let mut arr_dims = ref_type.array_dimensions;

                // check all indecies to make sure they are numbers
                let index_tree = tree.children[1].as_ref().unwrap();
                for idx in &index_tree.children {
                    arr_dims -= 1;
                    let idx_type = self.analyze_resolvable(idx.as_ref().unwrap())?;
                    if idx_type.basic_type != "number".to_string() {
                        return Err("Cannot index using a non-number".to_string());
                    }
                }

                return Ok(SymbolType{
                    basic_type: ref_type.basic_type,
                    is_pointer: ref_type.is_pointer,
                    array_dimensions: arr_dims,
                });
            }

            else if tree.parse_type == ParseType::GETSTRUCT {
                let struct_key = unwrap_id_tree(tree.children[1].as_ref().unwrap());

                return self.symbol_table.get_struct_key(ref_type.basic_type, struct_key);
            }
        }

        println!("FINDING SYMBOL {}", unwrap_id_tree(&tree));
        self.symbol_table.find_symbol(unwrap_id_tree(&tree))
    }

    fn analyze_vardef(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        //tree.print();
        let sym_type = self.analyze_type(tree.children[1].as_ref().unwrap())?;

        if tree.children[0].as_ref().unwrap().parse_type == ParseType::ID {
            let id = unwrap_id_tree(tree.children[0].as_ref().unwrap());

            println!{"Adding symbol {} of type {:?}", id, sym_type};
            self.symbol_table.add_symbol(id, sym_type.clone())?;
        }
        else {
            for id_tree in &tree.children[0].as_ref().unwrap().children {
                let id = unwrap_id_tree(id_tree.as_ref().unwrap());

                println!{"Adding symbol {} of type  {:?}", id, sym_type};
                self.symbol_table.add_symbol(id, sym_type.clone())?;
            }
        }
        
        Ok(sym_type)
    }

    /// To Do
    /// 
    fn analyze_body(&mut self, tree: &ParseTree) -> Result<(), String> {
        let mut ret: bool;
        let mut res: bool;
        for child in &tree.children {
            if self.expected_return_type.is_some() &&
                    child.as_ref().unwrap().parse_type == ParseType::RETURN {
                self.analyze_return(child.as_ref().unwrap())?;
                continue;
            }

            ret = false;
            res = false;
            match child.as_ref().unwrap().parse_type {
                ParseType::IF => self.analyze_if(child.as_ref().unwrap())?,
                // ParseType::LINK => self.analyze_link(child.as_ref().unwrap())?,
                ParseType::UNLINK => self.analyze_unlink(child.as_ref().unwrap())?,
                ParseType::WHILE => self.analyze_while(child.as_ref().unwrap())?,
                ParseType::REPEAT => self.analyze_repeat(child.as_ref().unwrap())?,
                ParseType::REPEATFOR => self.analyze_repeat_for(child.as_ref().unwrap())?,
                ParseType::REPEATFOREVER => self.analyze_repeat_forever(child.as_ref().unwrap())?,
                // ParseType::QUIT => self.analyze_quit(child.as_ref().unwrap())?,
                // ParseType::BREAK => self.analyze_break(child.as_ref().unwrap())?,
                // ParseType::CONTINUE => self.analyze_continue(child.as_ref().unwrap())?,
                ParseType::QUIT | ParseType::BREAK | ParseType::CONTINUE => (),
                // ParseType::CALL => self.analyze_call(child.as_ref().unwrap())?,
                ParseType::ASSIGN => self.analyze_assignment(child.as_ref().unwrap())?,
                ParseType::RETURN => ret = true,
                _ => res = true,
            }
            if res {
                self.analyze_resolvable(child.as_ref().unwrap())?;
            }
            if ret {
                return Err("Cannot return a value from the main program.".to_string());
            }
        }

        Ok(())
    }

    ///            <>!=      if   elif    else
    /// Children: BINCOMP, BLOCK, (IF || BLOCK)
    fn analyze_if(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Analyze the comparison
        if tree.children[0].as_ref().unwrap().parse_type != ParseType::BINCOMP {
            return Err("If statements must have a condition.".to_string());
        }
        self.analyze_conditional(tree.children[0].as_ref().unwrap())?;

        // Analyze the if block
        if tree.children[1].as_ref().unwrap().parse_type != ParseType::BLOCK {
            return Err("If statements must have a block to execute.".to_string());
        }
        self.symbol_table.scope_in();
        self.analyze_body(tree.children[1].as_ref().unwrap())?;
        self.symbol_table.scope_out();

        if tree.children.len() == 3 {
            // Analyze an else if
            if tree.children[2].as_ref().unwrap().parse_type == ParseType::IF {
                self.analyze_if(tree.children[2].as_ref().unwrap())?;
            }

            // Analyze an else block if it exists
            else if tree.children[2].as_ref().unwrap().parse_type == ParseType::BLOCK {
                self.symbol_table.scope_in();
                self.analyze_body(tree.children[2].as_ref().unwrap())?;
                self.symbol_table.scope_out();
            }

            else {
                return Err(format!("Received illegal if-child {:?}", 
                    tree.children[2].as_ref().unwrap().parse_type));
            }
        }

        Ok(())
    }

    /// 
    fn analyze_while(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Analyze the comparison
        if tree.children[0].as_ref().unwrap().parse_type != ParseType::BINCOMP {
            return Err("While statements must have a condition.".to_string());
        }
        self.analyze_conditional(tree.children[0].as_ref().unwrap())?;

        // Analyze the while block
        if tree.children[1].as_ref().unwrap().parse_type != ParseType::BLOCK {
            return Err("While statements must have a block to execute.".to_string());
        }
        self.symbol_table.scope_in();
        self.analyze_body(tree.children[1].as_ref().unwrap())?;
        self.symbol_table.scope_out();

        Ok(())
    }

    /// 
    fn analyze_repeat(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Check if a number literal was passed to repeat that amount of times
        if tree.children[0].as_ref().unwrap().parse_type != ParseType::LIT ||
                unwrap_lit_tree(tree.children[0].as_ref().unwrap()) != "number" {
            return Err("repeat must be provided a number literal.".to_string());
        }

        // Analyze the body
        self.symbol_table.scope_in();
        self.analyze_body(tree.children[1].as_ref().unwrap())?;
        self.symbol_table.scope_out();

        Ok(())
    }

    /// 
    fn analyze_repeat_for(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Make sure the second item is an array
        let arr_type: SymbolType = self.analyze_reference(tree.children[1].as_ref().unwrap())?;
        if arr_type.array_dimensions < 1 {
            return Err("You must provide an array to iterate over.".to_string());
        }

        // Analyze body:
        //   - Scope in
        self.symbol_table.scope_in();

        //   - Add name and type
        let mut symbol_type = arr_type.clone();
        symbol_type.array_dimensions -= 1;
        let symbol = unwrap_id_tree(tree.children[0].as_ref().unwrap());
        self.symbol_table.add_symbol(symbol, symbol_type);

        //   - Check Body
        self.analyze_body(tree.children[2].as_ref().unwrap())?;

        //   - Scope out
        self.symbol_table.scope_out();

        Ok(())
    }

    /// 
    fn analyze_repeat_forever(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Analyze the body
        self.symbol_table.scope_in();
        self.analyze_body(tree.children[0].as_ref().unwrap())?;
        self.symbol_table.scope_out();

        Ok(())
    }

    /// 
    fn analyze_return(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        // Check to see if the return type is nothing
        if unwrap_type_tree(tree.children[0].as_ref().unwrap()) == "nothing" {
            return Ok(SymbolType {
                basic_type: "nothing".to_string(),
                is_pointer: false,
                array_dimensions: 0,
            });
        }
        // Otherwise return the resolvable type
        Ok(self.analyze_resolvable(tree.children[0].as_ref().unwrap())?)
    }

    /// 
    fn analyze_unlink(&mut self, tree: &ParseTree) -> Result<(), String> {
        if !self.analyze_reference(tree.children[0].as_ref().unwrap())?.is_pointer {
            return Err(format!("{:?} is not linked", unwrap_id_tree(tree.children[0].as_ref().unwrap())));
        }
        Ok(())
    }
}


fn symbol_table_test() {
    let mut st = SymbolTable::new();

    st.add_type("PERSON".to_string());

    let sym_type = SymbolType {
        basic_type: "PERSON".to_string(),
        is_pointer: false,
        array_dimensions: 0,
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
