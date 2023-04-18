#![allow(dead_code)]

use std::{env, process};
use crate::parser::{Parser, ParseTree, ParseType};
use crate::semantic_analyzer::{SemanticAnalyzer, SymbolTable};

#[derive(Clone, PartialEq, Debug)]
pub enum PrimitiveType {
    NUMBER(f64),
    TEXT(String),
    NOTHING,
}

impl From <PrimitiveType> for bool {
    fn from(t: PrimitiveType) -> bool {
        match t {
            PrimitiveType::TEXT(t) => t.len() > 0,
            PrimitiveType::NUMBER(n) => n != 0.0,
            PrimitiveType::NOTHING => false,
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