// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{errors, file_handler::FileHandler};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    NumberValue(i64),
    FloatValue(f64),
    StringValue(String),
    BooleanValue(bool),

    As,     // as
    While,  // while
    For,    // for
    From,   // from
    To,     // to
    Step,   // step
    Number, // number
    Float,  // float
    String, // string
    Void,   // void
    Bool,   // bool
    Const,  // const
    If,     // if
    Else,   // else
    Echo,   // echo
    Exit,   // exit
    Delete, // delete
    Func,   // func
    End,    // end
    Return, // return
    Arrow,  // ->

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

    BitAnd,   // &
    BitOr,    // |
    BitXor,   // ^
    BitNot,   // ~
    BitRight, // >>
    BitLeft,  // <<

    BitAndAssign,   // &=
    BitOrAssign,    // |=
    BitXorAssign,   // ^=
    BitNotAssign,   // ~=
    BitRightAssign, // >>=
    BitLeftAssign,  // <<=

    OpenParen,    // (
    CloseParen,   // )
    OpenBracket,  // [
    CloseBracket, // ]
    OpenBrace,    // {
    CloseBrace,   // }
    Comma,        // ,
    Colon,        // :
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

fn check(
    chars: &[char],
    i: &mut usize,
    single: Token,
    double: Token,
    triple: Option<(Token, Token)>,
) -> Token {
    if *i + 2 < chars.len()
        && chars[*i + 1] == chars[*i]
        && chars[*i + 2] == '='
        && let Some((triple_assign, _)) = triple.clone()
    {
        *i += 2;
        triple_assign
    } else if *i + 1 < chars.len()
        && chars[*i + 1] == chars[*i]
        && let Some((_, triple_op)) = triple
    {
        *i += 1;
        triple_op
    } else if *i + 1 < chars.len() && chars[*i + 1] == '=' {
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
            ':' => tokens.push(Token::Colon),

            '+' => tokens.push(check(&chars, &mut i, Token::Plus, Token::PlusAssign, None)),
            '-' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    i += 1;
                    tokens.push(Token::Arrow);
                } else if i + 1 < chars.len() && chars[i + 1] == '=' {
                    i += 1;
                    tokens.push(Token::MinusAssign);
                } else {
                    tokens.push(Token::Minus);
                }
            }

            '/' => tokens.push(check(
                &chars,
                &mut i,
                Token::Divide,
                Token::DivideAssign,
                None,
            )),

            '%' => tokens.push(check(&chars, &mut i, Token::Mod, Token::ModAssign, None)),
            '=' => tokens.push(check(&chars, &mut i, Token::Assign, Token::Equal, None)),

            '&' => tokens.push(check(
                &chars,
                &mut i,
                Token::BitAnd,
                Token::BitAndAssign,
                None,
            )),

            '|' => tokens.push(check(
                &chars,
                &mut i,
                Token::BitOr,
                Token::BitOrAssign,
                None,
            )),

            '^' => tokens.push(check(
                &chars,
                &mut i,
                Token::BitXor,
                Token::BitXorAssign,
                None,
            )),

            '~' => tokens.push(check(
                &chars,
                &mut i,
                Token::BitNot,
                Token::BitNotAssign,
                None,
            )),

            '>' => tokens.push(check(
                &chars,
                &mut i,
                Token::Greater,
                Token::GreaterEqual,
                Some((Token::BitRightAssign, Token::BitRight)),
            )),

            '<' => tokens.push(check(
                &chars,
                &mut i,
                Token::Less,
                Token::LessEqual,
                Some((Token::BitLeftAssign, Token::BitLeft)),
            )),

            '*' => tokens.push(check(
                &chars,
                &mut i,
                Token::Multiply,
                Token::MultiplyAssign,
                None,
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
                let mut escaped = false;
                while i < chars.len() {
                    let c = chars[i];
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        break;
                    }
                    string.push(c);
                    i += 1;
                }

                if i >= chars.len() {
                    return Err(errors::A21.to_owned());
                }

                tokens.push(Token::StringValue(string))
            }

            _ => {
                if chars[i].is_ascii_digit() {
                    let mut num = String::new();
                    while i < chars.len()
                        && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == 'b')
                    {
                        num.push(chars[i]);
                        i += 1;
                    }

                    i -= 1;

                    if let Ok(number) = num.parse::<i64>() {
                        tokens.push(Token::NumberValue(number));
                    } else if let Ok(float) = num.parse::<f64>() {
                        tokens.push(Token::FloatValue(float));
                    } else if let Some(binary_str) = num.strip_prefix("0b")
                        && let Some(binary_num) = i64::from_str_radix(binary_str, 2).ok()
                    {
                        tokens.push(Token::NumberValue(binary_num));
                    } else {
                        return Err(errors::A34.to_owned());
                    }
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
                        "true" => tokens.push(Token::BooleanValue(true)),
                        "false" => tokens.push(Token::BooleanValue(false)),
                        "as" => tokens.push(Token::As),
                        "number" => tokens.push(Token::Number),
                        "float" => tokens.push(Token::Float),
                        "string" => tokens.push(Token::String),
                        "bool" => tokens.push(Token::Bool),
                        "const" => tokens.push(Token::Const),
                        "if" => tokens.push(Token::If),
                        "else" => tokens.push(Token::Else),
                        "for" => tokens.push(Token::For),
                        "from" => tokens.push(Token::From),
                        "to" => tokens.push(Token::To),
                        "step" => tokens.push(Token::Step),
                        "echo" => tokens.push(Token::Echo),
                        "exit" => tokens.push(Token::Exit),
                        "delete" => tokens.push(Token::Delete),
                        "func" => tokens.push(Token::Func),
                        "return" => tokens.push(Token::Return),
                        "void" => tokens.push(Token::Void),
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
