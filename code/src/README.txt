==========================
=   NOTES FOR THE CODE   =
==========================

- Using the cargo package, you can run this code with the command
   "cargo run" while in the /code/ directory.

- To feed in an input file, use the command:
   "cargo run -- filename.txt", replacing the filename with
   the file you want to interpret.

- You can redirect the output if you want to save it to a file:
   "cargo run -- filename.txt > output_filename.txt", this will
   make reading the output easier.

- For the parser, semantic_analyzer, and interpreter (and library_handler), 
   the top of each file has a DEBUG boolean. Setting this to "true" will
   print debug information that shows some of the internal operations of
   each of these objects.

=============================
=   NOTES FOR FUTURE WORK   =
=============================

- As the code stands, we believe we have successfully
   implemented all features of the lexer, parser, and semantic_analyzer
   to match the specifications given in our BNF and language description.
   However, the interpreter has not gone through extensive bug testing,
   and does not have proper implementation for "links" (or pointers).
   The code for the functions of "unlink", "link", and any memory
   management functions that rely on these are either unimplemented,
   or partially implemented, but untested.

- To properly handle "links", the memory management system may need to
   undergo a rework to handle some behind the scences features like
   garbage collection and proper memory deallocation. This may also
   require a redescription of how links work in syntax (meaning a
   change to the parser and semantic_analyzer).

- One other note, memory management does not not perform recursive sizing 
   of features. This means that for a complicated variable (like an array 
   of structures), the array object in memory creates pointers to structures:

   '[..., struct1_ptr, struct2_ptr, struct3_ptr, ..., struct1.key1, struct1.key2, ... struct2.key1, ...]'
          ^ array start                   array end ^ ^ struct1 start                 ^ struct2 start

   instead of dynamically sizing and allocating the entire variable space:

   '[..., struct1.key1, struct2.key2, ..., struct2.key1, ..., structN.keyN, ...]'
          ^ array start (and struct1 start)                              ^ array end

    This would make memory management slightly easier, but would require reimplementation
     of a large majority of the memory management code.

- The library_handler file serves as the gateway between our Rust code, and WebAssembly.
   The idea behind it is to be able to pull in a list of expected functions (or manually
   insert them), and then use this library_handler to alert the semantic_analyzer (and interpreter)
   of the expected external functions for error checking and management. Many of the expected
   built-in functions have yet to be created (such as "upper_bound", "lower_bound", "wait", etc.)
   They would all need some sort of header object for the semantic analyzer, and implementations
   that would be called in the interpreter.
