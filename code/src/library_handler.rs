use crate::semantic_analyzer::{SymbolType, FunctionObject};
use std::collections::HashMap;

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