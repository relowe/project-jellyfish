# WASM Pack
To test WebAssembly, we have made use of the "wasm-pack"
that allows for easy building and exporting of rust packages
into web assembly.

The tutorial (and manual) for this pack can be found here:
https://rustwasm.github.io/book/

This code itself has two main parts, the Rust part, and
 the Javascript part (using WebASM)

# RUST
All self-written rust code can be found in the
'/jelly-wasm/src/' folder. In this folder, we have
an implementation for an older version of the lexer,
stored inside of the "lexer.rs" file. To alert WebASM
of any internal Rust functions, they can be put in the "lib.rs"
file, with the '#[wasm_bindgen]' header.

"utils.rs" contains a panic hook function, which allows
any internal errors to properly backtrace so that the user
can get a meaningful message from them.

To compile this rust code (as changes are made), you can use
"wasm-pack build" command inside of the '/jelly-wasm/' directory.

# JavaScript (and WebAssembly)
After building from the 'wasm-pack build' command, a new "pkg"
folder should be generated. This folder contains the WebAssembly
code that can be accessed by JavaScript.

To test this code, we have made a simple NPM project under the
'jelly-wasm/www/' directory. This project contains two terminals,
one for user input, and one for user output. After lauching this
npm project, a user can type in the left terminal, and hit the
green play button to run the typed code through our lexer,
generating the output on the right.

To run this project, you can run the command:
'npm run start' while inside of the 'jelly-wasm/www/' directory.

For some computers, you may run into a path length error.
Depending on your machine, you should be able to go into your 
PATH variables to fix this issue.

After running successfuly, the website should be hosted localy at:
'http://localhost:8080/'
