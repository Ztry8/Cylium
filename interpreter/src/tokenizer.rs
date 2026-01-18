// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{errors, get_error, interpreter::Variable, show_error, types::Types};
use std::collections::HashMap;

macro_rules! is_string {
    ($sym:expr) => {
        $sym.is_alphabetic() || $sym.is_ascii_punctuation() || $sym.is_ascii_whitespace()
    };
}

#[derive(PartialEq, Clone)]
pub enum Token {
    OpenParen,  // (
    CloseParen, // )
    Value(Types),
    Plus,
    Minus,
    Multiply,
    Divide,
    Mod,
}

fn get_value(token: Token) -> Types {
    match token {
        Token::Value(value) => value,
        _ => panic!(),
    }
}

fn to_token(value: Types) -> Token {
    Token::Value(value)
}

fn compute(mut tokens: Vec<Token>, line_number: usize, line: &str) -> Token {
    assert!(
        !tokens.is_empty(),
        "{}",
        get_error(line_number, line, errors::A15)
    );

    assert!(
        !(tokens
            .iter()
            .any(|t| matches!(t, Token::Value(Types::Number(_))))
            && tokens
                .iter()
                .any(|t| matches!(t, Token::Value(Types::Float(_))))
            && tokens
                .iter()
                .any(|t| matches!(t, Token::Value(Types::String(_))))),
        "{}",
        get_error(line_number, line, errors::A14)
    );

    while let Some(start) = tokens.iter().position(|t| *t == Token::OpenParen) {
        let mut depth = 1;
        let mut end = start + 1;

        while end < tokens.len() && depth > 0 {
            match tokens[end] {
                Token::OpenParen => depth += 1,
                Token::CloseParen => depth -= 1,
                _ => {}
            }
            end += 1;
        }

        assert!(depth == 0, "{}", get_error(line_number, line, errors::A10));
        let expr = tokens.drain(start + 1..end - 1).collect();

        tokens[start] = compute(expr, line_number, line);
        tokens.remove(start + 1);
    }

    while tokens.len() != 1 {
        if let Some(index) = tokens
            .iter()
            .position(|t| matches!(*t, Token::Multiply | Token::Divide | Token::Mod))
        {
            tokens[index] = to_token(match tokens[index] {
                Token::Multiply => {
                    get_value(tokens[index - 1].clone()) * get_value(tokens[index + 1].clone())
                }
                Token::Divide => {
                    get_value(tokens[index - 1].clone()) / get_value(tokens[index + 1].clone())
                }
                Token::Mod => {
                    get_value(tokens[index - 1].clone()) % get_value(tokens[index + 1].clone())
                }
                _ => unreachable!(),
            });

            tokens.remove(index + 1);
            tokens.remove(index - 1);
        } else if let Some(index) = tokens
            .iter()
            .position(|t| matches!(*t, Token::Plus | Token::Minus))
        {
            tokens[index] = to_token(match tokens[index] {
                Token::Plus => {
                    get_value(tokens[index - 1].clone()) + get_value(tokens[index + 1].clone())
                }
                Token::Minus => {
                    get_value(tokens[index - 1].clone()) - get_value(tokens[index + 1].clone())
                }
                _ => unreachable!(),
            });

            tokens.remove(index + 1);
            tokens.remove(index - 1);
        }
    }

    tokens[0].clone()
}

pub fn tokenize(
    variables: &HashMap<String, Variable>,
    line_number: usize,
    line: &str,
    expr: &str,
) -> Types {
    let mut tokens = Vec::new();
    let chars = expr.chars().collect::<Vec<char>>();
    let mut i = 0;

    while i < chars.len() {
        let sym = chars[i];

        if sym == ' ' {
            i += 1;
            continue;
        }

        tokens.push(match sym {
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Multiply,
            '/' => Token::Divide,
            '%' => Token::Mod,
            _ => {
                let mut token = String::new();
                let mut j = 0;

                if sym.is_ascii_digit() {
                    while i + j < chars.len()
                        && (chars[i + j].is_ascii_digit() || chars[i + j] == '.')
                    {
                        token.push(chars[i + j]);
                        j += 1;
                    }

                    i += j - 1;
                    if let Ok(value) = token.parse::<i32>() {
                        Token::Value(Types::Number(value))
                    } else {
                        Token::Value(Types::Float(
                            token
                                .parse::<f32>()
                                .unwrap_or_else(|_| show_error(line_number, line, errors::A02)),
                        ))
                    }
                } else if is_string!(sym) {
                    while i + j < chars.len() && (is_string!(chars[i + j])) {
                        token.push(chars[i + j]);
                        j += 1;
                    }

                    i += j - 1;

                    let mut index = String::new();
                    if i + 1 < chars.len() && chars[i + 1] == '[' {
                        j = 2;

                        while i + j < chars.len() && chars[i + j] != ']' {
                            index.push(chars[i + j]);
                            j += 1;
                        }

                        i += j;
                    }

                    if let Some((value, _)) = &variables.get(&token) {
                        if let Types::Vector(source) = value {
                            if let Ok(index) = index.parse::<usize>() {
                                match &source[index] {
                                    Types::Number(value) => Token::Value(Types::Number(*value)),
                                    Types::Float(value) => Token::Value(Types::Float(*value)),
                                    Types::String(value) => {
                                        Token::Value(Types::String(value.clone()))
                                    }
                                    _ => show_error(line_number, line, errors::A16),
                                }
                            } else if let Some((Types::Number(index), _)) = variables.get(&index) {
                                let index = *index as usize;
                                match &source[index] {
                                    Types::Number(value) => Token::Value(Types::Number(*value)),
                                    Types::Float(value) => Token::Value(Types::Float(*value)),
                                    Types::String(value) => {
                                        Token::Value(Types::String(value.clone()))
                                    }
                                    _ => show_error(line_number, line, errors::A16),
                                }
                            } else if index == "length" {
                                Token::Value(Types::Number(source.len() as i32))
                            } else if index == "capacity" {
                                Token::Value(Types::Number(source.capacity() as i32))
                            } else {
                                show_error(line_number, line, errors::A17)
                            }
                        } else {
                            Token::Value(value.clone())
                        }
                    } else {
                        Token::Value(Types::String(token.trim().to_owned()))
                    }
                } else {
                    show_error(line_number, line, errors::A15);
                }
            }
        });

        i += 1;
    }

    match compute(tokens, line_number, line) {
        Token::Value(value) => value,
        _ => unreachable!(),
    }
}
