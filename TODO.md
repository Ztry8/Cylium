## Cylium Language Development Roadmap

* [x] Switch to AST-based execution instead of string matching
  * [x] Implement `if-else` block
  * [x] Add `proc` (procedures) support
  * [x] Implement `while` loop
  * [x] Implement `for` loop

* [x] Replace AST interpreter with IR (bytecode) based interpreter
  * [x] Implement `if-else` block
  * [x] Implement `while` loop
  * [x] Implement `for` loop

* [x] Extend the core language
  * [x] Replace procedures with functions
  * [x] Add arrays

* [x] Replace the bytecode interpreter backend with AOT compilation to C

* [ ] Add data structures
  * [ ] Add structs
  * [ ] Add unions
  * [ ] Add references to variables
  * [ ] Add `include` and support multi-file scripts

* [ ] Foreign Function Interface (FFI)
  * [ ] Rust integration
  * [ ] C/C++ integration

* [ ] Build the standard library
  * [ ] Math operations
  * [ ] Time operations
  * [ ] Vector operations
  * [ ] String manipulation
  * [ ] Random number generation