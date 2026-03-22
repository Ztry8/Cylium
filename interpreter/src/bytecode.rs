// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use crate::{
    errors,
    file_handler::FileHandler,
    lexer::Token,
    parser::{AstKind, AstNode, Cast, ElseBlock},
    types::{ReturnType, TypesCheck},
};

#[derive(Debug)]
pub struct Node {
    pub instruction: Instruction,
    pub line_number: usize,
}

#[derive(Debug)]
pub struct Func {
    pub args: Vec<(String, TypesCheck)>,
    pub body: Vec<Node>,
    #[allow(dead_code)]
    pub return_type: ReturnType,
}

#[derive(Debug)]
pub enum Instruction {
    StoreConst(String),
    StoreLocal(String),
    Load(String),

    PushInt(i64),
    PushFloat(f64),
    PushBool(bool),
    PushStr(String),

    AddInt,
    SubInt,
    MulInt,
    DivInt,
    ModInt,
    NegInt,

    AndInt,
    OrInt,
    XorInt,
    NotInt,
    RightInt,
    LeftInt,

    AddFloat,
    SubFloat,
    MulFloat,
    DivFloat,
    ModFloat,
    NegFloat,

    ConcatStr,
    ConcatStrInt,
    ConcatStrFloat,
    ConcatIntStr,
    ConcatFloatStr,
    RepeatStr,
    RepeatStrRev,

    EqInt,
    NeInt,
    GtInt,
    LtInt,
    GeInt,
    LeInt,

    EqFloat,
    NeFloat,
    GtFloat,
    LtFloat,
    GeFloat,
    LeFloat,

    EqBool,
    NeBool,
    AndBool,
    OrBool,
    NotBool,

    EqStr,
    NeStr,
    GtStr,
    LtStr,
    GeStr,
    LeStr,

    IntToFloat,
    IntToBool,
    IntToStr,
    FloatToInt,
    FloatToBool,
    FloatToStr,
    BoolToInt,
    BoolToFloat,
    BoolToStr,
    StrToInt,
    StrToFloat,
    StrToBool,

    Echo,
    Exit(i32),
    Delete(String),

    CallFunc(String, usize),
    Return,
    ReturnValue,

    Jump(usize),
    JumpIfFalse(usize),
    Pop,

    NewArray(String, TypesCheck),
    ArrayStore(String),
    ArrayLoad(String),
    ArrayLen(String),

    NewArrayLit(usize),
    NewArrayFill,

    ArrayLoadDeep(String, usize),
    ArrayStoreDeep(String, usize),
}

#[inline(always)]
fn push_node(out: &mut Vec<Node>, instruction: Instruction, line: usize) {
    out.push(Node {
        instruction,
        line_number: line,
    });
}

pub fn compile(handler: &FileHandler, ast: Vec<AstNode>) -> (HashMap<String, Func>, Vec<Node>) {
    let mut const_code = Vec::new();
    let mut funcs = HashMap::new();

    let mut funcs_rt: HashMap<String, ReturnType> = HashMap::new();

    funcs_rt.insert("input".to_owned(), ReturnType::String);
    funcs_rt.insert("sin".to_owned(), ReturnType::Float);
    funcs_rt.insert("cos".to_owned(), ReturnType::Float);
    funcs_rt.insert("sqrt".to_owned(), ReturnType::Float);
    funcs_rt.insert("len".to_owned(), ReturnType::Number);

    for node in &ast {
        if let AstKind::Func {
            name, return_type, ..
        } = &node.kind
        {
            funcs_rt.insert(name.clone(), return_type.clone());
        }
    }

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
            AstKind::Func {
                name,
                body,
                args,
                return_type,
            } => {
                let mut proc_body: Vec<Node> = Vec::new();

                compile_body(handler, body, &mut proc_body, &funcs_rt);

                funcs.insert(
                    name,
                    Func {
                        args,
                        body: proc_body,
                        return_type,
                    },
                );
            }
            _ => unreachable!(),
        }
    }

    (funcs, const_code)
}

fn compile_body(
    handler: &FileHandler,
    body: Vec<AstNode>,
    out: &mut Vec<Node>,
    funcs_rt: &HashMap<String, ReturnType>,
) {
    for node in body {
        let line = node.line;
        if let Err(e) = main_compile(node.kind, out, line, funcs_rt) {
            handler.show_error(line, &e);
        }
    }
}

fn main_compile(
    node: AstKind,
    out: &mut Vec<Node>,
    line: usize,
    funcs_rt: &HashMap<String, ReturnType>,
) -> Result<(), String> {
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
        AstKind::Return(None) => {
            push_node(out, Instruction::Return, line);
        }
        AstKind::Return(Some(expr)) => {
            expr_compile(out, *expr, line)?;
            push_node(out, Instruction::ReturnValue, line);
        }
        AstKind::FuncCall { name, args } => {
            let argc = args.len();
            for arg in args.into_iter().rev() {
                expr_compile(out, arg, line)?;
            }
            push_node(out, Instruction::CallFunc(name.clone(), argc), line);

            let has_value = funcs_rt
                .get(&name)
                .map(|rt| *rt != ReturnType::Void)
                .unwrap_or(false);
            if has_value {
                push_node(out, Instruction::Pop, line);
            }
        }
        AstKind::ArrayDecl {
            name,
            elem_type,
            sizes,
            init,
            is_const: _,
        } => {
            if let AstKind::Array(elems) = init.as_ref() {
                let fill_value: Option<&AstKind> = if elems.len() == 1 {
                    if let AstKind::Array(inner) = &elems[0] {
                        if inner.len() == 1 {
                            Some(&inner[0])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if !elems.is_empty()
                    && elems
                        .iter()
                        .all(|e| matches!(e, AstKind::Array(inner) if inner.len() == 1))
                {
                    if let AstKind::Array(inner) = &elems[0] {
                        Some(&inner[0])
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(fill) = fill_value {
                    if !sizes.is_empty() {
                        for sz in &sizes {
                            expr_compile(out, *sz.clone(), line)?;
                        }

                        expr_compile(out, fill.clone(), line)?;
                        for _ in 0..sizes.len() {
                            push_node(out, Instruction::NewArrayFill, line);
                        }

                        push_node(out, Instruction::StoreLocal(name), line);
                    } else {
                        return Err(errors::A15.to_owned());
                    }
                } else {
                    let n = elems.len();
                    for el in elems {
                        expr_compile(out, el.clone(), line)?;
                    }

                    push_node(out, Instruction::PushInt(n as i64), line);
                    push_node(out, Instruction::NewArray(name, elem_type), line);
                }
            } else {
                return Err(errors::A15.to_owned());
            }
        }
        AstKind::ArraySet {
            name,
            indices,
            op,
            expr,
            elem_type,
        } => {
            let depth = indices.len();
            if op != Token::Assign {
                for idx in &indices {
                    expr_compile(out, idx.clone(), line)?;
                }

                if depth == 1 {
                    push_node(out, Instruction::ArrayLoad(name.clone()), line);
                } else {
                    push_node(out, Instruction::ArrayLoadDeep(name.clone(), depth), line);
                }

                expr_compile(out, *expr, line)?;
                let inst = match (op, &elem_type) {
                    (Token::PlusAssign, TypesCheck::Float) => Instruction::AddFloat,
                    (Token::PlusAssign, _) => Instruction::AddInt,
                    (Token::MinusAssign, TypesCheck::Float) => Instruction::SubFloat,
                    (Token::MinusAssign, _) => Instruction::SubInt,
                    (Token::MultiplyAssign, TypesCheck::Float) => Instruction::MulFloat,
                    (Token::MultiplyAssign, _) => Instruction::MulInt,
                    (Token::DivideAssign, TypesCheck::Float) => Instruction::DivFloat,
                    (Token::DivideAssign, _) => Instruction::DivInt,
                    (Token::ModAssign, _) => Instruction::ModInt,
                    _ => Instruction::AddInt,
                };

                push_node(out, inst, line);
                for idx in indices {
                    expr_compile(out, idx, line)?;
                }
            } else {
                expr_compile(out, *expr, line)?;
                for idx in indices {
                    expr_compile(out, idx, line)?;
                }
            }
            if depth == 1 {
                push_node(out, Instruction::ArrayStore(name), line);
            } else {
                push_node(out, Instruction::ArrayStoreDeep(name, depth), line);
            }
        }
        AstKind::ArrayGet { name, indices } => {
            let depth = indices.len();
            for idx in indices {
                expr_compile(out, idx, line)?;
            }

            if depth == 1 {
                push_node(out, Instruction::ArrayLoad(name), line);
            } else {
                push_node(out, Instruction::ArrayLoadDeep(name, depth), line);
            }
        }
        AstKind::Assign {
            name,
            op,
            expr,
            var_type,
        } => {
            if op != Token::Assign {
                push_node(out, Instruction::Load(name.clone()), line);
            }

            expr_compile(out, *expr, line)?;

            match op {
                Token::Assign => {
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::PlusAssign => {
                    let inst = match var_type {
                        TypesCheck::Float => Instruction::AddFloat,
                        TypesCheck::String => Instruction::ConcatStr,
                        _ => Instruction::AddInt,
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::MinusAssign => {
                    let inst = match var_type {
                        TypesCheck::Float => Instruction::SubFloat,
                        _ => Instruction::SubInt,
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::MultiplyAssign => {
                    let inst = match var_type {
                        TypesCheck::Float => Instruction::MulFloat,
                        TypesCheck::String => Instruction::RepeatStr,
                        _ => Instruction::MulInt,
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::DivideAssign => {
                    let inst = match var_type {
                        TypesCheck::Float => Instruction::DivFloat,
                        _ => Instruction::DivInt,
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::ModAssign => {
                    let inst = match var_type {
                        TypesCheck::Float => Instruction::ModFloat,
                        _ => Instruction::ModInt,
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::BitAndAssign => {
                    let inst = match var_type {
                        TypesCheck::Number => Instruction::AndInt,
                        _ => unreachable!(),
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::BitOrAssign => {
                    let inst = match var_type {
                        TypesCheck::Number => Instruction::OrInt,
                        _ => unreachable!(),
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::BitXorAssign => {
                    let inst = match var_type {
                        TypesCheck::Number => Instruction::XorInt,
                        _ => unreachable!(),
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::BitRightAssign => {
                    let inst = match var_type {
                        TypesCheck::Number => Instruction::RightInt,
                        _ => unreachable!(),
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                Token::BitLeftAssign => {
                    let inst = match var_type {
                        TypesCheck::Number => Instruction::LeftInt,
                        _ => unreachable!(),
                    };
                    push_node(out, inst, line);
                    push_node(out, Instruction::StoreLocal(name), line);
                }
                _ => unreachable!(),
            }
        }
        AstKind::Condition { expr, yes, no } => {
            condition_compile(out, *expr, yes, no, line, funcs_rt)?;
        }
        AstKind::While { expr, body } => {
            while_compile(out, *expr, body, line, funcs_rt)?;
        }
        AstKind::For {
            var_name,
            start,
            end,
            step,
            body,
        } => {
            for_compile(out, var_name, *start, *end, *step, body, line, funcs_rt)?;
        }
        AstKind::ForIn {
            var_name,
            array_name,
            body,
        } => for_in_compile(out, var_name, array_name, body, line, funcs_rt)?,
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
    funcs_rt: &HashMap<String, ReturnType>,
) -> Result<(), String> {
    expr_compile(out, expr, line)?;
    let jif_idx = out.len();

    push_node(out, Instruction::JumpIfFalse(0), line);

    for node in yes {
        main_compile(node.kind, out, node.line, funcs_rt)?;
    }

    match no {
        None => {
            let end = out.len();
            out[jif_idx].instruction = Instruction::JumpIfFalse(end);
        }
        Some(ElseBlock::Else(else_body)) => {
            let jump_idx = out.len();
            push_node(out, Instruction::Jump(0), line);
            let else_start = out.len();
            out[jif_idx].instruction = Instruction::JumpIfFalse(else_start);
            for node in else_body {
                main_compile(node.kind, out, node.line, funcs_rt)?;
            }
            let end = out.len();
            out[jump_idx].instruction = Instruction::Jump(end);
        }
        Some(ElseBlock::ElseIf(node)) => {
            let jump_idx = out.len();
            push_node(out, Instruction::Jump(0), line);
            let elseif_start = out.len();
            out[jif_idx].instruction = Instruction::JumpIfFalse(elseif_start);
            main_compile(node.kind, out, node.line, funcs_rt)?;
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
    funcs_rt: &HashMap<String, ReturnType>,
) -> Result<(), String> {
    let loop_start = out.len();
    expr_compile(out, expr, line)?;
    let jif_idx = out.len();
    push_node(out, Instruction::JumpIfFalse(0), line);
    for node in body {
        main_compile(node.kind, out, node.line, funcs_rt)?;
    }
    push_node(out, Instruction::Jump(loop_start), line);
    let end = out.len();
    out[jif_idx].instruction = Instruction::JumpIfFalse(end);
    Ok(())
}

fn for_in_compile(
    out: &mut Vec<Node>,
    var_name: String,
    array_name: String,
    body: Vec<AstNode>,
    line: usize,
    funcs_rt: &HashMap<String, ReturnType>,
) -> Result<(), String> {
    let idx_var = format!("\x00fi_{}", var_name);
    push_node(out, Instruction::PushInt(0), line);
    push_node(out, Instruction::StoreLocal(idx_var.clone()), line);

    let loop_start = out.len();

    push_node(out, Instruction::Load(idx_var.clone()), line);
    push_node(out, Instruction::ArrayLen(array_name.clone()), line);
    push_node(out, Instruction::GtInt, line);

    let jif_idx = out.len();
    push_node(out, Instruction::JumpIfFalse(0), line);

    push_node(out, Instruction::Load(idx_var.clone()), line);
    push_node(out, Instruction::ArrayLoad(array_name.clone()), line);
    push_node(out, Instruction::StoreLocal(var_name.clone()), line);

    for node in body {
        main_compile(node.kind, out, node.line, funcs_rt)?;
    }

    push_node(out, Instruction::Load(idx_var.clone()), line);
    push_node(out, Instruction::PushInt(1), line);
    push_node(out, Instruction::AddInt, line);
    push_node(out, Instruction::StoreLocal(idx_var), line);

    push_node(out, Instruction::Jump(loop_start), line);
    let end_idx = out.len();
    out[jif_idx].instruction = Instruction::JumpIfFalse(end_idx);

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
    funcs_rt: &HashMap<String, ReturnType>,
) -> Result<(), String> {
    expr_compile(out, start, line)?;
    push_node(out, Instruction::StoreLocal(var_name.clone()), line);

    let end_var = format!("\x00fe_{}", var_name);
    expr_compile(out, end, line)?;
    push_node(out, Instruction::StoreLocal(end_var.clone()), line);

    let step_var = match step {
        Some(step_expr) => {
            let sv = format!("\x00fs_{}", var_name);
            expr_compile(out, step_expr, line)?;
            push_node(out, Instruction::StoreLocal(sv.clone()), line);
            Some(sv)
        }
        None => None,
    };

    let loop_start = out.len();

    push_node(out, Instruction::Load(end_var.clone()), line);
    push_node(out, Instruction::Load(var_name.clone()), line);
    push_node(out, Instruction::LeInt, line);

    let jif_idx = out.len();
    push_node(out, Instruction::JumpIfFalse(0), line);

    for node in body {
        main_compile(node.kind, out, node.line, funcs_rt)?;
    }

    push_node(out, Instruction::Load(var_name.clone()), line);
    match step_var {
        Some(sv) => push_node(out, Instruction::Load(sv), line),
        None => push_node(out, Instruction::PushInt(1), line),
    }
    push_node(out, Instruction::AddInt, line);
    push_node(out, Instruction::StoreLocal(var_name), line);

    push_node(out, Instruction::Jump(loop_start), line);
    let end_idx = out.len();
    out[jif_idx].instruction = Instruction::JumpIfFalse(end_idx);
    Ok(())
}

fn expr_compile(out: &mut Vec<Node>, value: AstKind, line: usize) -> Result<(), String> {
    match value {
        AstKind::UnaryOp {
            op,
            expr,
            expr_type,
        } => {
            expr_compile(out, *expr, line)?;
            let inst = match op {
                Token::Not => Instruction::NotBool,
                Token::BitNot => Instruction::NotInt,
                Token::Minus => match expr_type {
                    TypesCheck::Float => Instruction::NegFloat,
                    _ => Instruction::NegInt,
                },
                _ => return Err(errors::A15.to_owned()),
            };
            push_node(out, inst, line);
        }
        AstKind::AsOp { expr, op, src_type } => {
            expr_compile(out, *expr, line)?;
            push_node(out, cast_inst(&op, &src_type), line);
        }
        AstKind::FuncCall { name, args } => {
            let argc = args.len();
            for arg in args.into_iter().rev() {
                expr_compile(out, arg, line)?;
            }
            push_node(out, Instruction::CallFunc(name, argc), line);
        }
        AstKind::ArrayGet { name, indices } => {
            let depth = indices.len();
            for idx in indices {
                expr_compile(out, idx, line)?;
            }

            if depth == 1 {
                push_node(out, Instruction::ArrayLoad(name), line);
            } else {
                push_node(out, Instruction::ArrayLoadDeep(name, depth), line);
            }
        }
        AstKind::Array(elems) => {
            let n = elems.len();
            for el in elems {
                expr_compile(out, el, line)?;
            }
            
            push_node(out, Instruction::NewArrayLit(n), line);
        }
        AstKind::BinaryOp {
            left,
            op,
            right,
            left_type,
            right_type,
        } => {
            expr_compile(out, *right, line)?;
            expr_compile(out, *left, line)?;
            push_node(out, binary_op_inst(op, &left_type, &right_type), line);
        }
        AstKind::Ident(name) => push_node(out, Instruction::Load(name), line),
        AstKind::Number(v) => push_node(out, Instruction::PushInt(v), line),
        AstKind::Float(v) => push_node(out, Instruction::PushFloat(v), line),
        AstKind::String(v) => push_node(out, Instruction::PushStr(v), line),
        AstKind::Boolean(v) => push_node(out, Instruction::PushBool(v), line),
        _ => unreachable!(),
    }
    Ok(())
}

#[inline(always)]
fn cast_inst(op: &Cast, src: &TypesCheck) -> Instruction {
    match op {
        Cast::String => match src {
            TypesCheck::Number => Instruction::IntToStr,
            TypesCheck::Float => Instruction::FloatToStr,
            TypesCheck::Boolean => Instruction::BoolToStr,
            _ => unreachable!(),
        },
        Cast::Number => match src {
            TypesCheck::Float => Instruction::FloatToInt,
            TypesCheck::Boolean => Instruction::BoolToInt,
            TypesCheck::String => Instruction::StrToInt,
            _ => unreachable!(),
        },
        Cast::Float => match src {
            TypesCheck::Number => Instruction::IntToFloat,
            TypesCheck::Boolean => Instruction::BoolToFloat,
            TypesCheck::String => Instruction::StrToFloat,
            _ => unreachable!(),
        },
        Cast::Boolean => match src {
            TypesCheck::Number => Instruction::IntToBool,
            TypesCheck::Float => Instruction::FloatToBool,
            TypesCheck::String => Instruction::StrToBool,
            _ => unreachable!(),
        },
    }
}

#[inline(always)]
fn binary_op_inst(op: Token, lt: &TypesCheck, rt: &TypesCheck) -> Instruction {
    match op {
        Token::Or => Instruction::OrBool,
        Token::And => Instruction::AndBool,
        Token::Plus => match (lt, rt) {
            (TypesCheck::Number, TypesCheck::Number) => Instruction::AddInt,
            (TypesCheck::Float, TypesCheck::Float) => Instruction::AddFloat,
            (TypesCheck::String, TypesCheck::String) => Instruction::ConcatStr,
            (TypesCheck::String, TypesCheck::Number) => Instruction::ConcatStrInt,
            (TypesCheck::String, TypesCheck::Float) => Instruction::ConcatStrFloat,
            (TypesCheck::Number, TypesCheck::String) => Instruction::ConcatIntStr,
            (TypesCheck::Float, TypesCheck::String) => Instruction::ConcatFloatStr,
            _ => unreachable!(),
        },
        Token::Minus => match lt {
            TypesCheck::Float => Instruction::SubFloat,
            _ => Instruction::SubInt,
        },
        Token::Multiply => match (lt, rt) {
            (TypesCheck::Number, TypesCheck::Number) => Instruction::MulInt,
            (TypesCheck::Float, TypesCheck::Float) => Instruction::MulFloat,
            (TypesCheck::String, TypesCheck::Number) => Instruction::RepeatStr,
            (TypesCheck::Number, TypesCheck::String) => Instruction::RepeatStrRev,
            _ => unreachable!(),
        },
        Token::Divide => match lt {
            TypesCheck::Float => Instruction::DivFloat,
            _ => Instruction::DivInt,
        },
        Token::Mod => match lt {
            TypesCheck::Float => Instruction::ModFloat,
            _ => Instruction::ModInt,
        },
        Token::Equal => match lt {
            TypesCheck::Number => Instruction::EqInt,
            TypesCheck::Float => Instruction::EqFloat,
            TypesCheck::String => Instruction::EqStr,
            TypesCheck::Boolean => Instruction::EqBool,
            _ => unreachable!(),
        },
        Token::NotEqual => match lt {
            TypesCheck::Number => Instruction::NeInt,
            TypesCheck::Float => Instruction::NeFloat,
            TypesCheck::String => Instruction::NeStr,
            TypesCheck::Boolean => Instruction::NeBool,
            _ => unreachable!(),
        },
        Token::Greater => match lt {
            TypesCheck::Float => Instruction::GtFloat,
            TypesCheck::String => Instruction::GtStr,
            _ => Instruction::GtInt,
        },
        Token::Less => match lt {
            TypesCheck::Float => Instruction::LtFloat,
            TypesCheck::String => Instruction::LtStr,
            _ => Instruction::LtInt,
        },
        Token::GreaterEqual => match lt {
            TypesCheck::Float => Instruction::GeFloat,
            TypesCheck::String => Instruction::GeStr,
            _ => Instruction::GeInt,
        },
        Token::LessEqual => match lt {
            TypesCheck::Float => Instruction::LeFloat,
            TypesCheck::String => Instruction::LeStr,
            _ => Instruction::LeInt,
        },
        Token::BitAnd => match lt {
            TypesCheck::Number => Instruction::AndInt,
            _ => unreachable!(),
        },
        Token::BitOr => match lt {
            TypesCheck::Number => Instruction::OrInt,
            _ => unreachable!(),
        },
        Token::BitXor => match lt {
            TypesCheck::Number => Instruction::XorInt,
            _ => unreachable!(),
        },
        Token::BitRight => match lt {
            TypesCheck::Number => Instruction::RightInt,
            _ => unreachable!(),
        },
        Token::BitLeft => match lt {
            TypesCheck::Number => Instruction::LeftInt,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn var_compile(name: String, value: AstKind, is_const: bool) -> Result<Vec<Instruction>, String> {
    let mut temp: Vec<Node> = Vec::new();
    expr_compile(&mut temp, value, 0)?;
    let mut instructions: Vec<Instruction> = temp.into_iter().map(|n| n.instruction).collect();
    if is_const {
        instructions.push(Instruction::StoreConst(name));
    } else {
        instructions.push(Instruction::StoreLocal(name));
    }
    Ok(instructions)
}
