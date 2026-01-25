// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{errors, get_error, show_error, show_warning, tokenizer::tokenize, types::Types};
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
    do_condition: bool,
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
            do_condition: false,
            variables,
            labels,
            file,
        }
    }

    fn echo(&self, expr: &str) -> Result<(), String> {
        let expr = expr
            .strip_prefix("\"")
            .ok_or(errors::A15)?
            .strip_suffix("\"")
            .ok_or(errors::A15)?;

        if expr.contains('{') && expr.contains('}') {
            for line in expr.split("\\n") {
                let text = line.chars().collect::<Vec<char>>();

                let mut variable = false;
                let mut expr = String::new();

                for (i, sym) in text.iter().enumerate() {
                    if !(*sym != '{' || i != 0 && text[i - 1] == '\\') {
                        if variable {
                            return Err(errors::A10.to_owned());
                        }

                        variable = true;
                        continue;
                    } else if *sym == '}' && i != 0 && text[i - 1] != '\\' {
                        variable = false;

                        print!(
                            "{}",
                            if let Some((Types::Vector(_), _)) = self.variables.get(&expr) {
                                self.variables.get(&expr).unwrap().0.clone()
                            } else {
                                tokenize(&self.variables, &expr)?
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
            }
        } else {
            for line in expr.split("\\n") {
                println!("{}", line);
            }
        }

        Ok(())
    }

    fn goto(&self, dest: &str) -> Result<usize, String> {
        if let Ok(line) = dest.parse::<usize>() {
            Ok(line)
        } else {
            Ok(self.labels.get(dest.trim()).cloned().ok_or(errors::A11)?)
        }
    }

    fn compare(&self, expr: &str, op: &str) -> Result<Option<bool>, String> {
        if let Some((first, second)) = expr.split_once(op) {
            let first = tokenize(&self.variables, first)?;
            let second = tokenize(&self.variables, second)?;

            if discriminant(&first) != discriminant(&second) {
                return Err(errors::A14.to_owned());
            }

            Ok(Some(match op {
                ">=" => first >= second,
                "<=" => first <= second,
                ">" => first > second,
                "<" => first < second,
                "==" => first == second,
                "!=" => first != second,
                _ => unreachable!(),
            }))
        } else {
            Ok(None)
        }
    }

    fn condition(&self, token: &str, line_number: &mut usize) -> Result<bool, String> {
        let (condition, _) = token.split_once("then").ok_or(errors::A12)?;

        let check = |mut expr: &str| {
            expr = expr.trim();
            let not = expr.starts_with("not");

            if not {
                expr = &expr[3..];
            }

            let ops = [">=", "<=", "<", ">", "==", "!="];
            let mut result = None;

            for op in &ops {
                match self.compare(expr, op) {
                    Ok(Some(value)) => {
                        result = Some(value);
                        break;
                    }
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            }

            let result = match result {
                Some(v) => v,
                None => {
                    if let Types::Boolean(value) = self.parse_value(expr) {
                        value
                    } else {
                        return Err(errors::A13.to_owned());
                    }
                }
            };

            if not { Ok(!result) } else { Ok(result) }
        };

        let condition_true = if let Some((first, second)) = condition.split_once("and") {
            check(first)? && check(second)?
        } else if let Some((first, second)) = condition.split_once("or") {
            check(first)? || check(second)?
        } else {
            check(condition)?
        };

        if condition_true {
            Ok(true)
        } else {
            if self.file[*line_number - 1].starts_with("else") {
                *line_number += 1;
            }

            while !self.file[*line_number - 1].starts_with("else")
                && !self.file[*line_number - 1].starts_with("endif")
            {
                *line_number += 1;
            }

            if self.file[*line_number - 1].starts_with("endif")
                || !self.file[*line_number - 1].contains("if")
            {
                Ok(false)
            } else if let Some((_, token)) = self.file[*line_number - 1].split_once("if") {
                self.condition(token.trim(), line_number)
            } else {
                Err(errors::A13.to_owned())
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

    fn do_math(&mut self, name: &str, expr: &str, op: &str) -> Result<(), String> {
        let second = tokenize(&self.variables, expr)?;
        let (value, var_type) = self.variables.get_mut(name).ok_or(errors::A03)?;

        if *var_type != VariableType::Const {
            if !(discriminant(value) == discriminant(&second)
                || (op == "*="
                    && matches!(
                        (&value, &second),
                        (Types::String(_), Types::Number(_)) | (Types::Number(_), Types::String(_))
                    )))
            {
                return Err(errors::A14.to_owned());
            }

            match op {
                "%=" => value.rem_assign(second)?,
                "/=" => value.div_assign(second)?,
                "*=" => value.mul_assign(second)?,
                "+=" => value.add_assign(second)?,
                "-=" => value.sub_assign(second)?,
                "=" => *value = second,
                _ => return Err(errors::A06.to_owned()),
            }
        }

        Ok(())
    }

    pub fn split_string(string: &mut Types, pattern: &str) -> Result<(), ()> {
        if let Types::String(source) = string {
            *string = Types::Vector(source.split(pattern).map(Types::create).collect());
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn run(&mut self) {
        let mut line_number = 0;

        while line_number < self.file.len() {
            let line = self.file[line_number].trim().to_owned();
            line_number += 1;

            if line.starts_with('#') || line.is_empty() || line.ends_with(':') || line == "endif" {
                continue;
            }

            let tokens = line
                .split_once(" ")
                .unwrap_or_else(|| show_error(line_number, &line, errors::A01));

            match tokens.0 {
                "echo" => self
                    .echo(tokens.1)
                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                "exit" => std::process::exit(
                    tokens
                        .1
                        .parse::<i32>()
                        .unwrap_or_else(|_| show_error(line_number, &line, errors::A02)),
                ),
                "goto" => {
                    line_number = self
                        .goto(tokens.1)
                        .unwrap_or_else(|e| show_error(line_number, &line, &e))
                }
                "if" => {
                    self.do_condition = self
                        .condition(tokens.1, &mut line_number)
                        .unwrap_or_else(|e| show_error(line_number, &line, &e))
                }
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
                                tokenize(&self.variables, value)
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                var_type,
                            ),
                        );
                    }
                }
                _ => {
                    if tokens.0.starts_with("else") {
                        if self.do_condition {
                            while !self.file[line_number - 1].starts_with("endif") {
                                line_number += 1;
                            }
                        }

                        continue;
                    }

                    if self.variables.contains_key(tokens.0) {
                        if let Some((op, expr)) = tokens.1.split_once(' ') {
                            let expr: &str = expr.trim();

                            assert!(
                                self.variables.get(tokens.0).unwrap().1 == VariableType::Local,
                                "{}",
                                get_error(line_number, &line, errors::A07)
                            );

                            match op.trim() {
                                "as" => {
                                    let (value, _) = self.variables.get_mut(tokens.0).unwrap();

                                    let result = if let Types::Vector(_) = value {
                                        match expr {
                                            "numbers" => value.convert_to_number(),
                                            "strings" => value.convert_to_string(),
                                            "floats" => value.convert_to_float(),
                                            "bools" => value.convert_to_bool(),
                                            _ => show_error(line_number, &line, errors::A08),
                                        }
                                    } else {
                                        match expr {
                                            "vector" => value.convert_to_vector().map(|_| None),
                                            "number" => value.convert_to_number(),
                                            "string" => value.convert_to_string(),
                                            "float" => value.convert_to_float(),
                                            "bool" => value.convert_to_bool(),
                                            _ => show_error(line_number, &line, errors::A08),
                                        }
                                    };

                                    match result {
                                        Ok(Some(warning)) => {
                                            show_warning(line_number, &line, &warning)
                                        }
                                        Err(error) => show_error(line_number, &line, &error),
                                        Ok(None) => {}
                                    };
                                }
                                "%=" => self
                                    .do_math(tokens.0, expr, "%=")
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "/=" => self
                                    .do_math(tokens.0, expr, "/=")
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "*=" => self
                                    .do_math(tokens.0, expr, "*=")
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "+=" => self
                                    .do_math(tokens.0, expr, "+=")
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "-=" => self
                                    .do_math(tokens.0, expr, "-=")
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "=" => self
                                    .do_math(tokens.0, expr, "=")
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "push" => {
                                    let value = tokenize(&self.variables, expr)
                                        .unwrap_or_else(|e| show_error(line_number, &line, &e));
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
                                "split" => {
                                    let pattern = match tokenize(&self.variables, expr)
                                        .unwrap_or_else(|e| show_error(line_number, &line, &e))
                                    {
                                        Types::String(pattern) => pattern,
                                        _ => show_error(line_number, &line, errors::A18),
                                    };

                                    Self::split_string(
                                        &mut self.variables.get_mut(tokens.0).unwrap().0,
                                        &pattern,
                                    )
                                    .unwrap_or_else(|_| show_error(line_number, &line, errors::A18))
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

                        let result = tokenize(&self.variables, expr)
                            .unwrap_or_else(|e| show_error(line_number, &line, &e));

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
                                "%=" => source[index]
                                    .rem_assign(result)
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "*=" => source[index]
                                    .mul_assign(result)
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "/=" => source[index]
                                    .div_assign(result)
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "+=" => source[index]
                                    .add_assign(result)
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
                                "-=" => source[index]
                                    .sub_assign(result)
                                    .unwrap_or_else(|e| show_error(line_number, &line, &e)),
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
