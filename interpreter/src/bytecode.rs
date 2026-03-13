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

#[inline(always)]
fn push_node(out: &mut Vec<Node>, instruction: Instruction, line: usize) {
    out.push(Node {
        instruction,
        line_number: line,
    });
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
                let mut proc_body: Vec<Node> = Vec::new();

                compile_body(handler, body, &mut proc_body);

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

fn compile_body(handler: &FileHandler, body: Vec<AstNode>, out: &mut Vec<Node>) {
    for node in body {
        let line = node.line;
        if let Err(e) = main_compile(node.kind, out, line) {
            handler.show_error(line, &e);
        }
    }
}

fn main_compile(node: AstKind, out: &mut Vec<Node>, line: usize) -> Result<(), String> {
    match node {
        AstKind::Echo(expr) => {
            expr_compile(out, *expr, line)?;
            push_node(out, Instruction::Echo, line);
        }
        AstKind::Delete(name) => {
            push_node(out, Instruction::Delete(name), line);
        }
        AstKind::Exit(code) => {
            push_node(out, Instruction::Exit(code), line);
        }
        AstKind::VarDecl {
            name,
            value,
            is_const,
            ..
        } => {
            for inst in var_compile(name, *value, is_const)? {
                push_node(out, inst, line);
            }
        }
        AstKind::ProcCall { name, args } => {
            for arg in args.iter().rev() {
                let inst = match arg {
                    Token::Ident(name) => Instruction::Load(name.clone()),
                    Token::NumberValue(value) => Instruction::Push(Types::Number(*value)),
                    Token::FloatValue(value) => Instruction::Push(Types::Float(*value)),
                    Token::StringValue(value) => Instruction::Push(Types::String(value.clone())),
                    Token::BooleanValue(value) => Instruction::Push(Types::Boolean(*value)),
                    _ => unreachable!(),
                };
                push_node(out, inst, line);
            }
            push_node(out, Instruction::Call(name, args.len()), line);
        }
        AstKind::Assign { name, op, expr } => {
            if op != Token::Assign {
                push_node(out, Instruction::Load(name.clone()), line);
            }

            expr_compile(out, *expr, line)?;

            match op {
                Token::Assign => {
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::PlusAssign => {
                    push_node(out, Instruction::Plus, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::MinusAssign => {
                    push_node(out, Instruction::Minus, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::MultiplyAssign => {
                    push_node(out, Instruction::Multiply, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::DivideAssign => {
                    push_node(out, Instruction::Divide, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::ModAssign => {
                    push_node(out, Instruction::Mod, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                _ => unreachable!(),
            }
        }
        AstKind::Condition { expr, yes, no } => {
            condition_compile(out, *expr, yes, no, line)?;
        }
        AstKind::While { expr, body } => {
            while_compile(out, *expr, body, line)?;
        }
        AstKind::For {
            var_name,
            start,
            end,
            step,
            body,
        } => {
            for_compile(out, var_name, *start, *end, *step, body, line)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn condition_compile(
    out: &mut Vec<Node>,
    expr: AstKind,
    yes: Vec<AstNode>,
    no: Option<ElseBlock>,
    line: usize,
) -> Result<(), String> {
    expr_compile(out, expr, line)?;

    let jump_if_false_idx = out.len();
    push_node(out, Instruction::JumpIfFalse(0), line);

    for node in yes {
        main_compile(node.kind, out, node.line)?;
    }

    match no {
        None => {
            let end = out.len();
            out[jump_if_false_idx].instruction = Instruction::JumpIfFalse(end);
        }
        Some(ElseBlock::Else(else_body)) => {
            let jump_idx = out.len();
            push_node(out, Instruction::Jump(0), line);

            let else_start = out.len();
            out[jump_if_false_idx].instruction = Instruction::JumpIfFalse(else_start);

            for node in else_body {
                main_compile(node.kind, out, node.line)?;
            }

            let end = out.len();
            out[jump_idx].instruction = Instruction::Jump(end);
        }
        Some(ElseBlock::ElseIf(node)) => {
            let jump_idx = out.len();
            push_node(out, Instruction::Jump(0), line);

            let elseif_start = out.len();
            out[jump_if_false_idx].instruction = Instruction::JumpIfFalse(elseif_start);

            main_compile(node.kind, out, node.line)?;

            let end = out.len();
            out[jump_idx].instruction = Instruction::Jump(end);
        }
    }

    Ok(())
}

fn while_compile(
    out: &mut Vec<Node>,
    expr: AstKind,
    body: Vec<AstNode>,
    line: usize,
) -> Result<(), String> {
    let loop_start = out.len();

    expr_compile(out, expr, line)?;

    let jump_if_false_idx = out.len();
    push_node(out, Instruction::JumpIfFalse(0), line);

    for node in body {
        main_compile(node.kind, out, node.line)?;
    }

    push_node(out, Instruction::Jump(loop_start), line);

    let end = out.len();
    out[jump_if_false_idx].instruction = Instruction::JumpIfFalse(end);

    Ok(())
}

fn for_compile(
    out: &mut Vec<Node>,
    var_name: String,
    start: AstKind,
    end: AstKind,
    step: Option<AstKind>,
    body: Vec<AstNode>,
    line: usize,
) -> Result<(), String> {
    expr_compile(out, start, line)?;
    push_node(out, Instruction::StoreLocal(var_name.clone()), line);

    let loop_start = out.len();

    expr_compile(out, end, line)?;
    push_node(out, Instruction::Load(var_name.clone()), line);
    push_node(out, Instruction::LessEqual, line);

    let jump_if_false_idx = out.len();
    push_node(out, Instruction::JumpIfFalse(0), line);

    for node in body {
        main_compile(node.kind, out, node.line)?;
    }

    push_node(out, Instruction::Load(var_name.clone()), line);
    match step {
        Some(step_expr) => expr_compile(out, step_expr, line)?,
        None => push_node(out, Instruction::Push(Types::Number(1)), line),
    }
    push_node(out, Instruction::Plus, line);
    push_node(out, Instruction::StoreLocal(var_name), line);

    push_node(out, Instruction::Jump(loop_start), line);

    let end_idx = out.len();
    out[jump_if_false_idx].instruction = Instruction::JumpIfFalse(end_idx);

    Ok(())
}

fn expr_compile(out: &mut Vec<Node>, value: AstKind, line: usize) -> Result<(), String> {
    match value {
        AstKind::UnaryOp { op, expr } => {
            expr_compile(out, *expr, line)?;
            let inst = match op {
                Token::Not => Instruction::Not,
                Token::Minus => Instruction::Neg,
                _ => return Err(errors::A15.to_owned()),
            };
            push_node(out, inst, line);
        }
        AstKind::AsOp { expr, op } => {
            expr_compile(out, *expr, line)?;
            let inst = match op {
                Cast::String => Instruction::CastToString,
                Cast::Number => Instruction::CastToNumber,
                Cast::Float => Instruction::CastToFloat,
                Cast::Boolean => Instruction::CastToBoolean,
                Cast::Sqrt => Instruction::Sqrt,
                Cast::Sin => Instruction::Sin,
                Cast::Cos => Instruction::Cos,
            };
            push_node(out, inst, line);
        }
        AstKind::BinaryOp { left, op, right } => {
            expr_compile(out, *right, line)?;
            expr_compile(out, *left, line)?;
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
            push_node(out, inst, line);
        }
        AstKind::Ident(name) => push_node(out, Instruction::Load(name), line),
        AstKind::Number(value) => push_node(out, Instruction::Push(Types::Number(value)), line),
        AstKind::Float(value) => push_node(out, Instruction::Push(Types::Float(value)), line),
        AstKind::String(value) => push_node(out, Instruction::Push(Types::String(value)), line),
        AstKind::Boolean(value) => push_node(out, Instruction::Push(Types::Boolean(value)), line),
        _ => unreachable!(),
    }

    Ok(())
}

fn var_compile(name: String, value: AstKind, is_const: bool) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();

    let mut temp: Vec<Node> = Vec::new();
    expr_compile(&mut temp, value, 0)?;

    instructions.extend(temp.into_iter().map(|n| n.instruction));

    if is_const {
        instructions.push(Instruction::StoreConst(name));
    } else {
        instructions.push(Instruction::StoreLocal(name));
    }

    Ok(instructions)
}
