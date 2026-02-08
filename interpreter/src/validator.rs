// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use crate::{
    errors,
    file_handler::FileHandler,
    lexer::Token,
    parser::{AstKind, AstNode, Cast, ElseBlock},
    types::TypesCheck,
};

pub fn check_types(handler: &FileHandler, ast: &[AstNode]) {
    let mut consts = HashMap::new();
    let mut procs = HashMap::new();

    for node in ast {
        if let AstKind::Proc { name, args, .. } = &node.kind
            && procs.insert(name.clone(), args.clone()).is_some()
        {
            handler.show_error(node.line, errors::A38)
        }
    }

    for node in ast {
        match &node.kind {
            AstKind::VarDecl { name, type_, .. } => {
                if consts.insert(name.clone(), type_.clone()).is_some() {
                    handler.show_error(node.line, errors::A37)
                }
            }
            AstKind::Proc { body, args, .. } => {
                let mut variables = HashMap::new();

                for arg in args {
                    variables.insert(arg.0.clone(), (arg.1.clone(), false));
                }

                for node in body {
                    if let Err(e) = main_check(&procs, &mut variables, &consts, &node.kind) {
                        handler.show_error(node.line, &e);
                    }
                }
            }

            _ => unreachable!(),
        }
    }
}

fn main_check(
    procs: &HashMap<String, Vec<(String, TypesCheck)>>,
    variables: &mut HashMap<String, (TypesCheck, bool)>,
    consts: &HashMap<String, TypesCheck>,
    node: &AstKind,
) -> Result<(), String> {
    match node {
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
            let real_type = expr_check(variables, consts, value)?;

            if *type_ == real_type {
                if variables
                    .insert(name.clone(), (type_.clone(), *is_const))
                    .is_some()
                {
                    return Err(errors::A07.to_owned());
                }
            } else {
                return Err(errors::A43.to_owned());
            }
        }
        AstKind::Assign { name, op, expr } => {
            if let Some((type_, is_const)) = variables.get(name) {
                if !*is_const {
                    let value = expr_check(variables, consts, expr)?;

                    match op {
                        Token::Assign => {
                            if *type_ != value {
                                return Err(errors::A43.to_owned());
                            }
                        }
                        Token::PlusAssign => match (type_, value) {
                            (TypesCheck::String, TypesCheck::String) => {}
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            (TypesCheck::Float, TypesCheck::Float) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::MinusAssign => match (type_, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            (TypesCheck::Float, TypesCheck::Float) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::MultiplyAssign => match (type_, value) {
                            (TypesCheck::String, TypesCheck::Number) => {}
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            (TypesCheck::Float, TypesCheck::Float) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::DivideAssign => match (type_, value) {
                            (TypesCheck::Number, TypesCheck::Number) => {}
                            (TypesCheck::Float, TypesCheck::Float) => {}
                            _ => return Err(errors::A16.to_owned()),
                        },
                        Token::ModAssign => match (type_, value) {
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
        AstKind::ProcCall { name, args } => {
            if let Some(proc) = procs.get(name) {
                if args.len() != proc.len() {
                    return Err(errors::A27.to_owned());
                }

                for (i, arg) in args.iter().enumerate() {
                    let type_ = match arg {
                        Token::Ident(name) => {
                            expr_check(variables, consts, &AstKind::Ident(name.clone()))?
                        }
                        Token::NumberValue(_) => TypesCheck::Number,
                        Token::FloatValue(_) => TypesCheck::Float,
                        Token::StringValue(_) => TypesCheck::String,
                        Token::BooleanValue(_) => TypesCheck::Boolean,
                        _ => unreachable!(),
                    };

                    if type_ != proc[i].1 {
                        return Err(errors::A42.to_owned());
                    }
                }
            } else {
                return Err(errors::A24.to_owned());
            }
        }
        AstKind::Condition { expr, yes, no } => {
            if expr_check(variables, consts, expr)? == TypesCheck::Boolean {
                for node in yes {
                    main_check(procs, variables, consts, &node.kind)?;
                }

                match no {
                    Some(ElseBlock::ElseIf(node)) => {
                        main_check(procs, variables, consts, &node.kind)?
                    }
                    Some(ElseBlock::Else(nodes)) => {
                        for node in nodes {
                            main_check(procs, variables, consts, &node.kind)?;
                        }
                    }
                    None => {}
                };
            } else {
                return Err(errors::A15.to_owned());
            }
        }
        AstKind::While { expr, body } => {
            if expr_check(variables, consts, expr)? == TypesCheck::Boolean {
                for node in body {
                    main_check(procs, variables, consts, &node.kind)?;
                }
            } else {
                return Err(errors::A15.to_owned());
            }
        }
        AstKind::For {
            start,
            end,
            step,
            body,
            ..
        } => {
            if !matches!(
                expr_check(variables, consts, start)?,
                TypesCheck::Float | TypesCheck::Number
            ) {
                return Err(errors::A15.to_owned());
            }

            if !matches!(
                expr_check(variables, consts, end)?,
                TypesCheck::Float | TypesCheck::Number
            ) {
                return Err(errors::A15.to_owned());
            }

            if let Some(step) = *step.clone()
                && !matches!(
                    expr_check(variables, consts, &step)?,
                    TypesCheck::Float | TypesCheck::Number
                )
            {
                return Err(errors::A15.to_owned());
            }

            for node in body {
                main_check(procs, variables, consts, &node.kind)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn expr_check(
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
            } else if name == "input" {
                Ok(TypesCheck::String)
            } else {
                Err(errors::A03.to_owned())
            }
        }
        AstKind::UnaryOp { op, expr } => {
            let value = expr_check(variables, consts, expr)?;

            match op {
                Token::Not => match value {
                    TypesCheck::Boolean => Ok(TypesCheck::Boolean),
                    _ => Err(errors::A39.to_owned()),
                },
                Token::Minus => match value {
                    TypesCheck::Number => Ok(TypesCheck::Number),
                    TypesCheck::Float => Ok(TypesCheck::Float),
                    _ => Err(errors::A16.to_owned()),
                },
                _ => Err(errors::A15.to_owned()),
            }
        }
        AstKind::AsOp { expr, op } => {
            let value = expr_check(variables, consts, expr)?;

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
                Cast::Sin => match value {
                    TypesCheck::Float => Ok(TypesCheck::Float),
                    _ => Err(errors::A41.to_owned()),
                },
                Cast::Cos => match value {
                    TypesCheck::Float => Ok(TypesCheck::Float),
                    _ => Err(errors::A41.to_owned()),
                },
                Cast::Sqrt => match value {
                    TypesCheck::Float => Ok(TypesCheck::Float),
                    _ => Err(errors::A41.to_owned()),
                },
            }
        }
        AstKind::BinaryOp { left, op, right } => {
            let left = expr_check(variables, consts, left)?;
            let right = expr_check(variables, consts, right)?;

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
                _ => unreachable!(),
            }
        }
        _ => Err(errors::A15.to_owned()),
    }
}
