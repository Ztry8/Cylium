// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use interpreter::Interpreter;

mod errors;
mod interpreter;
mod tokenizer;
mod types;

fn get_error(line_number: usize, line: &str, error: &str) -> String {
    format!(
        "{}|{}\nError {}\nFor details, visit: https://cylium.site/materials/errors",
        line_number, line, error,
    )
}

fn show_error(line_number: usize, line: &str, error: &str) -> ! {
    panic!("{}", get_error(line_number, line, error))
}

fn show_warning(line_number: usize, line: &str, warning: &str) {
    println!(
        "{}|{}\nWarning {}\nFor details, visit: https://cylium.site/materials/errors",
        line_number, line, warning,
    )
}

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
                    "Cylium Interpreter {}\nhttps://cylium.site\n\nUsage:\n    cylium <file>\n    cylium <command>\n",
                    env!("CARGO_PKG_VERSION")
                );

                println!(
                    "Commands:\n    help    Show this help message\n    about   License, copyright and credits\n",
                );

                println!("Arguments:\n    <file>  Source file to execute\n",);
            }
            "about" => {
                println!(
                    "Cylium Interpreter {}\nhttps://cylium.site\n\nCopyright (c) 2026 Ztry8 (AslanD)\nAll rights reserved.\n",
                    env!("CARGO_PKG_VERSION")
                );

                println!(
                    "License:\n    This software is licensed under the Apache-2.0 License.\n    https://https://apache.org/licenses/\n"
                );

                println!(
                    "Credits:\n    Author: Ztry8 (AslanD)\n    https://cylium.site/contributors\n"
                );
            }
            _ => match std::fs::read_to_string(&file_name) {
                Ok(file) => {
                    if !file_name.ends_with(".cyl") {
                        println!("Warning: File extension must be '.cyl'\n");
                    }

                    let mut interpreter =
                        Interpreter::new(file.lines().map(String::from).collect());

                    interpreter.run();
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
