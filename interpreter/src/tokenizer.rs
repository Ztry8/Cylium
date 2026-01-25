// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{errors, interpreter::Variable, types::Types};
use std::collections::HashMap;

macro_rules! is_string {
    ($sym:expr) => {
        $sym.is_alphabetic() || $sym.is_ascii_punctuation()
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

fn compute(mut tokens: Vec<Token>) -> Result<Token, String> {
    if tokens.is_empty() {
        return Err(errors::A15.to_owned());
    }

    if tokens
        .iter()
        .any(|t| matches!(t, Token::Value(Types::Number(_))))
        && tokens
            .iter()
            .any(|t| matches!(t, Token::Value(Types::Float(_))))
        && tokens
            .iter()
            .any(|t| matches!(t, Token::Value(Types::String(_))))
    {
        return Err(errors::A14.to_owned());
    }

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

        if depth != 0 {
            return Err(errors::A14.to_owned());
        }

        let expr = tokens.drain(start + 1..end - 1).collect();

        tokens[start] = compute(expr)?;
        tokens.remove(start + 1);
    }

    while tokens.len() != 1 {
        if let Some(index) = tokens
            .iter()
            .position(|t| matches!(*t, Token::Multiply | Token::Divide | Token::Mod))
        {
            tokens[index] = to_token(match tokens[index] {
                Token::Multiply => get_value(tokens[index - 1].clone())
                    .mul(get_value(tokens[index + 1].clone()))?,
                Token::Divide => get_value(tokens[index - 1].clone())
                    .div(get_value(tokens[index + 1].clone()))?,
                Token::Mod => get_value(tokens[index - 1].clone())
                    .rem(get_value(tokens[index + 1].clone()))?,
                _ => unreachable!(),
            });

            tokens.remove(index + 1);
            tokens.remove(index - 1);
        } else if let Some(index) = tokens
            .iter()
            .position(|t| matches!(*t, Token::Plus | Token::Minus))
        {
            tokens[index] = to_token(match tokens[index] {
                Token::Plus => get_value(tokens[index - 1].clone())
                    .add(get_value(tokens[index + 1].clone()))?,
                Token::Minus => get_value(tokens[index - 1].clone())
                    .sub(get_value(tokens[index + 1].clone()))?,
                _ => unreachable!(),
            });

            tokens.remove(index + 1);
            tokens.remove(index - 1);
        }
    }

    Ok(tokens[0].clone())
}

pub fn tokenize(variables: &HashMap<String, Variable>, expr: &str) -> Result<Types, String> {
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
                        Token::Value(Types::Float(token.parse::<f32>().map_err(|_| errors::A02)?))
                    }
                } else if sym == '\"' {
                    j = 1;
                    while i + j < chars.len() && chars[i + j] != '\"' {
                        token.push(chars[i + j]);
                        j += 1;
                    }

                    i += j;
                    Token::Value(Types::String(token.to_owned()))
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
                                    _ => return Err(errors::A16.to_owned()),
                                }
                            } else if let Some((Types::Number(index), _)) = variables.get(&index) {
                                let index = *index as usize;
                                match &source[index] {
                                    Types::Number(value) => Token::Value(Types::Number(*value)),
                                    Types::Float(value) => Token::Value(Types::Float(*value)),
                                    Types::String(value) => {
                                        Token::Value(Types::String(value.clone()))
                                    }
                                    _ => return Err(errors::A16.to_owned()),
                                }
                            } else if index == "length" {
                                Token::Value(Types::Number(source.len() as i32))
                            } else if index == "capacity" {
                                Token::Value(Types::Number(source.capacity() as i32))
                            } else {
                                return Err(errors::A17.to_owned());
                            }
                        } else {
                            Token::Value(value.clone())
                        }
                    } else {
                        Token::Value(Types::String(token.trim().to_owned()))
                    }
                } else {
                    return Err(errors::A15.to_owned());
                }
            }
        });

        i += 1;
    }

    match compute(tokens) {
        Ok(Token::Value(value)) => Ok(value),
        Err(e) => Err(e),
        _ => unreachable!(),
    }
}
