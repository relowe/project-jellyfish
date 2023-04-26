#![allow(dead_code)]

use std::{env, process};
use crate::lexer::{TokenType};
use crate::parser::{Parser, ParseTree, ParseType};
use crate::semantic_analyzer::{SemanticAnalyzer, SymbolTable};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;

// ======================
// =  MEMORY MANAGEMENT =
// ======================

// different pointers types that will point at the memory array
#[derive(Debug, PartialEq, Clone)]
pub enum PointerType {
    LINK(Box<PointerType>), // a link pointer that just marks the linked address
    PRIMITIVE,
    ARRAY(Vec<(i32,i32)>, Box<PointerType>), // bounds and array type
    STRUCTURE(String), // structure ID
}

// the pointer structure itself
#[derive(Clone, PartialEq, Debug)]
pub struct Pointer {
    address: usize,
    size: usize,
    pointer_type: PointerType,
}

impl Pointer {
    fn null() -> Self {
        Pointer{
            address: 0,
            size: 0,
            pointer_type: PointerType::PRIMITIVE,
        }
    }
}

// a memory address and size, used by the heap
#[derive(Clone, Debug, PartialEq, Eq)]
struct MemorySpace {
    address: usize,
    size: usize,
}

impl PartialOrd for MemorySpace {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.size != other.size {
            return Some(self.size.cmp(&other.size));
        }
        Some(other.address.cmp(&self.address))
    }
}

impl Ord for MemorySpace {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.size != other.size {
            return self.size.cmp(&other.size);
        }
        other.address.cmp(&self.address)
    }
}

#[derive(Debug)]
pub struct Environment {
    // namespace stores id's (as strings), and the address that those id's point to in memory
    namespace: Vec<HashMap<String, Pointer>>,
    // memory is a vector that stores all the data in the program (like the memory heap in other languages)
    memory: Vec<PrimitiveType>,
    // heap stores available memory addresses and their sizes
    heap: BinaryHeap<MemorySpace>,
    // linked_values stores how many times a variable has been linked
    // this is used when deallocing to ensure we don't delete expected values
    linked_values: HashMap<usize, i32>,
}


impl Environment {
    // Create a new instance of the memory
    pub fn new() -> Self {
        // Create the environment
        let mut env = Environment {
            namespace: Vec::new(),
            memory: Vec::new(),
            heap: BinaryHeap::new(),
            linked_values: HashMap::new(),
        };

        // Fill in the first namespace with a hashmap
        env.namespace.push(HashMap::new());

        // Allocate an empty space at address 0 to catch unset references
        env.alloc(1);
        env.memory[0] = PrimitiveType::INVALID;

        // Return the environment
        env
    }

    // Find a memory address of the requested size
    // If no memory exists of that size, extend the
    //  memory space to create size for it
    fn alloc(&mut self, size: usize) -> usize {
        let mem_peek = self.heap.peek();

        if mem_peek.is_none() || mem_peek.unwrap().size < size {
            let addr:usize = self.memory.len();

            for i in 0..size {
                self.memory.push(PrimitiveType::INITIALIZED);
            }

            return addr;
        }

        let mem_space = self.heap.pop().unwrap();

        let addr: usize = mem_space.address;
        let remaining_size: usize = mem_space.size - size;

        if remaining_size > 0 {
            self.heap.push(MemorySpace{
                address: addr + size,
                size: remaining_size,
            });
        }

        addr
    }

    // Recursively remove all the memory from a provided pointer
    fn dealloc(&mut self, pointer: Pointer) {
        match pointer.pointer_type {
            PointerType::ARRAY(_,_) | PointerType::STRUCTURE(_) => {
                let mut ptr = Pointer{
                    pointer_type: PointerType::PRIMITIVE,
                    address: pointer.address,
                    size: 1,
                };

                for i in 0..pointer.size {
                    ptr.address = pointer.address + i;
                    self.dealloc(ptr.clone());
                }
            },
            _ => {
                let val = self.get_value(pointer.clone());
                match val {
                    Ok(PrimitiveType::POINTER(p)) => {
                        match p.pointer_type {
                            PointerType::LINK(p2) => (), // TODO
                            _ => self.dealloc(*p),
                        };
                        self.memory[pointer.address] = PrimitiveType::NOTHING
                    },
                    _ => self.memory[pointer.address] = PrimitiveType::NOTHING,
                };}  
        }
    }

    // Access the data in a given memory address (this returns a clone)
    fn get_value(&self, pointer: Pointer) -> Result<PrimitiveType, String> {
        println!{"Getting memory address {}", pointer.address}
        if pointer.address >= self.memory.len() {
            return Err("Accessing a memory address out of bounds".to_string());
        }

        Ok(self.memory[pointer.address].clone())
    }

    // Set the value at the specificed pointer address
    // Assumes the pointer is to a valid address, and that the type matches
    fn set_value(&mut self, pointer: Pointer, value: PrimitiveType) {
        println!{"Setting address {} to {:?}", pointer.address, value};
        self.memory[pointer.address] = value.clone();
    }

    // Access the pointer value of a given literal ID
    // If the provided ID does not exist, try to recurse
    //  if provided ID cannot be found, error
    fn get_id(&self, id: String) -> Result<Pointer, String> {
        println!{"Looking for id {}", id};

        if self.namespace.len() > 1 {
            for i in (0..self.namespace.len()).rev() {
                if self.namespace[i].contains_key(&id) {
                    return Ok(self.namespace[i].get(&id).unwrap().clone());
                }
            }
        }
        else {
            if self.namespace[self.namespace.len()-1].contains_key(&id) {
                return Ok(self.namespace[self.namespace.len()-1].get(&id).unwrap().clone());
            }
        }

        Err(format!{"Could not find id '{}' in the namespace", id})
    }

    // Set the pointer for a given ID
    // If the ID does not exist, error
    fn set_id(&mut self, id: String, pointer: Pointer) -> Result<(), String> {
        println!{"Setting id {} to {:?}", id, pointer};
        for i in (0..self.namespace.len()).rev() {
            if self.namespace[i].contains_key(&id) {
                self.namespace[i].insert(id, pointer.clone());
                return Ok(());
            }
        }

        Err(format!{"Could not find id '{}' in the namespace", id})
    }

    // Insert a new ID with a pointer value
    // If the ID exists (in the current namespace)
    fn insert_id(&mut self, id: String, pointer: Pointer) -> Result<(), String> {
        println!{"Adding new id {} with value {:?}", id, pointer};
        let len = self.namespace.len()-1;

        if self.namespace[len].contains_key(&id) {
            return Err(format!{"Cannot have duplicate variables {}", id});
        }

        self.namespace[len].insert(id, pointer);
        Ok(())
    }



    // Scope in by making a new namespace
    fn scope_in(&mut self) {
        self.namespace.push(HashMap::new());
    }

    // Scope out, deleting the last namespace and all addresses
    //  associated with that namespace
    fn scope_out(&mut self) {
        let names = self.namespace.pop().expect("Can not scope out further");

        // This is a very inneficient O(n^2)
        //  check for existing pointers, but
        //  our language is simple enough
        //  that it shouldn't be a huge
        //  time loss
        for ptr in names.values() {
            let mut seen = false;
            for i in 0..self.namespace.len() {
                if self.namespace[i].values().any(|val| val == ptr) {
                    seen = true;
                    break;
                }
            }
            if !seen {
                self.dealloc(ptr.clone());
            }
        }

        self.build_heap();
    }

    fn build_heap(&mut self) {
        // clear and rebuild the available heap
        // this is a lazy version of handling merging heap items
        //  together, by just emptying and reubilding the enitre heap
        // should be called after scoping out

        let mut addr: usize = 0;
        self.heap.clear();

        while self.memory[self.memory.len()-1] == PrimitiveType::NOTHING {
            self.memory.pop();
        }

        while addr < self.memory.len() {
            if self.memory[addr] != PrimitiveType::NOTHING {
                addr += 1;
                continue;
            } 

            let start: usize = addr;
            while addr < self.memory.len() && self.memory[addr] == PrimitiveType::NOTHING {
                addr += 1;
            }

            let size: usize = addr - start;

            self.heap.push(MemorySpace{
                address: start,
                size: size,
            });

            addr += 1;
        }
    }
 
}

#[derive(Debug, PartialEq, Clone)]
pub struct LiteralType {
    lit_type: String,
    is_primitive: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LiteralValue {
    lit_type: String,
    is_primitive: bool,
    values: Option<Vec<LiteralValue>>,
    value: Option<PrimitiveType>,
}

impl LiteralValue {
    fn null() -> Self {
        LiteralValue {
            lit_type: "nothing".to_string(),
            is_primitive: true,
            values: None,
            value: Some(PrimitiveType::NOTHING),
        }
    }

    fn from_number(n: f64) -> Self {
        LiteralValue {
            lit_type: "number".to_string(),
            is_primitive: true,
            value: Some(PrimitiveType::NUMBER(n)),
            values: None,
        }
    }

    fn from_text(t: String) -> Self {
        LiteralValue {
            lit_type: "text".to_string(),
            is_primitive: true,
            value: Some(PrimitiveType::TEXT(t)),
            values: None,
        }
    }

    fn extract_number(&self) -> Option<f64> {
        if self.lit_type != "number".to_string() || !self.is_primitive {
            return None;
        }
        
        match self.value.as_ref().unwrap() {
            PrimitiveType::NUMBER(n) => Some(*n),
            _ => None,
        }
    }

    fn extract_text(&self) -> Option<String> {
        if self.lit_type != "text".to_string() || !self.is_primitive {
            return None;
        }
        
        match self.value.as_ref().unwrap() {
            PrimitiveType::TEXT(t) => Some(t.to_string()),
            _ => None,
        }
    }
}

// =========================
// = END MEMORY MANAGEMENT =
// =========================

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
        TokenType::NUMBER(_x) => "number".to_string(),
        TokenType::TEXT(_x) => "text".to_string(),
        _ => "invalid".to_string(),
    }
}


#[derive(Clone, PartialEq, Debug)]
pub enum PrimitiveType {
    NUMBER(f64),
    TEXT(String),
    NOTHING,
    INITIALIZED, // memory created for a variable that isn't in use yet
    POINTER(Box<Pointer>),
    INVALID, // memory that should not be access (address 0)
}

impl From <PrimitiveType> for bool {
    fn from(t: PrimitiveType) -> bool {
        match t {
            PrimitiveType::TEXT(t) => t.len() > 0,
            PrimitiveType::NUMBER(n) => n != 0.0,
            _ => false,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum LoopStatus {
    DEFAULT,
    BREAK,
    CONTINUE,
    RETURN,
}

#[derive(Debug, Clone)]
struct InterpreterFunctionObj {
    name: String,
    param_names: Vec<String>,
    param_pointers: Vec<Pointer>,
    body: ParseTree
}

pub struct Interpreter {
    symbol_table: SymbolTable,
    return_value: LiteralValue,
    loop_status: LoopStatus,
    in_function_call: i32,
    env: Environment,
    structure_defs: HashMap<String, Vec<Pointer>>,
    function_defs: HashMap<String, InterpreterFunctionObj>,
}

impl Interpreter {
    pub fn new(tree: &ParseTree) -> Self {
        let symtab = SemanticAnalyzer::new().analyze(tree).expect("Semantic Analyzer errored");
        Interpreter {
            symbol_table: symtab,
            return_value: LiteralValue::null(),
            loop_status: LoopStatus::DEFAULT,
            in_function_call: 0,
            env: Environment::new(),
            structure_defs: HashMap::new(),
            function_defs: HashMap::new(),
        }
    }

    // Get the amount of memory needed for a strucutre,
    //  this will just be the number of key names in the structure
    pub fn get_struct_size(&self, id: String) -> Result<usize, String> {
        if self.symbol_table.struct_args.contains_key(&id) {
            return Ok(self.symbol_table.struct_args[&id].keys().len());
        }

        Err(format!{"Cannot find strucutre {}", id})
    }

    // A helper method to move a literal value
    //  into memory, this will allocate any unallocated
    //  memory for substructures (like arrays of structures, or structs inside structs)
    // This does not do type checking, but does do size/structure checking
    fn set_literal_in_memory(&mut self, pointer: Pointer, lit: LiteralValue) -> Result<(), String> {
        println!("++++++++++++++++++++++++++++++++++++");
        println!{"Setting pointer {:?}", pointer};
        println!{"with value {:?}", lit};
        println!{"++++++++++++++++++++++++++++++++++++"};
        
        match pointer.pointer_type {
            // Just set the value for a primitive
            PointerType::PRIMITIVE => {
                if !lit.is_primitive || (lit.lit_type != "number".to_string() && lit.lit_type != "text".to_string()) {
                    return Err("Cannot set a primitive type (text/number) equal to a non-primitive type".to_string());
                }
                self.env.set_value(pointer, lit.value.unwrap().clone());
            }
            // For an array, set the value for all its children
            PointerType::ARRAY(bounds, arr_pointer_type) => {
                if lit.is_primitive {
                    return Err(format!{"Expected array, got a primitive (text/number): {:?}", lit});
                }

                if (bounds[0].0 - bounds[0].1).abs() + 1 != lit.values.as_ref().unwrap().len() as i32 {
                    return Err(format!{"Expected array of size {}, got array of size {}", (bounds[0].0 - bounds[0].1).abs() + 1, lit.values.as_ref().unwrap().len()});
                }

                // If there are more bounds (multi-dimensional array) make a
                //  new pointer with the new bounds, and find how far to
                //  offset for each element
                // (We are serializing the array in memory)
                let mut pointer_type = *arr_pointer_type.clone();
                let mut offset_scale = 1;
                if bounds.len() > 1 {
                    let mut new_bounds: Vec<(i32, i32)> = Vec::new();
                    for i in 1..bounds.len() {
                        new_bounds.push(bounds[i].clone());
                        offset_scale = offset_scale * ((bounds[i].0 - bounds[i].1).abs() + 1);
                    }
                    // re-wrap back into an array
                    pointer_type = PointerType::ARRAY(new_bounds, Box::new(pointer_type));
                }

                println!("Offset scale = {}", offset_scale);

                // Create a running pointer for the array
                let mut ptr = Pointer {
                    pointer_type: pointer_type.clone(),
                    size: offset_scale as usize,
                    address: pointer.address.clone(),
                };

                // Recurse to put all values in array
                for i in 0..((bounds[0].0 - bounds[0].1).abs() + 1) {
                    let val = &lit.values.as_ref().unwrap()[i as usize];
                    ptr.address = pointer.address + (i*offset_scale) as usize;

                    // If this is a primitive now, we can just set it by recursing
                    if val.is_primitive || val.lit_type == "array".to_string() {
                        self.set_literal_in_memory(ptr.clone(), val.clone())?;
                    }
                    // Otherwise, we need to go to the current pointer
                    //  (or make a new pointer if it doesn't exist)
                    else {
                        let at_addr = self.env.get_value(ptr.clone())?;
                        println!{"Currently at address {:?}", at_addr};
                        if at_addr == PrimitiveType::INITIALIZED || at_addr == PrimitiveType::NOTHING {
                            // create a new pointer for space
                            let s = match &pointer_type {
                                PointerType::STRUCTURE(s) => s.to_string(),
                                _ => "invlalid".to_string(),
                            };

                            let struct_size = self.get_struct_size(s.clone())?;
                            let address = self.env.alloc(struct_size);
                            let struct_ptr = Pointer{
                                pointer_type: PointerType::STRUCTURE(s),
                                size: struct_size,
                                address: address,
                            };

                            // Set the value and recurse to fill it
                            self.env.set_value(ptr.clone(), PrimitiveType::POINTER(Box::new(struct_ptr.clone())));
                            self.set_literal_in_memory(struct_ptr.clone(), val.clone());
                        }
                        else {
                            // This means we have a strucure already there, so we
                            //  can pull the current pointer and recurse
                            match at_addr {
                                PrimitiveType::POINTER(p) => self.set_literal_in_memory(*p.clone(), val.clone()),
                                _ => return Err("Cannot reference structure pointer in memory".to_string()),
                            };
                            
                        }
                    } 
                }
            }
            PointerType::STRUCTURE(name) => {
                // We know the number of arguments should match at this point,
                //  but it doesn't hurt to check anyway
                let struct_size = self.get_struct_size(name.clone())?;
                if struct_size != lit.values.as_ref().unwrap().len() {
                    return Err(format!{"Mismatched number of arguments for Structure '{}'", name});
                }

                // Create a running pointer to set values
                let mut ptr = Pointer {
                    pointer_type: PointerType::PRIMITIVE,
                    size: 1,
                    address: pointer.address.clone(),
                };


                // Loop through each child setting its value appropriately
                let mut struct_ptrs = self.structure_defs.get(&name).expect("Could not load strucutre arguments").clone();
                for (struct_ptr, val) in struct_ptrs.iter().zip(lit.values.unwrap().iter()) {
                    // If this is a primitive value, get the details and set it
                    // Also increment the memory pointer
                    if struct_ptr.pointer_type == PointerType::PRIMITIVE {
                        ptr.pointer_type = struct_ptr.pointer_type.clone();
                        ptr.size = struct_ptr.size;
                        self.set_literal_in_memory(ptr.clone(), val.clone());
                    }
                    else {
                        // Check if a pointer already exists
                        ptr.pointer_type = struct_ptr.pointer_type.clone();
                        ptr.size = struct_ptr.size;
                        let at_addr = self.env.get_value(ptr.clone())?;

                        println!{"Need to set structure key {:?} to value {:?}", struct_ptr, val};
                        println!{"We are seeing value {:?}", at_addr};

                        if at_addr == PrimitiveType::INITIALIZED || at_addr == PrimitiveType::NOTHING {
                            let mut tmp_ptr = struct_ptr.clone();
                            tmp_ptr.address = self.env.alloc(tmp_ptr.size);
                            // Add this pointer to the memory block
                            self.env.set_value(ptr.clone(), PrimitiveType::POINTER(Box::new(tmp_ptr.clone())));
                            // Recurse to set the actual values
                            self.set_literal_in_memory(tmp_ptr.clone(), val.clone())?;
                        }
                        else {
                            match at_addr {
                                PrimitiveType::POINTER(p) => self.set_literal_in_memory(*p.clone(), val.clone()),
                                _ => return Err("Cannot reference structure pointer in memory".to_string()),
                            };
                        }
                    }

                    // When looping, increment the address
                    ptr.address += 1;
                }
            }
            PointerType::LINK(linked_ptr) => {
                println!("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");
                println!{"{:?}", linked_ptr};
            }
        }

        println!{"+++++++++++++++++ ENVIRONMENT MEMORY +++++++++++++++++++\n{:?}", self.env.memory};

        Ok(())
    }

    // Using the provided pointer, clone and wrap up the provided memory into
    //  a literal value. 
    // TODO
    fn get_literal_in_memory(&mut self, pointer: Pointer) -> Result<LiteralValue, String> {
        println!("++++++++++++++++++++++++++++++++++++");
        println!{"Getting pointer {:?}", pointer};
        println!{"++++++++++++++++++++++++++++++++++++"};

        // Check for redirects first, and handle those
        let at_addr = self.env.get_value(pointer.clone())?;
        match at_addr {
            PrimitiveType::POINTER(p) => {
                if p.pointer_type == pointer.pointer_type {
                    return self.get_literal_in_memory(*p);
                }
            },
            _ => (),
        }

        match pointer.pointer_type.clone() {
            PointerType::PRIMITIVE => {
                let val = self.env.get_value(pointer.clone())?;

                return match val {
                    PrimitiveType::NUMBER(n) => Ok(LiteralValue::from_number(n)),
                    PrimitiveType::TEXT(t) => Ok(LiteralValue::from_text(t)),
                    _ => Err(format!{"Attempting to access invalid memory at address {:?}", pointer.address}),
                };
            },

            PointerType::ARRAY(bounds, arr_pointer_type) => {
                // If there are more bounds (multi-dimensional array) make a
                //  new pointer with the new bounds, and find how far to
                //  offset for each element
                // (We need this info to undo the serialization)
                let mut pointer_type = *arr_pointer_type.clone();
                let mut offset_scale = 1;
                let mut num_elements = (bounds[0].0 - bounds[0].1).abs() + 1;
                if bounds.len() > 1 {
                    let mut new_bounds: Vec<(i32, i32)> = Vec::new();
                    for i in 1..bounds.len() {
                        new_bounds.push(bounds[i].clone());
                        offset_scale = offset_scale * ((bounds[i].0 - bounds[i].1).abs() + 1);
                    }
                    // re-wrap back into an array
                    pointer_type = PointerType::ARRAY(new_bounds, Box::new(pointer_type));
                }

                println!("Offset scale = {}", offset_scale);

                // Create a running pointer for the array
                let mut ptr = Pointer {
                    pointer_type: pointer_type.clone(),
                    size: offset_scale as usize,
                    address: pointer.address.clone(),
                };

                // Create the literal type and recursively add all children
                let mut lit_vec: Vec<LiteralValue> = Vec::new();
                
                for i in 0..num_elements {
                    lit_vec.push(self.get_literal_in_memory(ptr.clone())?);
                    ptr.address += offset_scale as usize;
                }

                return Ok(LiteralValue{
                    lit_type: "array".to_string(),
                    is_primitive: false,
                    values: Some(lit_vec),
                    value: None,
                });
            }

            PointerType::LINK(p) => {
                let mut ptr = pointer.clone();
                ptr.pointer_type = *p;
                return self.get_literal_in_memory(ptr);
            }

            PointerType::STRUCTURE(name) => {
                // Create a vector to store the structure literals
                let mut lit_vec: Vec<LiteralValue> = Vec::new();

                // Create a running pointer for the structure
                let mut ptr = Pointer {
                    pointer_type: PointerType::PRIMITIVE,
                    size: 1,
                    address: pointer.address.clone(),
                };

                let mut struct_ptrs = self.structure_defs.get(&name).expect("Could not load strucutre arguments").clone();
                for struct_ptr in struct_ptrs.iter() {
                    // If this is a primitive value, just pull its value
                    if struct_ptr.pointer_type == PointerType::PRIMITIVE {
                        ptr.pointer_type = struct_ptr.pointer_type.clone();
                        ptr.size = struct_ptr.size;
                        lit_vec.push(self.get_literal_in_memory(ptr.clone())?);
                    }
                    else {
                        // Make sure the pointer exists
                        ptr.pointer_type = struct_ptr.pointer_type.clone();
                        ptr.size = struct_ptr.size;
                        let at_addr = self.env.get_value(ptr.clone())?;

                        println!{"We are seeing structure value {:?}", at_addr};

                        if at_addr == PrimitiveType::INITIALIZED || at_addr == PrimitiveType::NOTHING {
                            return Err("Cannot get the value of an uninitialized memory address".to_string());
                        }
                        else {
                            lit_vec.push(match at_addr {
                                PrimitiveType::POINTER(p) => self.get_literal_in_memory(*p.clone())?,
                                _ => return Err("Cannot reference structure pointer in memory".to_string()),
                            });
                        }
                    }

                    // Increment the pointer
                    ptr.address += 1;

                }

                return Ok(LiteralValue {
                    lit_type: name.to_string(),
                    is_primitive: false,
                    values: Some(lit_vec),
                    value: None,
                });
            }
        }

        Ok(LiteralValue::null())
    }

    pub fn eval(&mut self, tree: &ParseTree) -> Result<(), String> {
        // DEF (could be None)
        if tree.children[0].is_some() {
            self.eval_definitions(tree.children[0].as_ref().unwrap())?;
        }

        // BODY (program section)
        self.eval_body(tree.children[1].as_ref().unwrap())?;
        Ok(())
    }

    fn eval_definitions(&mut self, tree: &ParseTree) -> Result<(), String> {
        // STRUCT DEFS
        if tree.children[0].is_some() {
            self.eval_struct_defs(tree.children[0].as_ref().unwrap())?;
        }

        // GLOBAL DEFS
        if tree.children[1].is_some() {
            self.eval_global_defs(tree.children[1].as_ref().unwrap())?;
        }

        // FUNCTION DEFS
        if tree.children[2].is_some() {
            self.eval_function_defs(tree.children[2].as_ref().unwrap())?;
        }

        Ok(())
    }

    fn eval_struct_defs(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Create a pointer structure for each argument
        //  and store them in a vector for later
        // Add all structure objects
        for struct_def_tree in &tree.children {
            let id = unwrap_id_tree(struct_def_tree.as_ref().unwrap().children[0].as_ref().unwrap());

            let mut struct_keys: Vec<Pointer> = Vec::new();

            for struct_arg in &struct_def_tree.as_ref().unwrap().children[1].as_ref().unwrap().children {
                // We don't actually need the id, just the expected pointers in order
                let expected_type = self.eval_type(struct_arg.as_ref().unwrap().children[1].as_ref().unwrap())?;
                println!("expected_type = {:?}", expected_type);
                struct_keys.push(expected_type);
            }
            println!("STRUCTURE POINTERS FOR {}", id);
            println!{"{:?}", struct_keys};
            self.structure_defs.insert(id, struct_keys);
        }
        
        Ok(())
    }

    fn eval_global_defs(&mut self, tree: &ParseTree) -> Result<(), String> {
        for child in &tree.children {
            if child.as_ref().unwrap().parse_type == ParseType::ASSIGN {
                self.eval_assignment(child.as_ref().unwrap())?;
            }
            else {
                self.eval_vardef(child.as_ref().unwrap())?;
            }
        }
        
        Ok(())
    }

    fn eval_function_defs(&mut self, tree: &ParseTree) -> Result<(), String> {

        // Loop through each child, get its name, params, and arguments
        for child in &tree.children {
            let fun_def = child.as_ref().unwrap();

            let function_id = unwrap_id_tree(fun_def.children[0].as_ref().unwrap());

            let mut param_names: Vec<String> = Vec::new();
            let mut param_pointers: Vec<Pointer> = Vec::new();

            for param in &fun_def.children[1].as_ref().unwrap().children {
                param_pointers.push(self.eval_type(param.as_ref().unwrap().children[1].as_ref().unwrap())?);
                param_names.push(unwrap_id_tree(param.as_ref().unwrap().children[0].as_ref().unwrap()));
            }

            let fn_obj = InterpreterFunctionObj {
                name: function_id.clone(),
                param_names: param_names,
                param_pointers: param_pointers,
                body: fun_def.children[3].as_ref().unwrap().clone(),
            };

            self.function_defs.insert(function_id, fn_obj);
        }

        println!{"**********************************"};
        println!{"**********************************"};
        println!{"**********************************"};
        println!{"{:?}", self.function_defs};
        println!{"**********************************"};
        println!{"**********************************"};
        println!{"**********************************"};

        Ok(())
    }

    /// 
    fn eval_body(&mut self, tree: &ParseTree) -> Result<(), String> {
        println!{"EVAL BODY"};
        let mut is_other: bool = false;
        for child in &tree.children {
            match child.as_ref().unwrap().parse_type {
                ParseType::IF => self.eval_if(child.as_ref().unwrap())?,
                ParseType::LINK => self.eval_link(child.as_ref().unwrap())?,
                ParseType::UNLINK => self.eval_unlink(child.as_ref().unwrap())?,
                ParseType::WHILE => self.eval_while(child.as_ref().unwrap())?,
                ParseType::REPEAT => self.eval_repeat(child.as_ref().unwrap())?,
                ParseType::REPEATFOR => self.eval_repeat_for(child.as_ref().unwrap())?,
                ParseType::REPEATFOREVER => self.eval_repeat_forever(child.as_ref().unwrap())?,
                ParseType::QUIT => self.eval_quit(child.as_ref().unwrap())?,
                ParseType::BREAK => self.eval_break(child.as_ref().unwrap())?,
                ParseType::CONTINUE => self.eval_continue(child.as_ref().unwrap())?,
                ParseType::ASSIGN => self.eval_assignment(child.as_ref().unwrap())?,
                ParseType::RETURN => self.eval_return(child.as_ref().unwrap())?,
                _ => is_other = true,
            }
            // Catch anything with an actual return type
            if is_other {
                if child.as_ref().unwrap().parse_type == ParseType::VARDEF {
                    self.eval_vardef(child.as_ref().unwrap())?;
                }
                else {
                    self.eval_resolvable(child.as_ref().unwrap())?;
                }
                is_other = false
            }

            if self.loop_status != LoopStatus::DEFAULT {
                break;
            }
        }

        Ok(())
    }

    /// Todo Memory
    /// 
    fn eval_resolvable(&mut self, tree: &ParseTree) -> Result<LiteralValue, String> {
        // Catch literals
        if tree.parse_type == ParseType::LIT {
            return match &tree.token.token_type {
                TokenType::NUMBER(n) => Ok(LiteralValue::from_number(n.clone())),
                TokenType::TEXT(t) => Ok(LiteralValue::from_text(t.clone())),
                _ => Ok(LiteralValue::null())
            }
        }

        // Catch binary operators
        else if tree.parse_type == ParseType::BINOP {
            let left = self.eval_resolvable(tree.children[0].as_ref().unwrap())?;
            let right = self.eval_resolvable(tree.children[1].as_ref().unwrap())?;

            if !left.is_primitive || !right.is_primitive {
                return Err("Cannot perform binary operations on arrays".to_string());
            }

            if left.lit_type == "text".to_string() {
                let mut s:String = left.extract_text().unwrap_or("".to_string());
                s.push_str(&right.extract_text().unwrap_or("".to_string()));

                return Ok(LiteralValue::from_text(s));
            }
            else {
                let left_val: f64 = left.extract_number().unwrap_or(0.0);
                let right_val: f64 = right.extract_number().unwrap_or(0.0);

                if tree.token.token_type == TokenType::DIV &&
                   right_val == 0.0 {
                    return Err("Cannot divide by zero".to_string());
                }

                let result: f64 = match tree.token.token_type {
                    TokenType::ADD => left_val + right_val,
                    TokenType::SUB => left_val - right_val,
                    TokenType::MUL => left_val * right_val,
                    TokenType::DIV => left_val / right_val,
                    TokenType::POW => left_val.powf(right_val),
                    TokenType::MOD => ((left_val as i32) % (right_val as i32)) as f64,
                    TokenType::BAND => ((left_val as i32) & (right_val as i32)) as f64,
                    TokenType::BOR => ((left_val as i32) | (right_val as i32)) as f64,
                    TokenType::BXOR => ((left_val as i32) ^ (right_val as i32)) as f64,
                    TokenType::BSL => ((left_val as i32) << (right_val as i32)) as f64,
                    TokenType::BSR => ((left_val as i32) >> (right_val as i32)) as f64,
                    _ => 0.0,
                };

                println!{"DID MATH, GOT VALUE {} {} {} => {}", left_val, tree.token.lexeme.clone().unwrap_or("".to_string()), right_val, result};

                return Ok(LiteralValue::from_number(result));
            }
        }

        // Catch references
        else if tree.parse_type == ParseType::GETINDEX ||
                tree.parse_type == ParseType::GETSTRUCT ||
                tree.parse_type == ParseType::ID {
            let pointer = self.eval_reference(tree)?;
            println!("Looking for literal at pointer {:?}", pointer);
            return self.get_literal_in_memory(pointer);
        }

        // Catch function calls
        else if tree.parse_type == ParseType::CALL {
            // TODO
            // get all argument values
            let mut vals: Vec<LiteralValue> = Vec::new();
            for child in &tree.children[1].as_ref().unwrap().children {
                vals.push(self.eval_resolvable(child.as_ref().unwrap())?);
            }
            
            // scope in
            self.env.scope_in();

            // for each argument/param
            //  alloc space
            //  insert id
            let fn_id = unwrap_id_tree(tree.children[0].as_ref().unwrap());
            let param_names: Vec<String>;
            let param_pointers: Vec<Pointer>;
            let body: ParseTree;

            // There is a borrowing issue here, so my solution is to clone it
            param_names = self.function_defs[&fn_id].param_names.clone();
            param_pointers = self.function_defs[&fn_id].param_pointers.clone();
            body = self.function_defs[&fn_id].body.clone();

            for ((val, name), pointer) in vals.iter().zip(param_names.iter()).zip(param_pointers.iter()) {
                let mut p = pointer.clone();
                p.address = self.env.alloc(pointer.size);
                self.env.insert_id(name.clone(), p.clone())?;
                self.set_literal_in_memory(p.clone(), val.clone())?;
            }


            println!{"%%%%%%%%%%%%%%%%%%%"};
            println!{"{:?}", self.env.memory};
            println!{"%%%%%%%%%%%%%%%%%%%%"};

            // call body and capture return value
            let prev_return_val = self.return_value.clone();
            self.in_function_call += 1;
            self.eval_body(&body)?;
            let new_return_val = self.return_value.clone();
            self.return_value = prev_return_val;

            // scope out
            self.env.scope_out();

            // return the return value
            return Ok(new_return_val);
        }

        // Catch arrays (just shove in all children)
        else if tree.parse_type == ParseType::ARRAYLIT {
            let mut vec: Vec<LiteralValue> = Vec::new();

            for child in &tree.children {
                vec.push(self.eval_resolvable(child.as_ref().unwrap())?);
            }

            return Ok(LiteralValue{
                lit_type: "array".to_string(),
                is_primitive: false,
                values: Some(vec),
                value: None
            });
        }

        // Catch strucutres (just shove in all children)
        else if tree.parse_type == ParseType::STRUCTLIT {
            let mut vec: Vec<LiteralValue> = Vec::new();

            for child in &tree.children {
                vec.push(self.eval_resolvable(child.as_ref().unwrap())?);
            }

            return Ok(LiteralValue{
                lit_type: "strucutre".to_string(),
                is_primitive: false,
                values: Some(vec),
                value: None
            });
        }

        // Catch links
        else if tree.parse_type == ParseType::LINK {
            if tree.children[0].is_none() {
                return Ok(LiteralValue{
                    lit_type: "link".to_string(),
                    is_primitive: false,
                    values: None,
                    value: None,
                });
            }
            else {
                return Ok(LiteralValue{
                    lit_type: "link".to_string(),
                    is_primitive: false,
                    values: None,
                    value: Some(PrimitiveType::POINTER(Box::new(self.eval_reference(tree.children[0].as_ref().unwrap())?))),
                });
            }
        }
        
        Ok(LiteralValue::null())
    }

    /// Will evaluate the type of variable name and create memory space
    ///  for it. If it is a pointer, this space will not be created
    ///  (a link will be made for it instead)
    /// Assignment will be in charge of setting the pointer value
    fn eval_vardef(&mut self, tree: &ParseTree) -> Result<Pointer, String> {
        println!{"EVAL VARDEF"};
        // Get the pointer
        let mut pointer = self.eval_type(tree.children[1].as_ref().unwrap())?;

        // Make the actual allocations
        // ID
        if tree.children[0].as_ref().unwrap().parse_type == ParseType::ID {
            let id = unwrap_id_tree(tree.children[0].as_ref().unwrap());

            println!{"Adding symbol {}", id};
            pointer.address = self.env.alloc(pointer.size.clone());
            self.env.insert_id(id, pointer.clone());
        }
        // IDS
        else {
            for id_tree in &tree.children[0].as_ref().unwrap().children {
                let id = unwrap_id_tree(id_tree.as_ref().unwrap());

                println!{"Adding symbol {}", id};            
                pointer.address = self.env.alloc(pointer.size.clone());
                self.env.insert_id(id, pointer.clone());
            }
        }

        Ok(pointer)
    }

    /// Create a pointer that corresponds to the provided type
    /// This pointer will have an invalid memory address
    fn eval_type(&mut self, tree: &ParseTree) -> Result<Pointer, String> {
        // Get the type of variable
        let mut pointer = Pointer{
            pointer_type: PointerType::PRIMITIVE,
            size: 1,
            address: 0,
        };

        // Look for array definition
        if tree.parse_type == ParseType::ARRAYDEF {
            // Build the bounds for this array
            let mut bounds: Vec<(i32, i32)> = Vec::new();
            let mut size: usize = 1;

            for bound_tree in tree.children[0].as_ref().unwrap().clone().children {
                let start: i32;
                let end: i32;
                let mut res: LiteralValue;
                if bound_tree.as_ref().unwrap().children[0].is_none() {
                    start = 1;
                }
                else {
                    res = self.eval_resolvable(bound_tree.as_ref().unwrap().children[0].as_ref().unwrap())?;
                    start = res.extract_number().unwrap_or(1.0) as i32;
                }

                res = self.eval_resolvable(bound_tree.as_ref().unwrap().children[1].as_ref().unwrap())?;
                end = res.extract_number().unwrap_or(1.0) as i32;

                size *= ((end - start).abs() + 1) as usize;
                bounds.push((start, end));
            }

            // Find the actual type
            let var_type = unwrap_type_tree(tree.children[1].as_ref().unwrap());
            if var_type != "number".to_string() && var_type != "text".to_string() {
                let struct_pointer = PointerType::STRUCTURE(var_type);
                pointer.pointer_type = PointerType::ARRAY(bounds, Box::new(struct_pointer));
            }
            else {
                pointer.pointer_type = PointerType::ARRAY(bounds, Box::new(PointerType::PRIMITIVE));
            }

            pointer.size = size as usize;
        }
        // Else check for pointers
        else if tree.parse_type == ParseType::POINTER {
            let link_type = self.eval_type(tree.children[0].as_ref().unwrap())?;
            pointer.pointer_type = PointerType::LINK(Box::new(link_type.pointer_type));
        }
        // Otherwise, look for structures/primitives
        else {
            let var_type = unwrap_type_tree(&tree);
            if var_type != "number".to_string() && var_type != "text".to_string() {
                pointer.pointer_type = PointerType::STRUCTURE(var_type.clone());
                pointer.size = self.get_struct_size(var_type)?;
            }
        }

        Ok(pointer)
    }

    ///            <>!=      if   elif    else
    /// Children: BINCOMP, BLOCK, (IF || BLOCK)
    fn eval_if(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Evaluate the comparison
        let cond = self.eval_conditional(tree.children[0].as_ref().unwrap())?;

        // Evaluate the body
        if bool::from(cond) {
            self.symbol_table.scope_in();
            self.eval_body(tree.children[1].as_ref().unwrap())?;
            self.symbol_table.scope_out();
        }
        else {
            if tree.children[2].is_some() {
                // Evaluate an else if
                if tree.children[2].as_ref().unwrap().parse_type == ParseType::IF {
                    self.eval_if(tree.children[2].as_ref().unwrap())?;
                }
    
                // Evaluate an else block if it exists
                else if tree.children[2].as_ref().unwrap().parse_type == ParseType::BLOCK {
                    self.symbol_table.scope_in();
                    self.eval_body(tree.children[2].as_ref().unwrap())?;
                    self.symbol_table.scope_out();
                }
            }
        }

        Ok(())
    }

    /// Todo Memory
    /// 
    fn eval_conditional(&mut self, tree: &ParseTree) -> Result<PrimitiveType, String> {
        Ok(PrimitiveType::NOTHING)
    }

    /// Todo Memory
    /// 
    fn eval_link(&mut self, tree: &ParseTree) -> Result<(), String> {
        Ok(())
    }

    /// Todo Memory
    /// 
    fn eval_unlink(&mut self, tree: &ParseTree) -> Result<(), String> {
        Ok(())
    }

    /// 
    fn eval_while(&mut self, tree: &ParseTree) -> Result<(), String> {
        // Evaluate the comparison
        while bool::from(self.eval_conditional(tree.children[0].as_ref().unwrap())?) {
            // Evaluate the while block
            self.symbol_table.scope_in();
            self.eval_body(tree.children[1].as_ref().unwrap())?;
            self.symbol_table.scope_out();

            if self.loop_status == LoopStatus::BREAK {
                self.loop_status = LoopStatus::DEFAULT;
                break;
            }
            if self.loop_status == LoopStatus::CONTINUE {
                self.loop_status = LoopStatus::DEFAULT;
            }
            if self.loop_status == LoopStatus::RETURN {
                break;
            }
        }

        Ok(())
    }

    /// Todo Memory
    /// 
    fn eval_repeat(&mut self, tree: &ParseTree) -> Result<(), String> {
        Ok(())
    }

    /// Todo Memory
    /// 
    fn eval_repeat_for(&mut self, tree: &ParseTree) -> Result<(), String> {
        Ok(())
    }

    /// Todo Memory
    /// 
    fn eval_repeat_forever(&mut self, tree: &ParseTree) -> Result<(), String> {
        Ok(())
    }

    /// Todo Memory
    /// 
    fn eval_assignment(&mut self, tree: &ParseTree) -> Result<(), String> {

        // Get the value to assign
        let res = self.eval_resolvable(tree.children[1].as_ref().unwrap())?;

        // Get the address of where to assign it
        let pointer: Pointer;
        if tree.children[0].as_ref().unwrap().parse_type == ParseType::VARDEF {
            pointer = self.eval_vardef(tree.children[0].as_ref().unwrap())?;
        }
        else {
            pointer = self.eval_reference(tree.children[0].as_ref().unwrap())?;
        }

        // Make the assignment
        self.set_literal_in_memory(pointer, res)?;

        Ok(())
    }

    // Given an ID and some different referencing, find the original pointer
    //  and modify it to match the specific referencing
    // Array -> move address and change bounds (or delete bounds)
    // Struct -> move address based on key
    fn eval_reference(&mut self, tree: &ParseTree) -> Result<Pointer, String> {

        let mut ptr: Pointer;

        if tree.parse_type != ParseType::ID {
            ptr = self.eval_reference(tree.children[0].as_ref().unwrap())?;

            if tree.parse_type == ParseType::GETINDEX {
                let (mut bounds, arr_type) = match ptr.pointer_type.clone(){
                    PointerType::ARRAY(bounds, arr_type) => (bounds.clone(), arr_type),
                    _ => return Err("Cannot index a non-array".to_string()),
                };

                println!{"STARTING ADDRESS == {} == ", ptr.address};

                // Figure out how far we need to offset the new pointer
                let mut offset = 1;
                for dim in bounds.iter().skip(1) {
                    offset *= (dim.0 - dim.1).abs() + 1;
                }

                // check all indexes to make sure they are numbers
                let index_tree = tree.children[1].as_ref().unwrap();
                for (idx, bound) in index_tree.children.iter().zip(bounds.clone().iter()) {
                    let idx_val = self.eval_resolvable(idx.as_ref().unwrap())?.extract_number().unwrap_or(0.0) as i32;

                    println!{"INDEXING ARRAY AT POSISION {}", idx_val};

                    // low to high bounds
                    if bound.0 < bound.1 {
                        if idx_val < bound.0 || idx_val > bound.1 {
                            return Err(format!{"Index out of bounds for index {} in range {} to {}", idx_val, bound.0, bound.1});
                        }

                        // remove this dimension
                        bounds = bounds.into_iter().rev().skip(1).rev().collect();
                        // move the address
                        ptr.address += (offset * (idx_val as i32 - bound.0)) as usize;
                        // change the remaining offset
                        offset /= (bound.0 - bound.1).abs() + 1;
                    }
                    // high to low bounds
                    else {
                        if idx_val > bound.0 || idx_val < bound.1 {
                            return Err(format!{"Index out of bounds for index {} in range {} to {}", idx_val, bound.0, bound.1});
                        }

                        // remove this dimension
                        bounds = bounds.into_iter().rev().skip(1).rev().collect();
                        // move the address
                        ptr.address += (offset * (bound.0 - idx_val as i32)) as usize;
                        // change the remaining offset
                        offset /= (bound.0 - bound.1).abs() + 1;
                    }

                    println!{"MOVED TO ADDRESS == {} == ", ptr.address};
                    // catch division rounding errors that lowers offset too much
                    if offset == 0 {
                        offset = 1;
                    }
                }
                
                if bounds.len() == 0 {
                    ptr.pointer_type = *arr_type.clone();
                    ptr.size = 1;
                }
                else {
                    ptr.size = (offset * ((bounds[0].0 - bounds[0].1).abs() + 1)) as usize;
                    ptr.pointer_type = PointerType::ARRAY(bounds, Box::new(*arr_type.clone()));
                }

                println!{"ENDING ADDRESS == {} == ", ptr.address};

                return Ok(ptr);
            }

            else if tree.parse_type == ParseType::GETSTRUCT {
                let struct_key = unwrap_id_tree(tree.children[1].as_ref().unwrap());

                
            }
        }

        println!("FINDING SYMBOL {}", unwrap_id_tree(&tree));
        self.env.get_id(unwrap_id_tree(&tree))
    }

    /// 
    fn eval_return(&mut self, tree: &ParseTree) -> Result<(), String> {
        self.return_value = LiteralValue::null();
        self.loop_status = LoopStatus::RETURN;

        // Check to see if the return type is nothing
        if tree.children[0].is_some() {
            self.return_value = self.eval_resolvable(tree.children[0].as_ref().unwrap())?;
        }

        // Otherwise return Ok
        Ok(())
    }

    fn eval_quit(&mut self, tree: &ParseTree) -> Result<(), String> {
        process::exit(0);
        Ok(())
    }

    fn eval_continue(&mut self, tree: &ParseTree) -> Result<(), String> {
        self.loop_status = LoopStatus::CONTINUE;
        Ok(())
    }

    fn eval_break(&mut self, tree: &ParseTree) -> Result<(), String> {
        self.loop_status = LoopStatus::BREAK;
        Ok(())
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

    tree.clone().expect("error").print();
    println!("\n\n\n\n");

    let mut int = Interpreter::new(tree.as_ref().unwrap());
    int.eval(tree.as_ref().unwrap()).unwrap();

}