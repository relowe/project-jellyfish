use std::{env};
use std::collections::HashMap;
use crate::lexer::{TokenType};
use crate::parser::{ParseTree, Parser, ParseType};
use crate::library_handler;
use indexmap::{IndexMap};

// A boolean to determine if debug information should be displayed
static DEBUG: bool = false;

// A static list off all the primitive data types
// These are added immediately into the symbol table
static PRIMATIVES: &[&str] = &["number", "text", "nothing"];

// Handle error reporting through web assembly
// For right now we just print the error, but later
//  on this would be passed to JavaScript code
macro_rules! log {
    ($($t:tt)*) => (println!("{}",  &format_args!($ ( $t ) *).to_string() ))
}

// Handle debugging through web assembly
// For right now we just print the error, but later
//  on this would be passed to JavaScript code
macro_rules! debug {
    ($($t:tt)*) => {
        if DEBUG {
            (println!("SA: {}",  &format_args!($ ( $t ) *).to_string() ))
        }
    }
}


#[derive(Clone, Eq, Debug)]
pub struct SymbolType {
    pub basic_type: String,
    pub is_pointer: bool,
    pub array_dimensions: i32,
}

// Ability to compare symbol types to ensure that
//  they match. Allows for generic type '*', and
//  ignores array dimensions of -1 (undefined size)
impl PartialEq for SymbolType {
    fn eq(&self, other: &Self) -> bool {
        let mut basic = true;
        let mut arr = true;
        if self.basic_type != "*".to_string() && other.basic_type != "*".to_string() {
            basic = self.basic_type == other.basic_type;
        }
        if self.array_dimensions >= 0 && other.array_dimensions >= 0 {
            arr = self.array_dimensions == other.array_dimensions;
        }
        
        basic && arr
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FunctionObject {
    pub params: Vec<SymbolType>,
    pub return_type: String,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SymbolTable {
    pub symbols: Vec<HashMap<String, SymbolType>>,
    pub basic_types: Vec<String>,
    pub struct_args: HashMap<String, IndexMap<String, SymbolType>>,
    pub functions: HashMap<String, FunctionObject>,
    pub depth: usize,
}

impl SymbolTable {
    // Create a new Symbol Table
    // Pull in all external functions, and load
    //  the primitive types
    pub fn new() -> Self {
        let mut st = SymbolTable {
            symbols: Vec::new(),
            basic_types: Vec::new(),
            struct_args: HashMap::new(),
            functions: library_handler::get_external_functions(),
            depth: 0,
        };

        st.symbols.push(HashMap::new());

        for primative in PRIMATIVES {
            st.basic_types.push(primative.to_string());
        }

        st
    }

    // Add a new type into the Symbol Table
    // Used when defining structures
    pub fn add_type(&mut self, basic_type: String) -> Result<(), String> {
        if self.basic_types.contains(&basic_type) {
            return Err(format!{"Type '{}' already exists", basic_type});
        }

        self.basic_types.push(basic_type);

        Ok(())
    }

    // Add a new symbol (varaiable name) and its corresponding type
    pub fn add_symbol(&mut self, symbol: String, symbol_type: SymbolType) -> Result<(), String> {
        if self.symbols[self.depth].contains_key(&symbol) {
            return Err(format!("Symbol '{}' already exists", &symbol));
        }

        if !self.basic_types.contains(&symbol_type.basic_type) {
            return Err(format!("Unknown type: {}", &symbol_type.basic_type));
        }

        self.symbols[self.depth].insert(symbol, symbol_type);

        Ok(())
    }

    // Look through all scopes of the symbol table
    //  to find the corresponding type of the symbol (id)
    // If the symbol can't be found in the table, error
    pub fn find_symbol(&mut self, symbol: String) -> Result<SymbolType, String> {
        for i in (0..self.depth+1).rev() {
            if self.symbols[i as usize].contains_key(&symbol) {
                return Ok(self.symbols[i as usize].get(&symbol).unwrap().clone());
            }
        }

        Err(format!("Unknown symbol '{}'", &symbol))
    }

    // Add a new function definition into the symbol table
    pub fn add_function(&mut self, id: String, obj: FunctionObject) -> Result<(), String> {
        // Check if the function is already defined
        if self.functions.contains_key(&id) {
            return Err(format!("Function {} has already been defined", &id));
        }

        // Check all the parameters to make sure they are actual types
        for symbol_type in &obj.params {
            if !self.basic_types.contains(&symbol_type.basic_type) {
                return Err(format!("Unknown type: {}", &symbol_type.basic_type));
            }
        }

        // Add in the function
        self.functions.insert(id, obj);

        Ok(())
    }

    // Scope in by adding a new HashMap to self.symbols
    // Also increase the depth counter
    pub fn scope_in(&mut self) {
        self.symbols.push(HashMap::new());
        self.depth += 1;
    }

    // Pop the last HashMap from self.symbols
    // Also decrease the depth counter
    pub fn scope_out(&mut self) {
        self.symbols.remove(self.depth);
        self.depth -= 1;
    }

    // Add a new structure definition into the Symbol Table
    pub fn add_struct_keys(&mut self, struct_id: String, struct_keys: IndexMap<String, SymbolType>) -> Result<(), String> {
        // Make sure a structure with the same name doesn't exist
        if self.struct_args.contains_key(&struct_id) {
            return Err(format!("Structure '{}' has already been defined", &struct_id));
        }

        // Check all structure arguments to ensure they are valid
        for (_key, value) in &struct_keys {
            if !self.basic_types.contains(&value.basic_type) {
                return Err(format!("Unknown type: {}", &value.basic_type));
            }
        }

        // Add the structure
        self.struct_args.insert(struct_id, struct_keys);
        Ok(())
    }

    // Get the type of a structure key
    pub fn get_struct_key(&self, struct_id: String, key_id: String) -> Result<SymbolType, String> {
        // Find the structure object
        if !self.struct_args.contains_key(&struct_id) {
            return Err(format!("Unknown Structure {}", &struct_id));
        }

        // Find the key
        if !self.struct_args.get(&struct_id).unwrap().contains_key(&key_id) {
            return Err(format!("Unknown key {} for structure {}", &key_id, &struct_id));
        }

        // Return the SymbolType of that key
        Ok(self.struct_args.get(&struct_id).unwrap().get(&key_id).unwrap().clone())
    }
}

// Helper method to pull the ID String from a parse tree node
pub fn unwrap_id_tree(tree: &ParseTree) -> String {
    match &tree.token.token_type {
        TokenType::ID(id) => id.to_string(),
        _ => "".to_string(),
    }
}

// Helper method to pull the type String from a parse tree node
pub fn unwrap_type_tree(tree: &ParseTree) -> String {
    match &tree.token.token_type {
        TokenType::ID(id) => id.to_string(),
        TokenType::TEXTTYPE => "text".to_string(),
        TokenType::NUMTYPE => "number".to_string(),
        TokenType::NOTHING => "nothing".to_string(),
        _ => "invalid".to_string(),
    }
}

// Helper method to get the type of a Literal in a parse tree
pub fn unwrap_lit_tree(tree: &ParseTree) -> String {
    match &tree.token.token_type {
        TokenType::NUMBER(_x) => "number".to_string(),
        TokenType::TEXT(_x) => "text".to_string(),
        _ => "invalid".to_string(),
    }
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    expected_return_type: Option<String>,
    expected_resolve_type: Option<SymbolType>,
}

impl SemanticAnalyzer {
    // Create a new Semantic Analyzer
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            expected_return_type: None,
            expected_resolve_type: None,
        }
    }

    // Helper method to get information about the current tree for errors
    fn err_header(&mut self, tree: &ParseTree) -> String {
        format!{"Error on line {}:{} - ", tree.token.row, tree.token.col}
    }

    // CODE tree
    pub fn analyze(&mut self, tree: &ParseTree) -> Result<SymbolTable, String> {
        // DEF (could be None)
        if tree.children[0].is_some() {
            self.analyze_definitions(tree.children[0].as_ref().unwrap())?;
        }
        let symtab = self.symbol_table.clone();
        // BODY (program section)
        self.analyze_body(tree.children[1].as_ref().unwrap())?;
        Ok(symtab)
    }

    // DEFINITION tree
    fn analyze_definitions(&mut self, tree: &ParseTree) -> Result<(), String> {
        // STRUCT DEFS (could be none)
        if tree.children[0].is_some() {
            self.analyze_struct_defs(tree.children[0].as_ref().unwrap())?;
        }

        // GLOBAL DEFS (could be none)
        if tree.children[1].is_some() {
            self.analyze_global_defs(tree.children[1].as_ref().unwrap())?;
        }

        // FUNCTION DEFS (could be none)
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

        // Add all structure objects one by one
        for struct_def_tree in &tree.children {
            let id = unwrap_id_tree(struct_def_tree.as_ref().unwrap().children[0].as_ref().unwrap());

            let mut struct_keys: IndexMap<String, SymbolType> = IndexMap::new();

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
        // Analyze each of the assignments one by one
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
        // Loop through each child, add its function object to symbol table
        for child in &tree.children {
            let fun_def = child.as_ref().unwrap();

            // Get the function id
            let function_id = unwrap_id_tree(fun_def.children[0].as_ref().unwrap());

            // Fill a vector with the function parameters
            let mut params: Vec<SymbolType> = Vec::new();
            for param in &fun_def.children[1].as_ref().unwrap().children {
                params.push(self.analyze_type(param.as_ref().unwrap().children[1].as_ref().unwrap())?);
            }

            // Get the return type
            let ret_type = unwrap_type_tree(fun_def.children[2].as_ref().unwrap());

            // Create the funciton object
            let fn_obj = FunctionObject {
                params: params,
                return_type: ret_type,
            };

            debug!{"Adding function {} of type {:?}", function_id, fn_obj};

            // Insert it
            self.symbol_table.add_function(function_id, fn_obj)?;
        }

        // Loop through each child, and check the function bodies
        for child in &tree.children {
            let fun_def = child.as_ref().unwrap();

            // Scope in and define the parameters
            self.symbol_table.scope_in();
            for param in &fun_def.children[1].as_ref().unwrap().children {
                let param_name = unwrap_id_tree(param.as_ref().unwrap().children[0].as_ref().unwrap());
                let param_type = self.analyze_type(param.as_ref().unwrap().children[1].as_ref().unwrap())?;
                self.symbol_table.add_symbol(param_name, param_type)?;
            }

            // Set the expected return type
            self.expected_return_type = Some(unwrap_type_tree(fun_def.children[2].as_ref().unwrap()));
            // Check the body
            self.analyze_body(fun_def.children[3].as_ref().unwrap())?;
            // Reset the expected return type
            self.expected_return_type = None;
            // Scope out
            self.symbol_table.scope_out();
        }
        
        Ok(())
    }

    fn analyze_assignment(&mut self, tree: &ParseTree) -> Result<(), String> {
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

        // Make sure the variable type (left_type) matches
        //  the literal value (right_type)
        if left_type != right_type {
            return Err(format!{"{} Type mismatch between {:?} and {:?}", self.err_header(tree), left_type, right_type});
        }

        Ok(())
    }

    fn analyze_type(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        let mut curr_tree = tree;
        let mut is_pointer = false;
        let mut array_dimensions = 0;

        // Catch pointers and unwrap
        if curr_tree.parse_type == ParseType::POINTER {
            is_pointer = true;
            curr_tree = curr_tree.children[0].as_ref().unwrap();
        }

        // Catch arrays and unwrap
        if curr_tree.parse_type == ParseType::ARRAYDEF {
            // If there are defined bounds, make sure they are integers
            // This will also set the array_dimensions
            // Otherwise, we don't know the current dimensions until later
            if curr_tree.children[0].as_ref().is_some() {
                let bounds_tree = curr_tree.children[0].as_ref().unwrap();

                for bound in &bounds_tree.children {
                    array_dimensions += 1;
                    let mut bound_type: SymbolType;
                    if bound.as_ref().unwrap().children[0].as_ref().is_some() {
                        bound_type = self.analyze_resolvable(bound.as_ref().unwrap().children[0].as_ref().unwrap())?;
                        if bound_type.basic_type != "number".to_string() {
                            return Err(format!{"{} Cannot set bounds of an array to a non-number!", self.err_header(curr_tree)});
                        }
                    }

                    bound_type = self.analyze_resolvable(bound.as_ref().unwrap().children[1].as_ref().unwrap())?;
                    if bound_type.basic_type != "number".to_string() {
                        return Err(format!{"{} Cannot set bounds of an array to a non-number!", self.err_header(curr_tree)});
                    }
                }
            }
            // This is for an array with unknown dimensions
            else {
                array_dimensions = -1;
            }

            curr_tree = curr_tree.children[1].as_ref().unwrap();
        }

        // Get the basic type
        let basic_type = unwrap_type_tree(&curr_tree);

        // Return the symbol type
        Ok(SymbolType{
            basic_type: basic_type,
            array_dimensions: array_dimensions,
            is_pointer: is_pointer,
        })
    }

    fn analyze_resolvable(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        // Catch just pure literal values
        if tree.parse_type == ParseType::LIT {
            return Ok(SymbolType{
                basic_type: unwrap_lit_tree(&tree),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        // Catch binary operations
        else if tree.parse_type == ParseType::BINOP {
            // Get the types of the left and right children
            let left_type = self.analyze_resolvable(tree.children[0].as_ref().unwrap())?;
            let right_type = self.analyze_resolvable(tree.children[1].as_ref().unwrap())?;

            // Check for addition of text (non-arrays)
            if tree.token.token_type == TokenType::ADD &&
               left_type.basic_type == "text".to_string() &&
               right_type.basic_type == "text".to_string() &&
               left_type.array_dimensions <= 0 &&
               right_type.array_dimensions <= 0 {
                return Ok(SymbolType{
                    basic_type: "text".to_string(),
                    is_pointer: false,
                    array_dimensions: 0,
                });
            }

            // Otherwise, they both must be numbers (non-arrays)
            if left_type.basic_type != "number".to_string() ||
               left_type.array_dimensions > 0 ||
               right_type.basic_type != "number".to_string() ||
               right_type.array_dimensions > 0 {
                return Err(format!{"{} Cannot perform binary operations on non-numbers", self.err_header(tree)});
            }

            // Return Ok
            return Ok(SymbolType{
                basic_type: "number".to_string(),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        // Catch negative
        else if tree.parse_type == ParseType::NEG {
            let child_type = self.analyze_resolvable(tree.children[0].as_ref().unwrap())?;

            // These must both be numbers
            if child_type.basic_type != "number".to_string() ||
               child_type.array_dimensions > 0 {
                return Err(format!{"{} Cannot perform negative operation on non-number", self.err_header(tree)});
            }

            // Return Ok
            return Ok(SymbolType{
                basic_type: "number".to_string(),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        // Catch absolute value
        else if tree.parse_type == ParseType::ABS {
            let child_type = self.analyze_resolvable(tree.children[0].as_ref().unwrap())?;

            // These must both be numbers
            if child_type.basic_type != "number".to_string() ||
               child_type.array_dimensions > 0 {
                return Err(format!{"{} Cannot perform absolute value operation on non-number", self.err_header(tree)});
            }

            // Return Ok
            return Ok(SymbolType{
                basic_type: "number".to_string(),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        // Catch bitwise not
        else if tree.parse_type == ParseType::BITNOT {
            let child_type = self.analyze_resolvable(tree.children[0].as_ref().unwrap())?;

            // These must both be numbers
            if child_type.basic_type != "number".to_string() ||
               child_type.array_dimensions != 0 {
                return Err(format!{"{} Cannot perform bitwise not operation on non-number", self.err_header(tree)});
            }

            // Return Ok
            return Ok(SymbolType{
                basic_type: "number".to_string(),
                is_pointer: false,
                array_dimensions: 0,
            });
        }

        // Catch references
        else if tree.parse_type == ParseType::GETINDEX ||
                tree.parse_type == ParseType::GETSTRUCT ||
                tree.parse_type == ParseType::ID {
            return self.analyze_reference(tree);
        }
        
        // Catch function calls
        else if tree.parse_type == ParseType::CALL {
            let fun_name = unwrap_id_tree(tree.children[0].as_ref().unwrap());

            debug!("Checking function {}", fun_name);


            if self.symbol_table.functions.contains_key(&fun_name) {
                let fun_obj = self.symbol_table.functions.get(&fun_name).expect("Could not load function map").clone();
                
                let ret = SymbolType {
                    basic_type: fun_obj.return_type.clone(),
                    is_pointer: false,
                    array_dimensions: 0,
                };

                let ex_res_type = self.expected_resolve_type.clone();

                // Functions with no arguments
                if fun_obj.params.len() == 0 {
                    if tree.children[1].as_ref().is_some() {
                        return Err(format!{"{} Function {} does not expect arguments", self.err_header(tree), fun_name})
                    }
                }
                // Functions with arguments
                else {
                    if tree.children[0].as_ref().is_none() {
                        return Err(format!{"{} Function {} expected arguments", self.err_header(tree), fun_name})
                    }

                    for (fun_arg, child) in fun_obj.params.into_iter().zip(tree.children[1].as_ref().unwrap().children.iter()) {
                        self.expected_resolve_type = Some(fun_arg.clone());
                        let res_type = self.analyze_resolvable(child.as_ref().unwrap())?;
        
                        if res_type != fun_arg.clone() {
                            return Err(format!{"{} Function argument {:?} does not match expected parameter type {:?}", self.err_header(child.as_ref().unwrap()), fun_arg.clone(), res_type});
                        }
                    }
                }

                // Set the expected resolve type
                self.expected_resolve_type = ex_res_type;

                return Ok(ret)

            }

            return Ok(SymbolType {
                basic_type: "external_function_call".to_string(),
                array_dimensions: 0, 
                is_pointer: false
            });
        }
        
        // Catch arrays (recursively unwrap them)
        else if tree.parse_type == ParseType::ARRAYLIT {
            debug!{"Expected resolve type: {:?}", self.expected_resolve_type};

            // Catch unexpected arrays
            if self.expected_resolve_type.is_none() {
                return Err(format!{"{} Unexpected array literal", self.err_header(tree)});
            }

            // Mark the current expected resolve type (this should be Some)
            let mut ex_res_type = self.expected_resolve_type.clone().unwrap();

            // Catch all arrays first
            // Compare size if we actually know it
            if ex_res_type.array_dimensions > 0 {

                // Take out a layer of array, then check all children
                let new_ex_res_type = SymbolType {
                    basic_type: ex_res_type.basic_type.clone(),
                    is_pointer: ex_res_type.is_pointer,
                    array_dimensions: ex_res_type.array_dimensions - 1,
                };

                // Check each child
                self.expected_resolve_type = Some(new_ex_res_type.clone());
                for child in &tree.children {
                    debug!("Checking array children for type {:?}", self.expected_resolve_type);
                    let element_type = self.analyze_resolvable(child.as_ref().unwrap())?;
                    if element_type != new_ex_res_type {
                        return Err(format!{"{} Type mis-match inside of array literal", self.err_header(child.as_ref().unwrap())});
                    }
                }

                debug!("Processed array of type {:?}", ex_res_type);

                self.expected_resolve_type = Some(ex_res_type.clone());

                return Ok(ex_res_type);
            }
            // Otherwise, attempt to predict size from the literal
            else {
                let mut arr_depth = 0;
                let mut t = tree;
                while t.parse_type == ParseType::ARRAYLIT {
                    t = t.children[0].as_ref().unwrap();
                    arr_depth += 1;
                }

                ex_res_type.array_dimensions = arr_depth;
                self.expected_resolve_type = Some(ex_res_type);
                // With a predicted size, now try to resolve
                return self.analyze_resolvable(tree);
            }
        }

        // Catch structures (and match children types)
        else if tree.parse_type == ParseType::STRUCTLIT {
            debug!{"IN STRUCT > Expected resolve type: {:?}", self.expected_resolve_type};

            // Catch unexpected structures
            if self.expected_resolve_type.is_none() {
                return Err(format!{"{} Unexpected structure literal", self.err_header(tree)});
            }

            // Mark the current expected resolve type (this should be Some)
            let ex_res_type = self.expected_resolve_type.clone().unwrap();

            // Get the structure expected arguments
            let struct_args = self.symbol_table.struct_args.get(&ex_res_type.basic_type).expect("Could not load structure arguments").clone();

            debug!("Structure args: {:?}", struct_args);

            if struct_args.keys().len() != tree.children.len() {
                return Err(format!{"{} Number of items does not match number of structure elements", self.err_header(tree)});
            }

            for (struct_arg, child) in struct_args.values().zip(tree.children.iter()) {
                self.expected_resolve_type = Some(struct_arg.clone());
                let res_type = self.analyze_resolvable(child.as_ref().unwrap())?;

                if res_type != struct_arg.clone() {
                    return Err(format!{"{} Structure item {:?} does not match expected type {:?}", self.err_header(child.as_ref().unwrap()), struct_arg.clone(), res_type});
                }
            }

            self.expected_resolve_type = Some(ex_res_type.clone());
            return Ok(ex_res_type);
        }

        // Catch link literals
        else if tree.parse_type == ParseType::LINKLIT {
            if self.expected_resolve_type.is_none() {
                return Err(format!{"{} Unexpected link", self.err_header(tree)});
            }

            if tree.children[0].is_some() {
                let res_type = self.analyze_reference(tree.children[0].as_ref().unwrap())?;
                if res_type != self.expected_resolve_type.clone().unwrap() {
                    return Err(format!{"{} Link to {:?} does not match expected link type {:?}", self.err_header(tree.children[0].as_ref().unwrap()), self.expected_resolve_type, res_type});
                }
            }
            return Ok(self.expected_resolve_type.clone().unwrap());
        }
        
        //Err("error".to_string())
        debug!("Parse type: {:?}", &tree.parse_type);
        Ok(SymbolType{
            basic_type: "invalid".to_string(),
            array_dimensions: 1,
            is_pointer: false,
        })
    }

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
                    left_type.array_dimensions <= 0 &&
                    right_type.array_dimensions <= 0 {
                    return Ok(());
                }

                // Otherwise, they both must be numbers (non-arrays)
                if left_type.basic_type != "number".to_string() ||
                    left_type.array_dimensions > 0 ||
                    right_type.basic_type != "number".to_string() ||
                    right_type.array_dimensions > 0 {
                        debug!{"{:?} <<<>>> {:?}", left_type, right_type};
                        return Err(format!{"{} Cannot perform binary comparison on non-numbers", self.err_header(tree)});
                    }
                return Ok(());
            },
            ParseType::ISLINKED | ParseType::ISNOTLINKED => {
                if !self.analyze_reference(tree.children[0].as_ref().unwrap())?.is_pointer {
                    return Err(format!{"{} Cannot check status of a non-linable object", self.err_header(tree)});
                }
                return Ok(());
            },
            _ => return Err(format!{"{} Must have a conditional", self.err_header(tree)}),
        }
    }

    fn analyze_reference(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        let ref_type: SymbolType;

        if tree.parse_type != ParseType::ID {
            ref_type = self.analyze_reference(tree.children[0].as_ref().unwrap())?;

            if tree.parse_type == ParseType::GETINDEX {
                let mut arr_dims = ref_type.array_dimensions;

                // check all indecies to make sure they are numbers
                let index_tree = tree.children[1].as_ref().unwrap();
                for idx in &index_tree.children {
                    // We can't do much about unknown array dimensions yet
                    if arr_dims != -1 {
                        arr_dims -= 1;
                    }
                    let idx_type = self.analyze_resolvable(idx.as_ref().unwrap())?;
                    if idx_type.basic_type != "number".to_string() {
                        return Err(format!{"{} Cannot index using a non-number", self.err_header(tree)});
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

        debug!("FINDING SYMBOL {}", unwrap_id_tree(&tree));
        self.symbol_table.find_symbol(unwrap_id_tree(&tree))
    }

    fn analyze_vardef(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        //tree.print();
        let sym_type = self.analyze_type(tree.children[1].as_ref().unwrap())?;

        // ID
        if tree.children[0].as_ref().unwrap().parse_type == ParseType::ID {
            let id = unwrap_id_tree(tree.children[0].as_ref().unwrap());

            debug!{"Adding symbol {} of type {:?}", id, sym_type};
            self.symbol_table.add_symbol(id, sym_type.clone())?;
        }
        // IDS
        else {
            for id_tree in &tree.children[0].as_ref().unwrap().children {
                let id = unwrap_id_tree(id_tree.as_ref().unwrap());

                debug!{"Adding symbol {} of type  {:?}", id, sym_type};
                self.symbol_table.add_symbol(id, sym_type.clone())?;
            }
        }
        
        Ok(sym_type)
    }

    fn analyze_body(&mut self, tree: &ParseTree) -> Result<(), String> {
        let mut is_other: bool = false;
        for child in &tree.children {
            match child.as_ref().unwrap().parse_type {
                ParseType::IF => self.analyze_if(child.as_ref().unwrap())?,
                ParseType::LINK => (),
                ParseType::UNLINK => self.analyze_unlink(child.as_ref().unwrap())?,
                ParseType::WHILE => self.analyze_while(child.as_ref().unwrap())?,
                ParseType::REPEAT => self.analyze_repeat(child.as_ref().unwrap())?,
                ParseType::REPEATFOR => self.analyze_repeat_for(child.as_ref().unwrap())?,
                ParseType::REPEATFOREVER => self.analyze_repeat_forever(child.as_ref().unwrap())?,
                ParseType::QUIT | ParseType::BREAK | ParseType::CONTINUE => (),
                ParseType::ASSIGN => self.analyze_assignment(child.as_ref().unwrap())?,
                ParseType::RETURN => self.analyze_return(child.as_ref().unwrap())?,
                _ => is_other = true,
            }
            // Catch anything with an actual return type
            if is_other {
                if child.as_ref().unwrap().parse_type == ParseType::VARDEF {
                    self.analyze_vardef(child.as_ref().unwrap())?;
                }
                else {
                    self.analyze_resolvable(child.as_ref().unwrap())?;
                }
                is_other = false
            }
        }

        Ok(())
    }

    fn analyze_if(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Analyze the comparison
        self.analyze_conditional(tree.children[0].as_ref().unwrap())?;

        // Analyze the body
        self.symbol_table.scope_in();
        self.analyze_body(tree.children[1].as_ref().unwrap())?;
        self.symbol_table.scope_out();

        if tree.children[2].is_some() {
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
                return Err(format!("{} Received illegal if-child {:?}",
                    self.err_header(tree.children[2].as_ref().unwrap()), 
                    tree.children[2].as_ref().unwrap().parse_type));
            }
        }

        Ok(())
    }

    fn analyze_while(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Analyze the comparison
        self.analyze_conditional(tree.children[0].as_ref().unwrap())?;

        // Analyze the while block
        self.symbol_table.scope_in();
        self.analyze_body(tree.children[1].as_ref().unwrap())?;
        self.symbol_table.scope_out();

        Ok(())
    }

    fn analyze_repeat(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Check if a number was passed to repeat that amount of times
        let repeat_type = self.analyze_resolvable(tree.children[0].as_ref().unwrap())?;
        if repeat_type.basic_type != "number".to_string() || repeat_type.array_dimensions != 0 {
            return Err(format!{"{} Repeat must be provided a number literal", self.err_header(tree.children[0].as_ref().unwrap())});
        }

        // Analyze the body
        self.symbol_table.scope_in();
        self.analyze_body(tree.children[1].as_ref().unwrap())?;
        self.symbol_table.scope_out();

        Ok(())
    }

    fn analyze_repeat_for(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Make sure the second item is an array
        let repeat_type = self.analyze_resolvable(tree.children[1].as_ref().unwrap())?;
        if repeat_type.array_dimensions == 0 {
            return Err(format!{"{} Repeat must have an array to loop over", self.err_header(tree.children[1].as_ref().unwrap())});
        }

        // Analyze body:
        // Scope in
        self.symbol_table.scope_in();

        // Add name and type
        let mut symbol_type = repeat_type.clone();
        // We can't subtract from an unknown-size array
        if symbol_type.array_dimensions > 0 {
            symbol_type.array_dimensions -= 1;
        }
        let symbol = unwrap_id_tree(tree.children[0].as_ref().unwrap());
        self.symbol_table.add_symbol(symbol, symbol_type)?;

        // Check Body
        self.analyze_body(tree.children[2].as_ref().unwrap())?;

        // Scope out
        self.symbol_table.scope_out();

        Ok(())
    }

    fn analyze_repeat_forever(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Analyze the body
        self.symbol_table.scope_in();
        self.analyze_body(tree.children[0].as_ref().unwrap())?;
        self.symbol_table.scope_out();

        Ok(())
    }

    fn analyze_return(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Make sure we are expecting a return
        if self.expected_return_type.is_none() {
            return Err(format!{"{} Unexpected return statement", self.err_header(tree)});
        }

        let expected_type = self.expected_return_type.clone().unwrap();
        let mut ret_type: String = "nothing".to_string();

        // Check to see if the return type is nothing
        if tree.children[0].is_some() {
            let sym_type = self.analyze_resolvable(tree.children[0].as_ref().unwrap())?;
            ret_type = sym_type.basic_type;
        }

        // Make sure the expected type matches the return type
        if expected_type != ret_type {
            return Err(format!{"{} Mismatched return types. Expected {:?}, got {:?}", self.err_header(tree), expected_type, ret_type}.to_string());
        }

        // Otherwise return Ok
        Ok(())
    }

    /// 
    fn analyze_unlink(&mut self, tree: &ParseTree) -> Result<(), String> {
        if !self.analyze_reference(tree.children[0].as_ref().unwrap())?.is_pointer {
            return Err(format!("{} {:?} is not a linkable object", self.err_header(tree.children[0].as_ref().unwrap()), unwrap_id_tree(tree.children[0].as_ref().unwrap())));
        }
        Ok(())
    }
}

// Generic main function that just runs the SA and prints
//  the resulting symbol table. If a filename is provided
//  in the system arguments, analyze that file instead.
pub fn _test() {
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

    let tree = match p.parse() {
        Ok(t) => t.unwrap(),
        Err(s) => {
            log!{"Parser Error: {}", s};
            return;
        },
    };

    if DEBUG {
        tree.print();
        println!("\n\n\n\n");
    }

    let mut sa = SemanticAnalyzer::new();

    sa.analyze(&tree).expect("Something errored");
    println!("{:?}", sa.symbol_table);
}
