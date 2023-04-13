#![allow(dead_code)]

use std::{env, process};
use std::collections::HashMap;
use crate::lexer::{Token, TokenType};
use crate::parser::{ParseTree, ParseType};
use crate::semantic_analyzer::{SemanticAnalyzer};

