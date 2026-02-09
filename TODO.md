## Cylium Language Development Roadmap

* [x] Switch to AST-based execution instead of string matching
  * [x] Implement `if-else` block
  * [x] Add `proc` (procedures) support
  * [x] Implement `while` loop
  * [x] Implement `for` loop

* [x] Replace AST interpreter with IR (bytecode) based interpreter
  * [ ] Implement `if-else` block
  * [ ] Implement `while` loop
  * [ ] Implement `for` loop

* [ ] Replace procedures with functions 
* [ ] Add vectors type
* [ ] Add `include` and support multi-file scripts

* [ ] Build the standard library
  * [ ] Math operations
  * [ ] Vector operations
  * [ ] String manipulation
  * [ ] Random number generation
  * [ ] File system API
  * [ ] Networking API

* [ ] Replace interpreter with JIT compilation ?
* [ ] Community libraries support

* [ ] Foreign Function Interface (FFI)
  * [ ] Rust integration
  * [ ] C/C++ integration