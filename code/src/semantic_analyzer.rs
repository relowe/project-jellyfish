#![allow(dead_code)]

use std::{env, process};
use crate::parser;

#[derive(Debug, PartialEq, Clone)]
pub enum VarType {
    TEXT,
    NUMBER,
    FUNCTION,
    STRUCT,
    ARRAY,
    POINTER,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    name: &str,
    var_type: VarType,
    is_pointer: bool
}

impl SemanticAnalyzer {
    
}

pub fn main() {

}