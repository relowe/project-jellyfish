use crate::semantic_analyzer::{SymbolType, FunctionObject};
use crate::interpreter::{LiteralValue};
use std::collections::HashMap;

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
    println!{"Attempting to call external function {}", name};
    match name.as_str() {
        "print" | "display" => {
            print!{">> "};
            for val in vals {
                print!{"{}", val.to_string()};
            }
            println!{};
            return Ok(LiteralValue::null());
        },


        _ => { return Ok(LiteralValue::null()); }
    }
}