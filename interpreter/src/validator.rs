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

type Functions = HashMap<String, (Vec<(String, TypesCheck)>, ReturnType)>;

pub fn check_types(handler: &FileHandler, ast: &mut [AstNode]) {
    let mut consts = HashMap::new();

    consts.insert("PI".to_owned(), TypesCheck::Float);
    consts.insert("E".to_owned(), TypesCheck::Float);

    consts.insert("IS_LINUX".to_owned(), TypesCheck::Boolean);
    consts.insert("IS_MACOS".to_owned(), TypesCheck::Boolean);
    consts.insert("IS_WINDOWS".to_owned(), TypesCheck::Boolean);

    consts.insert("IS_X86_64".to_owned(), TypesCheck::Boolean);
    consts.insert("IS_ARM64".to_owned(), TypesCheck::Boolean);

    let mut funcs: HashMap<String, (Vec<(String, TypesCheck)>, ReturnType)> = HashMap::new();

    for node in ast.iter() {
        if let AstKind::Func {
            name,
            args,
            return_type,
            ..
        } = &node.kind
            && funcs
                .insert(name.clone(), (args.clone(), return_type.clone()))
                .is_some()
        {
            handler.show_error(node.line, errors::A38)
        }
    }

    match funcs.get("main") {
        Some((args, ret)) if args.is_empty() && *ret == ReturnType::Number => {}
        Some(_) => handler.show_error(0, errors::A22),
        None => {}
    }

    for node in ast.iter_mut() {
        match &mut node.kind {
            AstKind::VarDecl { name, type_, .. } => {
                if consts.insert(name.clone(), type_.clone()).is_some() {
                    handler.show_error(node.line, errors::A37)
                }
            }
            AstKind::Func {
                body,
                args,
                return_type,
                ..
            } => {
                let mut variables = HashMap::new();

                for arg in args.iter() {
                    variables.insert(arg.0.clone(), (arg.1.clone(), false));
                }

                for node in body.iter_mut() {
                    if let Err(e) =
                        main_check(&funcs, &mut variables, &consts, &mut node.kind, return_type)
                    {
                        handler.show_error(node.line, &e);
                    }
                }

                if *return_type != ReturnType::Void && !always_returns(body) {
                    handler.show_error(node.line, errors::A46);
                }
            }

            _ => unreachable!(),
        }
    }
}

fn always_returns(body: &[AstNode]) -> bool {
    for node in body.iter().rev() {
        match &node.kind {
            AstKind::Return(Some(_)) => return true,
            AstKind::Condition { yes, no, .. } => {
                let yes_ret = always_returns(yes);
                let no_ret = match no {
                    Some(ElseBlock::Else(els)) => always_returns(els),
                    Some(ElseBlock::ElseIf(n)) => always_returns(std::slice::from_ref(n)),
                    None => false,
                };

                if yes_ret && no_ret {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

fn main_check(
    funcs: &Functions,
    variables: &mut HashMap<String, (TypesCheck, bool)>,
    consts: &HashMap<String, TypesCheck>,
    node: &mut AstKind,
    return_type: &ReturnType,
) -> Result<(), String> {
    match node {
        AstKind::Return(None) => {
            if *return_type != ReturnType::Void {
                return Err(errors::A43.to_owned());
            }
        }
        AstKind::Return(Some(expr)) => {
            let t = expr_annotate(funcs, variables, consts, expr)?;
            let expected = match return_type {
                ReturnType::Number => TypesCheck::Number,
                ReturnType::Float => TypesCheck::Float,
                ReturnType::String => TypesCheck::String,
                ReturnType::Boolean => TypesCheck::Boolean,
                ReturnType::Void => return Err(errors::A43.to_owned()),
            };

            if t != expected {
                return Err(errors::A43.to_owned());
            }
        }
        AstKind::FuncCall { name, args } => {
            match name.as_str() {
                "input" => {
                    if !args.is_empty() {
                        return Err(errors::A27.to_owned());
                    }

                    return Ok(());
                }
                "sin" | "cos" | "sqrt" => {
                    if args.len() != 1 {
                        return Err(errors::A27.to_owned());
                    }

                    let t = expr_annotate(funcs, variables, consts, &mut args[0])?;
                    if t != TypesCheck::Float {
                        return Err(errors::A42.to_owned());
                    }

                    return Ok(());
                }
                "shell" => {
                    if args.len() != 1 {
                        return Err(errors::A27.to_owned());
                    }

                    let t = expr_annotate(funcs, variables, consts, &mut args[0])?;
                    if t != TypesCheck::String {
                        return Err(errors::A42.to_owned());
                    }

                    return Ok(());
                }
                _ => {}
            }

            if let Some((params, _)) = funcs.get(name) {
                if args.len() != params.len() {
                    return Err(errors::A27.to_owned());
                }

                for (i, arg) in args.iter_mut().enumerate() {
                    let t = expr_annotate(funcs, variables, consts, arg)?;
                    if t != params[i].1 {
                        return Err(errors::A42.to_owned());
                    }
                }
            } else {
                return Err(errors::A24.to_owned());
            }
        }
        AstKind::Delete(name) => {
            if let Some((_, is_const)) = variables.remove(name) {
                if is_const {
                    return Err(errors::A28.to_owned());
                }
            } else {
                return Err(errors::A03.to_owned());
            }
        }
        AstKind::VarDecl {
            name,
            type_,
            value,
            is_const,
        } => {
            let real_type = expr_annotate(funcs, variables, consts, value)?;

            if *type_ == real_type {
                if variables
                    .insert(name.clone(), (type_.clone(), *is_const))
                    .is_some()
                {
                    return Err(errors::A37.to_owned());
                }
            } else {
                return Err(errors::A43.to_owned());
            }
        }
        AstKind::Assign {
            name,
            op,
            expr,
            var_type,
        } => {
            if let Some((type_, is_const)) = variables.get(name) {
                *var_type = type_.clone();
                if !*is_const {
                    let value = expr_annotate(funcs, variables, consts, expr)?;

                    match op {
                        Token::Assign => {
                            if *var_type != value {
                                return Err(errors::A43.to_owned());
                            }
                        }
                        Token::PlusAssign => match (var_type, value) {
                            (TypesCheck::String, TypesCheck::String) => {}
                            (TypesCheck::String, TypesCheck::Number) => {}
                            (TypesCheck::String, TypesCheck::Float) => {}
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            (TypesCheck::Float, TypesCheck::Float) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::MinusAssign => match (var_type, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            (TypesCheck::Float, TypesCheck::Float) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::MultiplyAssign => match (var_type, value) {
                            (TypesCheck::String, TypesCheck::Number) => {}
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            (TypesCheck::Float, TypesCheck::Float) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::DivideAssign => match (var_type, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            (TypesCheck::Float, TypesCheck::Float) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::ModAssign => match (var_type, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::BitAndAssign => match (var_type, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::BitOrAssign => match (var_type, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::BitXorAssign => match (var_type, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::BitRightAssign => match (var_type, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::BitLeftAssign => match (var_type, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        _ => unreachable!(),
                    }
                } else {
                    return Err(errors::A07.to_owned());
                }
            } else {
                return Err(errors::A03.to_owned());
            }
        }
        AstKind::Condition { expr, yes, no } => {
            if expr_annotate(funcs, variables, consts, expr)? == TypesCheck::Boolean {
                for node in yes.iter_mut() {
                    main_check(funcs, variables, consts, &mut node.kind, return_type)?
                }

                match no {
                    Some(ElseBlock::ElseIf(node)) => {
                        main_check(funcs, variables, consts, &mut node.kind, return_type)?
                    }
                    Some(ElseBlock::Else(nodes)) => {
                        for node in nodes.iter_mut() {
                            main_check(funcs, variables, consts, &mut node.kind, return_type)?
                        }
                    }
                    None => {}
                };
            } else {
                return Err(errors::A15.to_owned());
            }
        }
        AstKind::While { expr, body } => {
            if expr_annotate(funcs, variables, consts, expr)? == TypesCheck::Boolean {
                for node in body.iter_mut() {
                    main_check(funcs, variables, consts, &mut node.kind, return_type)?
                }
            } else {
                return Err(errors::A15.to_owned());
            }
        }
        AstKind::For {
            var_name,
            start,
            end,
            step,
            body,
        } => {
            let start_type = expr_annotate(funcs, variables, consts, start)?;

            if !matches!(start_type, TypesCheck::Number) {
                return Err(errors::A15.to_owned());
            }

            if !matches!(
                expr_annotate(funcs, variables, consts, end)?,
                TypesCheck::Number
            ) {
                return Err(errors::A15.to_owned());
            }

            if let Some(step) = step.as_mut()
                && !matches!(
                    expr_annotate(funcs, variables, consts, step)?,
                    TypesCheck::Number
                )
            {
                return Err(errors::A15.to_owned());
            }

            variables.insert(var_name.clone(), (start_type, false));

            for node in body.iter_mut() {
                main_check(funcs, variables, consts, &mut node.kind, return_type)?
            }
        }
        AstKind::Echo(expr) => {
            expr_annotate(funcs, variables, consts, expr)?;
        }
        _ => {}
    }

    Ok(())
}

fn expr_annotate(
    funcs: &Functions,
    variables: &HashMap<String, (TypesCheck, bool)>,
    consts: &HashMap<String, TypesCheck>,
    node: &mut AstKind,
) -> Result<TypesCheck, String> {
    match node {
        AstKind::UnaryOp {
            expr, expr_type, ..
        } => {
            let t = expr_annotate(funcs, variables, consts, expr)?;
            *expr_type = t;
            expr_check(funcs, variables, consts, node)
        }
        AstKind::AsOp { expr, src_type, .. } => {
            let t = expr_annotate(funcs, variables, consts, expr)?;
            *src_type = t;
            expr_check(funcs, variables, consts, node)
        }
        AstKind::FuncCall { args, .. } => {
            for arg in args.iter_mut() {
                expr_annotate(funcs, variables, consts, arg)?;
            }

            expr_check(funcs, variables, consts, node)
        }
        AstKind::Ident(_) => expr_check(funcs, variables, consts, node),
        AstKind::BinaryOp {
            left,
            right,
            left_type,
            right_type,
            ..
        } => {
            let lt = expr_annotate(funcs, variables, consts, left)?;
            let rt = expr_annotate(funcs, variables, consts, right)?;
            *left_type = lt;
            *right_type = rt;
            expr_check(funcs, variables, consts, node)
        }
        _ => expr_check(funcs, variables, consts, node),
    }
}

fn expr_check(
    funcs: &Functions,
    variables: &HashMap<String, (TypesCheck, bool)>,
    consts: &HashMap<String, TypesCheck>,
    node: &AstKind,
) -> Result<TypesCheck, String> {
    match node {
        AstKind::Number(_) => Ok(TypesCheck::Number),
        AstKind::Float(_) => Ok(TypesCheck::Float),
        AstKind::String(_) => Ok(TypesCheck::String),
        AstKind::Boolean(_) => Ok(TypesCheck::Boolean),
        AstKind::Ident(name) => {
            if let Some((value, _)) = variables.get(name) {
                Ok(value.clone())
            } else if let Some(value) = consts.get(name) {
                Ok(value.clone())
            } else {
                Err(errors::A03.to_owned())
            }
        }
        AstKind::FuncCall { name, args } => {
            let argc = args.len();

            match name.as_str() {
                "input" => {
                    return match argc {
                        0 => Ok(TypesCheck::String),
                        _ => Err(errors::A27.to_owned()),
                    };
                }
                "sin" | "cos" | "sqrt" => {
                    return match argc {
                        1 => match expr_check(funcs, variables, consts, &args[0])? {
                            TypesCheck::Float => Ok(TypesCheck::Float),
                            _ => Err(errors::A42.to_owned()),
                        },
                        _ => Err(errors::A27.to_owned()),
                    };
                }
                "shell" => {
                    return match argc {
                        1 => match expr_check(funcs, variables, consts, &args[0])? {
                            TypesCheck::String => Ok(TypesCheck::Number),
                            _ => Err(errors::A42.to_owned()),
                        },
                        _ => Err(errors::A27.to_owned()),
                    };
                }
                _ => {}
            }

            if let Some((_, ret)) = funcs.get(name) {
                match ret {
                    ReturnType::Number => Ok(TypesCheck::Number),
                    ReturnType::Float => Ok(TypesCheck::Float),
                    ReturnType::String => Ok(TypesCheck::String),
                    ReturnType::Boolean => Ok(TypesCheck::Boolean),
                    ReturnType::Void => Err(errors::A15.to_owned()),
                }
            } else {
                Err(errors::A24.to_owned())
            }
        }
        AstKind::UnaryOp { op, expr, .. } => {
            let value = expr_check(funcs, variables, consts, expr)?;

            match op {
                Token::Not => match value {
                    TypesCheck::Boolean => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::BitNot => match value {
                    TypesCheck::Number => Ok(TypesCheck::Number),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::Minus => match value {
                    TypesCheck::Number => Ok(TypesCheck::Number),
                    TypesCheck::Float => Ok(TypesCheck::Float),
                    _ => Err(errors::A16.to_owned()),
                },
                _ => Err(errors::A15.to_owned()),
            }
        }
        AstKind::AsOp { expr, op, .. } => {
            let value = expr_check(funcs, variables, consts, expr)?;

            match op {
                Cast::String => match value {
                    TypesCheck::String => Err(errors::A40.to_owned()),
                    _ => Ok(TypesCheck::String),
                },
                Cast::Number => match value {
                    TypesCheck::Number => Err(errors::A40.to_owned()),
                    _ => Ok(TypesCheck::Number),
                },
                Cast::Float => match value {
                    TypesCheck::Float => Err(errors::A40.to_owned()),
                    _ => Ok(TypesCheck::Float),
                },
                Cast::Boolean => match value {
                    TypesCheck::Boolean => Err(errors::A40.to_owned()),
                    _ => Ok(TypesCheck::Boolean),
                },
            }
        }
        AstKind::BinaryOp {
            left, op, right, ..
        } => {
            let left = expr_check(funcs, variables, consts, left)?;
            let right = expr_check(funcs, variables, consts, right)?;

            match op {
                Token::Or => match (left, right) {
                    (TypesCheck::Boolean, TypesCheck::Boolean) => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::And => match (left, right) {
                    (TypesCheck::Boolean, TypesCheck::Boolean) => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::Equal => match (left, right) {
                    (TypesCheck::String, TypesCheck::String) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Boolean, TypesCheck::Boolean) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::NotEqual => match (left, right) {
                    (TypesCheck::String, TypesCheck::String) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Boolean, TypesCheck::Boolean) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::Greater => match (left, right) {
                    (TypesCheck::String, TypesCheck::String) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::Less => match (left, right) {
                    (TypesCheck::String, TypesCheck::String) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::GreaterEqual => match (left, right) {
                    (TypesCheck::String, TypesCheck::String) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::LessEqual => match (left, right) {
                    (TypesCheck::String, TypesCheck::String) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Boolean),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::Plus => match (left, right) {
                    (TypesCheck::String, TypesCheck::String) => Ok(TypesCheck::String),
                    (TypesCheck::String, TypesCheck::Number) => Ok(TypesCheck::String),
                    (TypesCheck::String, TypesCheck::Float) => Ok(TypesCheck::String),
                    (TypesCheck::Number, TypesCheck::String) => Ok(TypesCheck::String),
                    (TypesCheck::Float, TypesCheck::String) => Ok(TypesCheck::String),
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Float),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::Minus => match (left, right) {
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Float),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::Multiply => match (left, right) {
                    (TypesCheck::String, TypesCheck::Number) => Ok(TypesCheck::String),
                    (TypesCheck::Number, TypesCheck::String) => Ok(TypesCheck::String),
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Float),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::Divide => match (left, right) {
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    (TypesCheck::Float, TypesCheck::Float) => Ok(TypesCheck::Float),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::Mod => match (left, right) {
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::BitAnd => match (left, right) {
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::BitOr => match (left, right) {
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::BitXor => match (left, right) {
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::BitRight => match (left, right) {
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    _ => Err(errors::A16.to_owned()),
                },
                Token::BitLeft => match (left, right) {
                    (TypesCheck::Number, TypesCheck::Number) => Ok(TypesCheck::Number),
                    _ => Err(errors::A16.to_owned()),
                },

                _ => unreachable!(),
            }
        }
        _ => Err(errors::A15.to_owned()),
    }
}
