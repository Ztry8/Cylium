// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{errors, get_error, show_error, tokenizer::tokenize, types::Types};
use std::{collections::HashMap, mem::discriminant};

pub type Variable = (Types, VariableType);

#[derive(Clone, PartialEq)]
pub enum VariableType {
    Const,
    Local,
}

pub struct Interpreter {
    variables: HashMap<String, Variable>,
    labels: HashMap<String, usize>,
    file: Vec<String>,
}

impl Interpreter {
    pub fn new(file: Vec<String>) -> Self {
        let mut labels = HashMap::new();

        for (number, line) in file.iter().enumerate() {
            if let Some(label) = line.trim().strip_suffix(':') {
                labels.insert(label.to_owned(), number - 1);
            }
        }

        let mut variables = HashMap::new();

        variables.insert(
            "PI".to_owned(),
            (Types::Float(std::f32::consts::PI), VariableType::Const),
        );
        variables.insert(
            "E".to_owned(),
            (Types::Float(std::f32::consts::E), VariableType::Const),
        );

        variables.insert(
            "SQRT_2".to_owned(),
            (Types::Float(std::f32::consts::SQRT_2), VariableType::Const),
        );

        variables.insert(
            "TAU".to_owned(),
            (Types::Float(std::f32::consts::TAU), VariableType::Const),
        );

        Self {
            variables,
            labels,
            file,
        }
    }

    fn echo(&self, line_number: usize, line: &str, expr: &str) {
        if expr.contains('{') && expr.contains('}') {
            let text = expr.chars().collect::<Vec<char>>();

            let mut variable = false;
            let mut expr = String::new();

            for (i, sym) in text.iter().enumerate() {
                if !(*sym != '{' || i != 0 && text[i - 1] == '\\') {
                    assert!(!variable, "{}", get_error(line_number, line, errors::A10));
                    variable = true;
                    continue;
                } else if *sym == '}' && i != 0 && text[i - 1] != '\\' {
                    variable = false;

                    print!(
                        "{}",
                        if let Some((Types::Vector(_), _)) = self.variables.get(&expr) {
                            self.variables.get(&expr).unwrap().0.clone()
                        } else {
                            tokenize(&self.variables, line_number, line, &expr)
                        }
                    );

                    expr.clear();
                    continue;
                }

                if variable {
                    expr.push(*sym);
                } else {
                    print!("{}", sym);
                }
            }

            println!();
        } else {
            println!("{}", expr)
        }
    }

    fn goto(&self, line_number: usize, line: &str, dest: &str) -> usize {
        dest.parse::<usize>().unwrap_or_else(|_| {
            *self
                .labels
                .get(dest.trim())
                .unwrap_or_else(|| show_error(line_number, line, errors::A11))
        })
    }

    fn compare(&self, line_number: usize, line: &str, expr: &str, op: &str) -> Option<bool> {
        if let Some((first, second)) = expr.split_once(op) {
            let first = tokenize(&self.variables, line_number, line, first);
            let second = tokenize(&self.variables, line_number, line, second);

            assert!(
                discriminant(&first) == discriminant(&second),
                "{}",
                get_error(line_number, line, errors::A14),
            );

            Some(match op {
                ">=" => first >= second,
                "<=" => first <= second,
                ">" => first > second,
                "<" => first < second,
                "==" => first == second,
                "!=" => first != second,
                _ => unreachable!(),
            })
        } else {
            None
        }
    }

    fn condition(&self, token: &str, line_number: &mut usize, line: &str) {
        let (condition, action) = token
            .split_once("then")
            .unwrap_or_else(|| show_error(*line_number, line, errors::A12));

        let check = |mut expr: &str| {
            expr = expr.trim();
            let not = expr.starts_with("not");

            if not {
                expr = &expr[3..];
            }

            let ops = [">=", "<=", "<", ">", "==", "!="];

            let result = ops
                .iter()
                .find_map(|op| self.compare(*line_number, line, expr, op))
                .unwrap_or_else(|| {
                    if let Types::Boolean(value) = self.parse_value(expr) {
                        value
                    } else {
                        show_error(*line_number, line, errors::A13);
                    }
                });

            if not { !result } else { result }
        };

        let condition_true = if let Some((first, second)) = condition.split_once("and") {
            check(first) && check(second)
        } else if let Some((first, second)) = condition.split_once("or") {
            check(first) || check(second)
        } else {
            check(condition)
        };

        if condition_true {
            if action.trim().starts_with("echo") {
                self.echo(
                    *line_number,
                    &line,
                    action
                        .trim()
                        .split_once(" ")
                        .unwrap_or_else(|| show_error(*line_number, line, errors::A01))
                        .1
                        .trim(),
                );
            } else {
                *line_number = self.goto(*line_number, line, action);
            }
        } else if *line_number != self.file.len() && self.file[*line_number].starts_with("else if")
        {
            *line_number += 1;

            self.condition(
                self.file[*line_number - 1]
                    .split_once("if")
                    .unwrap()
                    .1
                    .trim(),
                line_number,
                line,
            );
        } else if *line_number != self.file.len() && self.file[*line_number].starts_with("else") {
            let action = self.file[*line_number]
                .split_once(" ")
                .unwrap_or_else(|| show_error(*line_number, line, errors::A01))
                .1
                .trim();

            if action.starts_with("echo") {
                self.echo(
                    *line_number,
                    line,
                    action
                        .split_once(" ")
                        .unwrap_or_else(|| show_error(*line_number, line, errors::A01))
                        .1
                        .trim(),
                );
            } else {
                *line_number = self.goto(*line_number, line, action);
            }
        }
    }

    fn parse_value(&self, expr: &str) -> Types {
        let expr = expr.trim();

        self.variables
            .get(expr)
            .cloned()
            .unwrap_or_else(|| (Types::create(expr), VariableType::Local))
            .0
    }

    fn do_math(&mut self, line_number: usize, line: &str, name: &str, expr: &str, op: &str) {
        let second = tokenize(&self.variables, line_number, line, expr);
        let (value, var_type) = self
            .variables
            .get_mut(name)
            .unwrap_or_else(|| show_error(line_number, line, errors::A03));

        if *var_type != VariableType::Const {
            assert!(
                discriminant(value) == discriminant(&second),
                "{}",
                get_error(line_number, line, errors::A14),
            );

            match op {
                "%=" => *value %= second,
                "/=" => *value /= second,
                "*=" => *value *= second,
                "+=" => *value += second,
                "-=" => *value -= second,
                _ => show_error(line_number, line, errors::A06),
            }
        }
    }

    pub fn run(&mut self) {
        let mut line_number = 0;

        while line_number < self.file.len() {
            let line = self.file[line_number].trim().to_owned();
            line_number += 1;

            if line.starts_with('#') || line.is_empty() || line.ends_with(':') {
                continue;
            }

            let tokens = line
                .split_once(" ")
                .unwrap_or_else(|| show_error(line_number, &line, errors::A01));

            match tokens.0 {
                "echo" => self.echo(line_number, &line, tokens.1),
                "exit" => std::process::exit(
                    tokens
                        .1
                        .parse::<i32>()
                        .unwrap_or_else(|_| show_error(line_number, &line, errors::A02)),
                ),
                "goto" => line_number = self.goto(line_number, &line, tokens.1),
                "if" => self.condition(tokens.1, &mut line_number, &line),
                "delete" => {
                    self.variables
                        .remove(tokens.1.trim())
                        .unwrap_or_else(|| show_error(line_number, &line, errors::A03));
                }
                "var" | "const" => {
                    let (name, value) = tokens
                        .1
                        .split_once('=')
                        .unwrap_or_else(|| show_error(line_number, &line, errors::A04));

                    let name = name.trim();
                    let value = value.trim();

                    assert!(
                        !(name.is_empty()
                            || !name.is_ascii()
                            || name.len() > 256
                            || name.chars().next().unwrap().is_numeric()),
                        "{}",
                        get_error(line_number, &line, errors::A05)
                    );

                    let var_type = match tokens.0 {
                        "var" => VariableType::Local,
                        _ => VariableType::Const,
                    };

                    if value == "input" {
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();

                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();

                        self.variables.insert(
                            name.trim().to_owned(),
                            (Types::String(input.trim().to_owned()), var_type),
                        );
                    } else if value.starts_with("vector") {
                        if let Some((_, capacity)) = value.split_once(' ') {
                            self.variables.insert(
                                name.trim().to_owned(),
                                (
                                    Types::Vector(Vec::with_capacity(
                                        capacity.parse::<usize>().unwrap_or_else(|_| {
                                            show_error(line_number, &line, errors::A02)
                                        }),
                                    )),
                                    var_type,
                                ),
                            );
                        } else {
                            self.variables.insert(
                                name.trim().to_owned(),
                                (Types::Vector(Vec::new()), var_type),
                            );
                        }
                    } else if value.contains(',') {
                        self.variables.insert(
                            name.trim().to_owned(),
                            (
                                Types::Vector(
                                    value
                                        .split(',')
                                        .map(|member| Types::create(member.trim()))
                                        .collect(),
                                ),
                                var_type,
                            ),
                        );
                    } else {
                        self.variables.insert(
                            name.trim().to_owned(),
                            (
                                tokenize(&self.variables, line_number, &line, value),
                                var_type,
                            ),
                        );
                    }
                }
                _ => {
                    if tokens.0.starts_with("else") {
                        continue;
                    }

                    if self.variables.contains_key(tokens.0) {
                        if let Some((op, expr)) = tokens.1.split_once(' ') {
                            let expr: &str = expr.trim();

                            match op.trim() {
                                "as" => {
                                    let (value, var_type) =
                                        self.variables.get_mut(tokens.0).unwrap();

                                    assert!(
                                        *var_type == VariableType::Local,
                                        "{}",
                                        get_error(line_number, &line, errors::A07)
                                    );

                                    if let Types::Vector(_) = value {
                                        match expr {
                                            "numbers" => {
                                                value.convert_to_number(line_number, &line)
                                            }
                                            "strings" => {
                                                value.convert_to_string(line_number, &line)
                                            }
                                            "floats" => value.convert_to_float(line_number, &line),
                                            "bools" => value.convert_to_bool(line_number, &line),
                                            _ => show_error(line_number, &line, errors::A08),
                                        };
                                    } else {
                                        match expr {
                                            "vector" => value.convert_to_vector(),
                                            "number" => value.convert_to_number(line_number, &line),
                                            "string" => value.convert_to_string(line_number, &line),
                                            "float" => value.convert_to_float(line_number, &line),
                                            "bool" => value.convert_to_bool(line_number, &line),
                                            _ => show_error(line_number, &line, errors::A08),
                                        };
                                    }
                                }
                                "%=" => self.do_math(line_number, &line, tokens.0, expr, "%="),
                                "/=" => self.do_math(line_number, &line, tokens.0, expr, "/="),
                                "*=" => self.do_math(line_number, &line, tokens.0, expr, "*="),
                                "+=" => self.do_math(line_number, &line, tokens.0, expr, "+="),
                                "-=" => self.do_math(line_number, &line, tokens.0, expr, "-="),
                                "push" => {
                                    let value = tokenize(&self.variables, line_number, &line, expr);
                                    if let Types::Vector(source) =
                                        &mut self.variables.get_mut(tokens.0).unwrap().0
                                    {
                                        source.push(value)
                                    } else {
                                        show_error(line_number, &line, errors::A09)
                                    }
                                }
                                "remove" => {
                                    if let Types::Vector(source) =
                                        &mut self.variables.get_mut(tokens.0).unwrap().0
                                    {
                                        source.remove(expr.parse::<usize>().unwrap());
                                    } else {
                                        show_error(line_number, &line, errors::A09)
                                    }
                                }
                                _ => show_error(line_number, &line, errors::A06),
                            }
                        } else if let Some((Types::Vector(source), var_type)) =
                            self.variables.get_mut(tokens.0)
                        {
                            assert!(
                                *var_type == VariableType::Local,
                                "{}",
                                get_error(line_number, &line, errors::A07)
                            );
                            match tokens.1.trim() {
                                "unique" => {
                                    let mut unique = Vec::new();

                                    for x in source.iter() {
                                        if !unique.contains(x) {
                                            unique.push(x.clone());
                                        }
                                    }

                                    *source = unique;
                                }
                                _ => show_error(line_number, &line, errors::A06),
                            }
                        }
                    } else if tokens.0.contains('[') {
                        let chars: Vec<char> = tokens.0.chars().collect();
                        let mut i = chars.iter().position(|ch| *ch == '[').unwrap() + 1;
                        let start = i - 1;

                        let mut index = String::new();
                        while i < chars.len() && chars[i] != ']' {
                            index.push(chars[i]);
                            i += 1;
                        }

                        let (op, expr) = tokens
                            .1
                            .split_once(' ')
                            .unwrap_or_else(|| show_error(line_number, &line, errors::A01));

                        let op: &str = op.trim();
                        let expr: &str = expr.trim();

                        let result = tokenize(&self.variables, line_number, &line, expr);
                        let (value, var_type) = self
                            .variables
                            .get_mut(&tokens.0[..start])
                            .unwrap_or_else(|| show_error(line_number, &line, errors::A03));

                        assert!(
                            *var_type == VariableType::Local,
                            "{}",
                            get_error(line_number, &line, errors::A07)
                        );

                        if let Types::Vector(source) = value {
                            let index = index
                                .parse::<usize>()
                                .unwrap_or_else(|_| show_error(line_number, &line, errors::A02));

                            assert!(
                                discriminant(&source[index]) == discriminant(&result),
                                "{}",
                                get_error(line_number, &line, errors::A14),
                            );

                            match op {
                                "=" => source[index] = result,
                                "%=" => source[index] %= result,
                                "*=" => source[index] *= result,
                                "/=" => source[index] /= result,
                                "+=" => source[index] += result,
                                "-=" => source[index] -= result,
                                _ => show_error(line_number, &line, errors::A06),
                            }
                        }
                    } else {
                        show_error(line_number, &line, errors::A06);
                    }
                }
            }
        }
    }
}
