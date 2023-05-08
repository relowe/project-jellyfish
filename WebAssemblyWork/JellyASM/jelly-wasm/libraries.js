
const function_map = new Map();
function_map.set('name', name);

export function call(name, args) {
    if(function_map.has(name)) {
        let fn = function_map.get(name);
        return fn.call(fn, args);
    }
    return null;
}

export function add_function(name, fn) {
    function_map.set(name, fn);
}

export function name() {
    return 'Rust';
}

export function edit_html() {
    var title = document.getElementById("title-text");
    console.log("HELLO WORLD!!!");
    if(title != null) {
        title.textContent = "Changed!";
        return true;
    }
    return false;
}