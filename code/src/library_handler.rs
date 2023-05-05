use crate::semantic_analyzer::{SymbolType, FunctionObject};
use crate::interpreter::{LiteralValue};
use std::collections::HashMap;

// A boolean to determine if debug information should be displayed
static DEBUG: bool = false;

// Handle error reporting through web assembly
// For right now we just print the error, but later
//  on this would be passed to JavaScript code
macro_rules! log {
    ($($t:tt)*) => (print!("{}",  &format_args!($ ( $t ) *).to_string() ))
}
macro_rules! logln {
    ($($t:tt)*) => (println!("{}",  &format_args!($ ( $t ) *).to_string() ))
}

// Handle debugging through web assembly
// For right now we just print the error, but later
//  on this would be passed to JavaScript code
macro_rules! debug {
    ($($t:tt)*) => {
        if DEBUG {
            (println!("LIB: {}",  &format_args!($ ( $t ) *).to_string() ))
        }
    }
}

// Collect and return a list of all external functions
pub fn get_external_functions() -> HashMap<String, FunctionObject> {
    let mut map: HashMap<String, FunctionObject> = HashMap::new();

    let mut lwb = FunctionObject {
        params: Vec::new(),
        return_type: "number".to_string(),
    };
    lwb.params.push(SymbolType{
        basic_type: "*".to_string(),
        is_pointer: false,
        array_dimensions: -1,
    });
    map.insert("lower_bound".to_string(), lwb);

    let mut upb = FunctionObject {
        params: Vec::new(),
        return_type: "number".to_string(),
    };
    upb.params.push(SymbolType{
        basic_type: "*".to_string(),
        is_pointer: false,
        array_dimensions: -1,
    });
    map.insert("upper_bound".to_string(), upb);


    map
}

// Actually handle a function call
pub fn handle_call(name: String, vals: Vec<LiteralValue>) -> Result<LiteralValue, String> {
    debug!{"Attempting to call external function {}", name};
    match name.as_str() {
        "print" | "display" => {
            for val in vals {
                log!{"{}", val.to_string()};
            }
            logln!{""};
            return Ok(LiteralValue::null());
        },


        _ => { return Ok(LiteralValue::null()); }
    }
}