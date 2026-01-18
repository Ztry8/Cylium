<div align="center">
  
# Cylium
## Imperative, strongly typed scripting language
[Website](https://cylium.site) | [Getting started](https://cylium.site/tutorials) | [Docs](https://cylium.site/docs)

</div>

### What is Cylium?

Cylium is a minimalistic programming language focused on clarity and control.    
It combines the efficiency of low-level languages such as C and Rust    
with a clean, easy-to-learn syntax.

The language emphasizes simplicity while still offering precise control over memory.    
Explicit memory management is available when needed, without unnecessary boilerplate.

- **Clear and minimal design** — clean, expressive syntax focused on readability and control

- **Low-level efficiency** — inspired by the performance and control of C and Rust

- **Flexible usage** — suitable for scripting, tooling, and systems-level development

#### Example:
```py
# read input from user
var name = input

# printing on stdout
echo Hello, {name}!

# exit the program gracefully
delete name
exit 0
```

### Read More

This is the official and primary repository of the Cylium programming language.    
It includes the full interpreter source code and all development-related resources.    
Documentation and tutorials are available on the [official website](https://cylium.site).

### Compilation from Source

You can compile Cylium yourself from source by following these steps:    
(Precompiled binaries are also available for download [here](https://cylium.site/))

1. **Install Rust**  
   Follow the instructions at [https://rustup.rs](https://rustup.rs) to install Rust and Cargo.

2. **Clone the repository**  
   ```
   git clone https://github.com/ztry8/cylium.git
   cd cylium/interpreter
   ```
   
3. **Build the release version**  
   ```
   cargo build --release
   ```

After building, the compiled binary will be available in `target/release/`

### [Contributing](https://github.com/Ztry8/Cylium?tab=contributing-ov-file)

### [License](https://github.com/Ztry8/Cylium?tab=Apache-2.0-1-ov-file)

### [TODO](https://github.com/Ztry8/Cylium/blob/main/TODO.md)
