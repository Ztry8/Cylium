// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{
    errors,
    file_handler::FileHandler,
    lexer::Token,
    parser::{AstKind, AstNode, ElseBlock},
    types::Types,
};

use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug)]
enum VectorIndex {
    Index(usize),
    Length,
    Capacity,
}

#[derive(Clone)]
struct Frame {
    vars: HashMap<String, (Types, bool)>,
}

pub struct Interpreter {
    procs: HashMap<String, (Vec<String>, Vec<AstNode>)>,
    consts: HashMap<String, (Types, bool)>,
    handler: FileHandler,
}

impl Interpreter {
    pub fn new(handler: FileHandler, ast: &[AstNode]) -> Self {
        let mut procs = HashMap::new();

        let mut consts_frame = Frame {
            vars: HashMap::new(),
        };

        consts_frame
            .vars
            .insert("PI".to_owned(), (Types::Float(std::f32::consts::PI), true));

        consts_frame.vars.insert(
            "TAU".to_owned(),
            (Types::Float(std::f32::consts::TAU), true),
        );

        consts_frame
            .vars
            .insert("E".to_owned(), (Types::Float(std::f32::consts::E), true));

        consts_frame.vars.insert(
            "SQRT_2".to_owned(),
            (Types::Float(std::f32::consts::SQRT_2), true),
        );

        for node in ast {
            if let AstKind::Proc { name, args, body } = &node.kind {
                procs.insert(name.clone(), (args.clone(), body.clone()));
            } else if let AstKind::VarDecl {
                name,
                value,
                is_const,
            } = &node.kind
            {
                if *is_const {
                    let value = Self::eval(*value.clone(), &consts_frame)
                        .unwrap_or_else(|e| handler.show_error(node.line, &e));

                    consts_frame.vars.insert(name.to_string(), (value, true));
                } else {
                    handler.show_error(node.line, errors::A20);
                }
            } else {
                handler.show_error(node.line, errors::A20);
            }
        }

        if !procs.contains_key("main") {
            handler.show_error(0, errors::A22);
        }

        Self {
            handler,
            consts: consts_frame.vars,
            procs,
        }
    }

    fn get_vector(frame: &Frame, name: &str) -> Result<(String, VectorIndex), String> {
        let mut chars: Vec<char> = name.chars().collect();
        let mut grab_name = true;

        let mut name = String::new();
        let mut index = String::new();

        if chars.pop() != Some(']') {
            return Err(errors::A17.to_owned());
        }

        for sym in chars {
            if sym == '[' {
                grab_name = false;
                continue;
            }

            if grab_name {
                name.push(sym);
            } else {
                index.push(sym);
            }
        }

        if let Ok(num) = index.parse::<usize>() {
            Ok((name, VectorIndex::Index(num)))
        } else {
            match index.as_str() {
                "length" => Ok((name, VectorIndex::Length)),
                "capacity" => Ok((name, VectorIndex::Capacity)),
                _ => {
                    if let Some((Types::Number(index), _)) = frame.vars.get(&index) {
                        Ok((name, VectorIndex::Index((*index) as usize)))
                    } else {
                        Err(errors::A17.to_owned())
                    }
                }
            }
        }
    }

    pub fn run(&self) {
        let (_, body) = self.procs.get("main").unwrap().clone();
        let mut frame = Frame {
            vars: HashMap::new(),
        };

        frame.vars.extend(self.consts.clone());

        for stmt in &body {
            if let Err(e) = self.exec(stmt.clone(), &mut frame) {
                self.handler.show_error(stmt.line, &e);
            }
        }
    }

    fn exec(&self, node: AstNode, frame: &mut Frame) -> Result<(), String> {
        match node.kind {
            AstKind::VarDecl {
                name,
                value,
                is_const,
            } => {
                let v = match *value {
                    AstKind::Ident(ref n) if n == "input" => {
                        io::stdout().flush().unwrap();
                        let mut buf = String::new();
                        if io::stdin().read_line(&mut buf).is_err() {
                            FileHandler::show_warning(errors::C02);
                        }

                        Types::String(buf.trim().to_string())
                    }
                    _ => Self::eval(*value, frame)?,
                };

                frame.vars.insert(name, (v, is_const));
            }

            AstKind::Delete(name) => {
                if let Some((_, is_const)) = frame.vars.remove(&name) {
                    if is_const {
                        return Err(errors::A28.to_owned());
                    }
                } else {
                    return Err(errors::A03.to_owned());
                }
            }

            AstKind::Assign { name, op, expr } => {
                let rhs = Self::eval(*expr, frame)?;

                let (cur, is_const) = if name.contains('[') {
                    let (name, index) = Self::get_vector(frame, &name)?;

                    if let Some((Types::Vector(vec), is_const)) = frame.vars.get(&name) {
                        match index {
                            VectorIndex::Index(index) => (
                                vec.get(index).ok_or(errors::A17.to_owned()).cloned()?,
                                *is_const,
                            ),
                            _ => return Err(errors::A15.to_owned()),
                        }
                    } else {
                        return Err(errors::A30.to_owned());
                    }
                } else {
                    frame.vars.get(&name).ok_or(errors::A03)?.clone()
                };

                if is_const {
                    return Err(errors::A07.to_owned());
                }

                let new = match op {
                    Token::Assign => rhs,
                    Token::PlusAssign => cur.clone().add(rhs)?,
                    Token::MinusAssign => cur.clone().sub(rhs)?,
                    Token::MultiplyAssign => cur.clone().mul(rhs)?,
                    Token::DivideAssign => cur.clone().div(rhs)?,
                    Token::ModAssign => cur.clone().rem(rhs)?,
                    _ => return Err(errors::A15.to_owned()),
                };

                frame.vars.insert(name, (new, false));
            }

            AstKind::Echo(expr) => {
                let v = Self::eval(*expr, frame)?;

                println!("{}", v);
            }

            AstKind::Exit(code) => std::process::exit(code),

            AstKind::MethodCall {
                receiver,
                name,
                args,
            } => {}

            AstKind::ProcCall { name, args } => {
                let (params, body) = self.procs.get(&name).ok_or(errors::A24)?.clone();

                if params.len() != args.len() {
                    return Err(errors::A27.to_owned());
                }

                let mut new_frame = Frame {
                    vars: HashMap::new(),
                };

                new_frame.vars.extend(self.consts.clone());

                for (i, p) in params.iter().enumerate() {
                    let val = match &args[i] {
                        Token::Value(v) => v.clone(),
                        Token::Ident(n) => frame.vars.get(n).cloned().ok_or(errors::A03)?.0,
                        _ => return Err(errors::A15.to_owned()),
                    };

                    new_frame.vars.insert(p.clone(), (val, false));
                }

                for stmt in body {
                    if let Err(e) = self.exec(stmt.clone(), &mut new_frame) {
                        self.handler.show_error(node.line, &e);
                    }
                }
            }

            AstKind::While { expr, body } => {
                while Self::eval(*expr.clone(), frame)?.as_bool()? {
                    for stmt in &body {
                        self.exec(stmt.clone(), frame)?;
                    }
                }
            }

            AstKind::For {
                var_name,
                start,
                end,
                step,
                body,
            } => {
                let start = Self::eval(*start, frame)?;
                let end = Self::eval(*end, frame)?;

                let step = Self::eval(
                    step.unwrap_or_else(|| {
                        if matches!(end, Types::Float(_)) {
                            if start < end {
                                AstKind::Value(Types::Float(1.0))
                            } else {
                                AstKind::Value(Types::Float(-1.0))
                            }
                        } else {
                            if start < end {
                                AstKind::Value(Types::Number(1))
                            } else {
                                AstKind::Value(Types::Number(-1))
                            }
                        }
                    }),
                    frame,
                )?;

                if !((matches!(start, Types::Number(_))
                    && matches!(end, Types::Number(_))
                    && matches!(step, Types::Number(_)))
                    || (matches!(start, Types::Float(_))
                        && matches!(end, Types::Float(_))
                        && matches!(step, Types::Float(_))))
                {
                    return Err(errors::A31.to_owned());
                }

                if step.is_zero()?
                    || (start < end && step.is_neg()?)
                    || (start > end && !step.is_neg()?)
                {
                    return Err(errors::A33.to_owned());
                }

                frame.vars.insert(var_name.clone(), (start.clone(), false));

                loop {
                    if let Some((counter, _)) = frame.vars.get(&var_name).cloned() {
                        if if start < end {
                            counter < end
                        } else {
                            counter > end
                        } {
                            for stmt in &body {
                                self.exec(stmt.clone(), frame)?;
                            }

                            let (var, _) = frame
                                .vars
                                .get_mut(&var_name)
                                .ok_or(errors::A32.to_owned())?;
                            
                            *var = counter.add(step.clone())?;
                        } else {
                            break;
                        }
                    } else {
                        return Err(errors::A32.to_owned());
                    }
                }
            }

            AstKind::Condition { expr, yes, no } => {
                if Self::eval(*expr, frame)?.as_bool()? {
                    for s in yes {
                        self.exec(s, frame)?;
                    }
                } else if let Some(n) = no {
                    match n {
                        ElseBlock::Else(b) => {
                            for s in b {
                                self.exec(s, frame)?;
                            }
                        }
                        ElseBlock::ElseIf(n) => self.exec(*n, frame)?,
                    }
                }
            }

            _ => return Err(errors::A15.to_owned()),
        }

        Ok(())
    }

    fn eval(expr: AstKind, frame: &Frame) -> Result<Types, String> {
        match expr {
            AstKind::Value(v) => Ok(v),

            AstKind::Ident(n) => {
                if matches!(
                    n.as_str(),
                    "number" | "float" | "bool" | "string" | "sqrt" | "cos" | "sin"
                ) {
                    Ok(Types::String(n))
                } else if n.as_str() == "vector" {
                    Ok(Types::Vector(Vec::new()))
                } else if n.contains('[') {
                    let (name, index) = Self::get_vector(frame, &n)?;

                    if let Some((Types::Vector(vec), _)) = frame.vars.get(&name) {
                        match index {
                            VectorIndex::Index(index) => {
                                vec.get(index).ok_or(errors::A17.to_owned()).cloned()
                            }
                            VectorIndex::Length => Ok(Types::Number(vec.len() as i32)),
                            VectorIndex::Capacity => Ok(Types::Number(vec.capacity() as i32)),
                        }
                    } else {
                        Err(errors::A30.to_owned())
                    }
                } else {
                    Ok(frame.vars.get(&n).cloned().ok_or(errors::A03.to_owned())?.0)
                }
            }

            AstKind::UnaryOp { op, expr } => {
                let v = Self::eval(*expr, frame)?;

                match op {
                    Token::Not => Ok(Types::Boolean(!v.as_bool()?)),
                    Token::Minus => Ok(Types::Number(-v.as_number()?)),
                    _ => Err(errors::A15.to_owned()),
                }
            }

            AstKind::BinaryOp { left, op, right } => {
                let mut l = Self::eval(*left, frame)?;
                let r = Self::eval(*right, frame)?;

                match op {
                    Token::As => {
                        let target = r.as_string()?;
                        match target {
                            "number" => {
                                if let Some(e) = l.convert_to_number()? {
                                    FileHandler::show_warning(&e);
                                }
                            }
                            "float" => {
                                if let Some(e) = l.convert_to_float()? {
                                    FileHandler::show_warning(&e);
                                }
                            }
                            "bool" => {
                                if let Some(e) = l.convert_to_bool()? {
                                    FileHandler::show_warning(&e);
                                }
                            }
                            "string" => {
                                if let Some(e) = l.convert_to_string()? {
                                    FileHandler::show_warning(&e);
                                }
                            }
                            "sqrt" => l.sqrt()?,
                            "cos" => l.cos()?,
                            "sin" => l.sin()?,
                            _ => return Err(errors::A08.to_owned()),
                        }

                        Ok(l)
                    }

                    Token::Plus => l.add(r),
                    Token::Minus => l.sub(r),
                    Token::Multiply => l.mul(r),
                    Token::Divide => l.div(r),
                    Token::Mod => l.rem(r),

                    Token::Equal => Ok(Types::Boolean(l == r)),
                    Token::NotEqual => Ok(Types::Boolean(l != r)),
                    Token::Greater => Ok(Types::Boolean(l > r)),
                    Token::Less => Ok(Types::Boolean(l < r)),
                    Token::GreaterEqual => Ok(Types::Boolean(l >= r)),
                    Token::LessEqual => Ok(Types::Boolean(l <= r)),

                    Token::And => Ok(Types::Boolean(l.as_bool()? && r.as_bool()?)),
                    Token::Or => Ok(Types::Boolean(l.as_bool()? || r.as_bool()?)),

                    _ => Err(errors::A16.to_owned()),
                }
            }

            _ => Err(errors::A15.to_owned()),
        }
    }
}
