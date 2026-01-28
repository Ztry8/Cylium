// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{errors, file_handler::FileHandler, types::Types};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Value(Types),

    Call,     // call
    As,       // as
    While,    // while
    EndWhile, // endwhile
    Var,      // var
    Const,    // const
    If,       // if
    Else,     // else
    EndIf,    // endif
    Echo,     // echo
    Exit,     // exit
    Delete,   // delete
    Proc,     // proc
    End,

    Plus,           // +
    Minus,          // -
    Multiply,       // *
    Divide,         // /
    Mod,            // %
    Assign,         // =
    PlusAssign,     // +=
    MinusAssign,    // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=
    ModAssign,      // %=

    Equal,        // ==
    NotEqual,     // !=
    Greater,      // >
    Less,         // <
    GreaterEqual, // >=
    LessEqual,    // <=
    And,          // and
    Or,           // or
    Not,          // not

    OpenParen,    // (
    CloseParen,   // )
    OpenBracket,  // [
    CloseBracket, // ]
    OpenBrace,    // {
    CloseBrace,   // }
    Comma,        // ,
}

pub fn tokenize_file(handler: &FileHandler) -> Vec<Vec<Token>> {
    let mut tokens = Vec::new();

    for (line_number, line) in handler.ready_file.iter().enumerate() {
        let line = line.trim();

        tokens.push(match tokenize_line(line) {
            Ok(value) => value,
            Err(e) => handler.show_error(line_number, &e),
        });
    }

    tokens
}

fn check(chars: &[char], i: &mut usize, single: Token, double: Token) -> Token {
    if *i + 1 < chars.len() && chars[*i + 1] == '=' {
        *i += 1;
        double
    } else {
        single
    }
}

fn tokenize_line(line: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' => {
                i += 1;
                continue;
            }

            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            '[' => tokens.push(Token::OpenBracket),
            ']' => tokens.push(Token::CloseBracket),
            '{' => tokens.push(Token::OpenBrace),
            '}' => tokens.push(Token::CloseBrace),
            ',' => tokens.push(Token::Comma),

            '+' => tokens.push(check(&chars, &mut i, Token::Plus, Token::PlusAssign)),
            '-' => tokens.push(check(&chars, &mut i, Token::Minus, Token::MinusAssign)),
            '/' => tokens.push(check(&chars, &mut i, Token::Divide, Token::DivideAssign)),
            '%' => tokens.push(check(&chars, &mut i, Token::Mod, Token::ModAssign)),
            '=' => tokens.push(check(&chars, &mut i, Token::Assign, Token::Equal)),
            '>' => tokens.push(check(&chars, &mut i, Token::Greater, Token::GreaterEqual)),
            '<' => tokens.push(check(&chars, &mut i, Token::Less, Token::LessEqual)),

            '*' => tokens.push(check(
                &chars,
                &mut i,
                Token::Multiply,
                Token::MultiplyAssign,
            )),

            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::NotEqual);
                    i += 1;
                } else {
                    return Err(errors::A15.to_owned());
                }
            }

            '"' => {
                i += 1;
                let mut string = String::new();
                while i < chars.len() && !(i > 0 && chars[i - 1] != '\\' && chars[i] == '"') {
                    string.push(chars[i]);
                    i += 1;
                }

                if i >= chars.len() {
                    return Err(errors::A21.to_owned());
                }

                tokens.push(Token::Value(Types::String(string)))
            }

            _ => {
                if chars[i].is_ascii_digit() {
                    let mut num = String::new();
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        num.push(chars[i]);
                        i += 1;
                    }

                    i -= 1;
                    tokens.push(Token::Value(Types::create(&num)));
                } else if chars[i].is_alphabetic() || chars[i] == '_' {
                    let mut ident = String::new();
                    while i < chars.len()
                        && (chars[i].is_alphanumeric()
                            || chars[i] == '_'
                            || chars[i] == '['
                            || chars[i] == ']')
                    {
                        ident.push(chars[i]);
                        i += 1;
                    }

                    i -= 1;
                    match ident.as_str() {
                        "true" => tokens.push(Token::Value(Types::Boolean(true))),
                        "false" => tokens.push(Token::Value(Types::Boolean(false))),
                        "call" => tokens.push(Token::Call),
                        "as" => tokens.push(Token::As),
                        "var" => tokens.push(Token::Var),
                        "const" => tokens.push(Token::Const),
                        "if" => tokens.push(Token::If),
                        "else" => tokens.push(Token::Else),
                        "endif" => tokens.push(Token::EndIf),
                        "endwhile" => tokens.push(Token::EndWhile),
                        "echo" => tokens.push(Token::Echo),
                        "exit" => tokens.push(Token::Exit),
                        "delete" => tokens.push(Token::Delete),
                        "proc" => tokens.push(Token::Proc),
                        "end" => tokens.push(Token::End),
                        "while" => tokens.push(Token::While),
                        "not" => tokens.push(Token::Not),
                        "and" => tokens.push(Token::And),
                        "or" => tokens.push(Token::Or),
                        _ => tokens.push(Token::Ident(ident)),
                    }
                } else {
                    return Err(errors::A15.to_owned());
                }
            }
        }

        i += 1;
    }

    Ok(tokens)
}
