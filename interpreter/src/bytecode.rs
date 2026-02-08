// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use crate::{
    errors,
    file_handler::FileHandler,
    lexer::Token,
    parser::{AstKind, AstNode, Cast, ElseBlock},
    types::{Types, TypesCheck},
};

#[derive(Debug)]
pub struct Node {
    pub instruction: Instruction,
    pub line_number: usize,
}

#[derive(Debug)]
pub struct Proc {
    pub args: Vec<(String, TypesCheck)>,
    pub body: Vec<Node>,
}

#[derive(Debug)]
pub enum Instruction {
    StoreConst(String),
    StoreLocal(String),
    Push(Types),
    Load(String),
    Neg,
    Not,
    Or,
    And,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Plus,
    Minus,
    Multiply,
    Divide,
    Mod,
    CastToString,
    CastToFloat,
    CastToNumber,
    CastToBoolean,
    Sin,
    Cos,
    Sqrt,

    Echo,
    Exit(i32),
    Delete(String),
    Call(String, usize),

    Jump(usize),
    JumpIfFalse(usize),
}

pub fn compile(handler: &FileHandler, ast: Vec<AstNode>) -> (HashMap<String, Proc>, Vec<Node>) {
    let mut const_code = Vec::new();
    let mut procs = HashMap::new();

    for node in ast {
        match node.kind {
            AstKind::VarDecl {
                name,
                value,
                is_const,
                ..
            } => match var_compile(name, *value, is_const) {
                Ok(var) => const_code.extend(var.into_iter().map(|inst| Node {
                    instruction: inst,
                    line_number: node.line,
                })),
                Err(e) => handler.show_error(node.line, &e),
            },
            AstKind::Proc { name, body, args } => {
                let mut proc_body = Vec::new();

                for node in body {
                    let mut temp = Vec::new();

                    if let Err(e) = main_compile(node.kind, &mut temp) {
                        handler.show_error(node.line, &e);
                    };

                    proc_body.extend(temp.into_iter().map(|inst| Node {
                        instruction: inst,
                        line_number: node.line,
                    }))
                }

                procs.insert(
                    name,
                    Proc {
                        args,
                        body: proc_body,
                    },
                );
            }
            _ => unreachable!(),
        }
    }

    (procs, const_code)
}

fn main_compile(node: AstKind, instructions: &mut Vec<Instruction>) -> Result<(), String> {
    match node {
        AstKind::Echo(expr) => {
            expr_compile(instructions, *expr)?;
            instructions.push(Instruction::Echo)
        }
        AstKind::Delete(name) => {
            instructions.push(Instruction::Delete(name));
        }
        AstKind::Exit(code) => {
            instructions.push(Instruction::Exit(code));
        }
        AstKind::VarDecl {
            name,
            value,
            is_const,
            ..
        } => {
            instructions.extend(var_compile(name, *value, is_const)?);
        }
        AstKind::ProcCall { name, args } => {
            for arg in args.iter().rev() {
                match arg {
                    Token::Ident(name) => instructions.push(Instruction::Load(name.clone())),
                    Token::NumberValue(value) => {
                        instructions.push(Instruction::Push(Types::Number(*value)))
                    }
                    Token::FloatValue(value) => {
                        instructions.push(Instruction::Push(Types::Float(*value)))
                    }
                    Token::StringValue(value) => {
                        instructions.push(Instruction::Push(Types::String(value.clone())))
                    }
                    Token::BooleanValue(value) => {
                        instructions.push(Instruction::Push(Types::Boolean(*value)))
                    }
                    _ => unreachable!(),
                }
            }

            instructions.push(Instruction::Call(name, args.len()));
        }
        AstKind::Assign { name, op, expr } => {
            instructions.push(Instruction::Load(name.clone()));
            expr_compile(instructions, *expr)?;

            let mut make = |inst| {
                instructions.push(inst);
                instructions.push(Instruction::StoreLocal(name.clone()));
            };

            match op {
                Token::Assign => {
                    instructions.push(Instruction::StoreLocal(name));
                }
                Token::PlusAssign => {
                    make(Instruction::Plus);
                }
                Token::MinusAssign => {
                    make(Instruction::Minus);
                }
                Token::MultiplyAssign => {
                    make(Instruction::Multiply);
                }
                Token::DivideAssign => {
                    make(Instruction::Divide);
                }
                Token::ModAssign => {
                    make(Instruction::Mod);
                }
                _ => unreachable!(),
            };
        }
        AstKind::Condition { expr, yes, no } => {
            condition_compile(instructions, *expr, yes, no)?;
        }

        AstKind::While { expr, body } => {
            while_compile(instructions, *expr, body)?;
        }

        AstKind::For {
            var_name,
            start,
            end,
            step,
            body,
        } => {
            for_compile(instructions, var_name, *start, *end, *step, body)?;
        }
        _ => unreachable!(),
    };

    Ok(())
}

fn condition_compile(
    instructions: &mut Vec<Instruction>,
    expr: AstKind,
    yes: Vec<AstNode>,
    no: Option<ElseBlock>,
) -> Result<(), String> {
    todo!()
}

fn while_compile(
    instructions: &mut Vec<Instruction>,
    expr: AstKind,
    body: Vec<AstNode>,
) -> Result<(), String> {
    todo!()
}

fn for_compile(
    instructions: &mut Vec<Instruction>,
    var_name: String,
    start: AstKind,
    end: AstKind,
    step: Option<AstKind>,
    body: Vec<AstNode>,
) -> Result<(), String> {
    todo!()
}

fn expr_compile(instructions: &mut Vec<Instruction>, value: AstKind) -> Result<(), String> {
    match value {
        AstKind::UnaryOp { op, expr } => {
            expr_compile(instructions, *expr)?;

            match op {
                Token::Not => instructions.push(Instruction::Not),
                Token::Minus => instructions.push(Instruction::Neg),
                _ => return Err(errors::A15.to_owned()),
            };
        }

        AstKind::AsOp { expr, op } => {
            expr_compile(instructions, *expr)?;

            match op {
                Cast::String => instructions.push(Instruction::CastToString),
                Cast::Number => instructions.push(Instruction::CastToNumber),
                Cast::Float => instructions.push(Instruction::CastToFloat),
                Cast::Boolean => instructions.push(Instruction::CastToBoolean),
                Cast::Sqrt => instructions.push(Instruction::Sqrt),
                Cast::Sin => instructions.push(Instruction::Sin),
                Cast::Cos => instructions.push(Instruction::Cos),
            }
        }

        AstKind::BinaryOp { left, op, right } => {
            expr_compile(instructions, *right)?;
            expr_compile(instructions, *left)?;

            let inst = match op {
                Token::Or => Instruction::Or,
                Token::And => Instruction::And,
                Token::Equal => Instruction::Equal,
                Token::NotEqual => Instruction::NotEqual,
                Token::Greater => Instruction::Greater,
                Token::Less => Instruction::Less,
                Token::GreaterEqual => Instruction::GreaterEqual,
                Token::LessEqual => Instruction::LessEqual,
                Token::Plus => Instruction::Plus,
                Token::Minus => Instruction::Minus,
                Token::Multiply => Instruction::Multiply,
                Token::Divide => Instruction::Divide,
                Token::Mod => Instruction::Mod,
                _ => unreachable!(),
            };

            instructions.push(inst);
        }

        AstKind::Ident(name) => instructions.push(Instruction::Load(name)),

        AstKind::Number(value) => instructions.push(Instruction::Push(Types::Number(value))),
        AstKind::Float(value) => instructions.push(Instruction::Push(Types::Float(value))),
        AstKind::String(value) => instructions.push(Instruction::Push(Types::String(value))),
        AstKind::Boolean(value) => instructions.push(Instruction::Push(Types::Boolean(value))),
        // AstKind::Vector(value) => {
        //     instructions.push(Instruction::Push(Types::Vector(
        //         value
        //             .iter()
        //             .map(|v| match v {
        //                 AstKind::Number(value) => Types::Number(*value),
        //                 AstKind::Float(value) => Types::Float(*value),
        //                 AstKind::String(value) => Types::String(value.clone()),
        //                 AstKind::Boolean(value) => Types::Boolean(*value),
        //                 _ => unreachable!(),
        //             })
        //             .collect(),
        //     )));
        // }
        _ => unreachable!(),
    };

    Ok(())
}

fn var_compile(name: String, value: AstKind, is_const: bool) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    expr_compile(&mut instructions, value)?;

    if is_const {
        instructions.push(Instruction::StoreConst(name))
    } else {
        instructions.push(Instruction::StoreLocal(name))
    }

    Ok(instructions)
}
