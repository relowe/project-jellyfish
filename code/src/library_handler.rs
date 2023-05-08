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

// The library handler file is in charge of compiling a collection of
//  all expected external function calles (or build in library functions)
//  and defining them in a way that allows for error checking.

// Collect and return a list of all external functions.
// For each function, generage a FunctionObject that the semantic
//  analyzer can use to ensure that the arguments for the function
//  are passed correctly.
pub fn get_external_functions() -> HashMap<String, FunctionObject> {
    let mut map: HashMap<String, FunctionObject> = HashMap::new();

    // Sample definition of the lowerbound function (lwb)
    // Note that below in the handle_call function, there is
    //  no actual implementation for the "lwb" or "upb" function yet.
    let mut lwb = FunctionObject {
        params: Vec::new(),
        return_type: "number".to_string(),
    };
    // Here, a * means any data type, and array dimenstions of -1
    //  allow for any sized array
    lwb.params.push(SymbolType{
        basic_type: "*".to_string(),
        is_pointer: false,
        array_dimensions: -1,
    });
    map.insert("lower_bound".to_string(), lwb);

    // Sample definition of the upperbound function (upb)
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

// Actually handle a function call. The arguments for the function come in from
//  the interpreter as a LiteralValue. By defining the arguments in "get_external_functions" above,
//  you can ensure that the arguments will match the expected types. Without a definition above,
//  any set of arguments will be accpeted as valid (like with the "print" function)
pub fn handle_call(name: String, vals: Vec<LiteralValue>) -> Result<LiteralValue, String> {
    debug!{"Attempting to call external function {}", name};

    match name.as_str() {
        // This is the "print" function. It also accepts the name "display". 
        // Each argument (or literal value) is converted to a string and printed.
        "print" | "display" => {
            for val in vals {
                log!{"{}", val.to_string()};
            }
            logln!{""};
            return Ok(LiteralValue::null());
        },

        // Other external (or library) functions would be handled here
        // "function_name" => { //todo },


        // In the default case, we are just returning null. For proper handling, an error
        //  should be genrated and returned that alerts the user of an invalid function call.
        _ => { return Ok(LiteralValue::null()); }
    }
}