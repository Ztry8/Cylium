// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use file_handler::FileHandler;
use parser::Parser;

mod codegen;
mod compiler;
mod errors;
mod file_handler;
mod lexer;
mod parser;
mod types;
mod validator;

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("{}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            println!("{}", s);
        } else {
            unreachable!()
        }
    }));

    let mut args = std::env::args();
    args.next();

    if let Some(file_name) = args.next() {
        match file_name.as_str() {
            "help" => {
                println!(
                    "Cylium Compiler {}\nhttps://cylium.site\n\nUsage:\n    cylium <file> [-o <output>]\n    cylium <command>\n",
                    env!("CARGO_PKG_VERSION")
                );

                println!(
                    "Commands:\n    help    Show this help message\n    about   License, copyright and credits\n",
                );

                println!(
                    "Arguments:\n    <file>      Source file to compile\n    -o <output> Output executable path (default: same name as <file>, without '.cyl')\n",
                );
            }
            "about" => {
                println!(
                    "Cylium Compiler {}\nhttps://cylium.site\n\nCopyright (c) 2026 Ztry8 (AslanD)\nAll rights reserved.\n",
                    env!("CARGO_PKG_VERSION")
                );

                println!(
                    "License:\n    This software is licensed under the Apache-2.0 License.\n    https://apache.org/licenses/\n"
                );

                println!(
                    "Credits:\n    Author: Ztry8 (AslanD)\n    https://github.com/Ztry8/Cylium/graphs/contributors\n"
                );
            }
            _ => match std::fs::read_to_string(&file_name) {
                Ok(file) => {
                    if !file_name.ends_with(".cyl") {
                        println!("Warning: File extension must be '.cyl'\n");
                    }

                    let file: Vec<String> = file.lines().map(String::from).collect();
                    let handler = FileHandler::new(file);

                    let tokens = lexer::tokenize_file(&handler);

                    let mut parser = Parser::new(tokens);
                    let mut ast = parser.start(&handler);

                    validator::check_types(&handler, &mut ast);

                    let output_path = output_path_for(&file_name, &mut args);

                    match compiler::compile_and_build(&ast, &output_path) {
                        Ok(built_path) => {
                            println!("Compiled successfully: {}", built_path.display());
                        }
                        Err(e) => {
                            println!("Error: {e}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(_) => {
                    println!("Error: Specified file not found.");
                }
            },
        }
    } else {
        println!("Error: Expected 1 argument. Type 'help' for assistance");
    }
}

fn output_path_for(file_name: &str, args: &mut std::env::Args) -> std::path::PathBuf {
    let mut explicit_output = None;
    while let Some(arg) = args.next() {
        if arg == "-o" {
            explicit_output = args.next();
            break;
        }
    }

    if let Some(out) = explicit_output {
        return std::path::PathBuf::from(out);
    }

    match file_name.strip_suffix(".cyl") {
        Some(stem) => std::path::PathBuf::from(stem),
        None => std::path::PathBuf::from(format!("{file_name}.out")),
    }
}
