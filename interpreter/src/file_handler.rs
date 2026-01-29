// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

pub struct FileHandler {
    pub ready_file: Vec<String>,
    raw_file: Vec<String>,
}

impl FileHandler {
    pub fn new(file: Vec<String>) -> Self {
        Self {
            raw_file: file
                .clone()
                .into_iter()
                .map(|line| line.trim().to_owned())
                .collect(),
            ready_file: file
                .into_iter()
                .map(|line| line.trim().to_owned())
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .collect(),
        }
    }

    fn get_error(line_number: usize, line: &str, error: &str) -> String {
        format!(
            "{}|{}\nError {}\nFor details, visit: https://cylium.site/materials/errors",
            line_number + 1,
            line,
            error,
        )
    }

    pub fn show_error(&self, line_number: usize, error: &str) -> ! {
        let line = &self.ready_file[line_number];

        panic!(
            "{}",
            Self::get_error(
                self.raw_file.iter().position(|x| x == line).unwrap(),
                line,
                error
            )
        )
    }

    pub fn show_warning(warning: &str) {
        println!(
            "Warning {}\nFor details, visit: https://cylium.site/materials/errors",
            warning,
        )
    }
}
