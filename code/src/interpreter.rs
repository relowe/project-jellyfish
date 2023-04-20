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
    PRIMITIVE,
    ARRAY(i32, i32, Box<PrimitiveType>), // start and end index, and array type
    STRUCTURE(String), // structure ID
}

// the pointer structure itself
#[derive(Clone, PartialEq, Debug)]
pub struct Pointer {
    address: usize,
    size: usize,
    pointer_type: PointerType,
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
    heap: BinaryHeap<MemorySpace>
}


impl Environment {
    // Create a new instance of the memory
    pub fn new() -> Self {
        // Create the environment
        let mut env = Environment {
            namespace: Vec::new(),
            memory: Vec::new(),
            heap: BinaryHeap::new(),
        };

        // Fill in the first namespace with a hashmap
        env.namespace.push(HashMap::new());

        // Return the environment
        env
    }

    fn alloc(&mut self, size: usize) -> usize {
        // find a memory address of the requested size
        // if no memory exists of this size, create it at the
        //  end of the memory space

        let mem_peek = self.heap.peek();

        if mem_peek.is_none() || mem_peek.unwrap().size < size {
            let addr:usize = self.memory.len();

            for i in 0..size {
                self.memory.push(PrimitiveType::NOTHING);
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

    fn get_by_address(&mut self, p: Pointer) -> LiteralValue {
        LiteralValue {
            lit_type: PointerType::PRIMITIVE,
            value: Vec::new()
        }
    }

    // < REFERENCE > (by value)
    fn get_id_value(&self, id: String, recursive: bool) -> LiteralValue {
        // returns the value at the provided id tree
        // this will handle arrays and structs, etc
        // if the value does not exist (is nothing), then throw an error

        LiteralValue {
            lit_type: PointerType::PRIMITIVE,
            value: Vec::new()
        }
    }

    // < REFERENCE > (by reference)
    fn get_id_address(&self, id: String, recursive: bool) -> Pointer {
        // returns the address at the given id tree

        Pointer {
            address: 0,
            size: 0,
            pointer_type: PointerType::PRIMITIVE,
        }
    }

    // < VARDEF >
    fn insert_name(&mut self, id: String, var_type: SymbolType) -> Pointer {
        // inserts a name into the namespace using the provided parse tree
        // if the name exists already, error


        Pointer {
            address: 0,
            size: 0,
            pointer_type: PointerType::PRIMITIVE,
        }
    }

    fn set_value(&mut self, pointer: Pointer, value: LiteralValue) {
        // sets the value at the pointer
        // for arrays, this will run a for-each loop
        // for structures, this will also run a for-each loop
        // complicated structures may need a recursive calls
    }

    fn scope_in(&mut self) {
        // scope in the environment
    }

    fn scope_out(&mut self) {
        // scope out the environment and delete all
        // the memory used by the previous scope (namespace)
        // this is essentially dealloc
    }

    fn build_heap(&mut self) {
        // clear and rebuild the available heap
        // this is a lazy version of handling merging heap items
        //  together, by just emptying and reubilding the enitre heap
        // should be called after scoping out

        let mut addr: usize = 0;
        self.heap.clear();

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


#[derive(Debug)]
pub struct LiteralValue {
    lit_type: PointerType,
    value: Vec<PrimitiveType>
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

#[derive(Clone, Debug)]
pub struct SymbolType {
    pub basic_type: String,
    pub is_pointer: bool,
    pub array_dimensions: Vec<(i32, i32)>,
}


#[derive(Clone, PartialEq, Debug)]
pub enum PrimitiveType {
    NUMBER(f64),
    TEXT(String),
    NOTHING,
    POINTER(Box<Pointer>),
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

impl From <PrimitiveType> for i32 {
    fn from(t: PrimitiveType) -> i32 {
        match t {
            PrimitiveType::NUMBER(n) => n as i32,
            _ => 0,
        }
    }
}

impl From <PrimitiveType> for String {
    fn from(t: PrimitiveType) -> String {
        match t {
            PrimitiveType::TEXT(t) => t,
            _ => "".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub enum LoopStatus {
    DEFAULT,
    BREAK,
    CONTINUE,
    RETURN,
}

pub struct Interpreter {
    symbol_table: SymbolTable,
    return_value: PrimitiveType,
    loop_status: LoopStatus,
    memory: (),
}

impl Interpreter {
    pub fn new(tree: &ParseTree) -> Self {
        let symtab = SemanticAnalyzer::new().analyze(tree).expect("Semantic Analyzer errored");
        Interpreter {
            symbol_table: symtab,
            return_value: PrimitiveType::NOTHING,
            loop_status: LoopStatus::DEFAULT,
            memory: (),
        }
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
        Ok(())
    }

    /// 
    fn eval_body(&mut self, tree: &ParseTree) -> Result<(), String> {
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
    fn eval_resolvable(&mut self, tree: &ParseTree) -> Result<PrimitiveType, String> {
        Ok(PrimitiveType::NOTHING)
    }

    /// Todo Memory
    /// 
    fn eval_vardef(&mut self, tree: &ParseTree) -> Result<PrimitiveType, String> {
        Ok(PrimitiveType::NOTHING)
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
        Ok(())
    }

    /// 
    fn eval_return(&mut self, tree: &ParseTree) -> Result<(), String> {
        self.return_value = PrimitiveType::NOTHING;
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

    fn eval_type(&mut self, tree: &ParseTree) -> Result<SymbolType, String> {
        //tree.print();

        let mut curr_tree = tree;
        let mut is_pointer = false;
        let mut array_dimensions: Vec<(i32, i32)> = Vec::new();

        if curr_tree.parse_type == ParseType::POINTER {
            is_pointer = true;
            curr_tree = curr_tree.children[0].as_ref().unwrap();
        }

        if curr_tree.parse_type == ParseType::ARRAYDEF {
            //curr_tree.print();

            // If there are defined bounds, make sure they are integers
            // This will also set the array_dimensions
            // Otherwise, we don't know the current dimensions until later
            if curr_tree.children[0].as_ref().is_some() {
                let bounds_tree = curr_tree.children[0].as_ref().unwrap();

                for bound in &bounds_tree.children {
                    let mut bound_type;
                    if bound.as_ref().unwrap().children[0].as_ref().is_some() {
                        bound_type = self.eval_resolvable(bound.as_ref().unwrap().children[0].as_ref().unwrap())?;
                    }

                    bound_type = self.eval_resolvable(bound.as_ref().unwrap().children[1].as_ref().unwrap())?;
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


}

pub fn main() {
    /*
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
    */

    let mut env = Environment::new();

    for i in 0..10 {
        println!{"Alloc {}: {:?}", i, env.alloc(1)};

        if i % 2 == 0 {
            env.memory.push(PrimitiveType::NUMBER(5.0)); 
        }
    }

    env.build_heap();

    println!{"{:?}", env};

    env.memory.push(PrimitiveType::NUMBER(5.0));

    for i in 0..2 {
        println!{"Alloc {}: {:?}", i, env.alloc(1)};
    }

    println!{"{:?}", env};
}