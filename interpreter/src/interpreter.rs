// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::{
    errors,
    file_handler::FileHandler,
    lexer::Token,
    parser::{AstKind, AstNode, ElseBlock},
    scope::Scope,
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

pub struct Interpreter {
    procs: HashMap<String, (Vec<String>, Vec<AstNode>)>,
    handler: FileHandler,
    consts: Scope,
}

impl Interpreter {
    pub fn new(handler: FileHandler, ast: &[AstNode]) -> Self {
        let mut procs = HashMap::new();

        let mut consts_frame = Scope::new();

        consts_frame.declare("PI".to_owned(), Types::Float(std::f64::consts::PI), true);

        consts_frame.declare("TAU".to_owned(), Types::Float(std::f64::consts::TAU), true);

        consts_frame.declare("E".to_owned(), Types::Float(std::f64::consts::E), true);

        consts_frame.declare(
            "SQRT_2".to_owned(),
            Types::Float(std::f64::consts::SQRT_2),
            true,
        );

        let dummy = Self {
            procs: HashMap::new(),
            handler: FileHandler::new(Vec::new()),
            consts: Scope::new(),
        };

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
                    let value = dummy
                        .eval(value, &consts_frame)
                        .unwrap_or_else(|e| handler.show_error(node.line, &e));

                    consts_frame.declare(name.to_string(), value, true);
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
            consts: consts_frame,
            procs,
        }
    }

    #[inline(always)]
    fn get_vector(&self, scope: &Scope, name: &str) -> Result<(String, VectorIndex), String> {
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
                    if let Some((Types::Number(index), _)) = scope.get(&self.consts, &index) {
                        Ok((name, VectorIndex::Index((*index) as usize)))
                    } else {
                        Err(errors::A17.to_owned())
                    }
                }
            }
        }
    }

    pub fn run(&self) {
        let (_, body) = self.procs.get("main").unwrap();
        let mut frame = Scope::new();

        for stmt in body {
            if let Err(e) = self.exec(stmt, &mut frame) {
                self.handler.show_error(stmt.line, &e);
            }
        }
    }

    fn exec(&self, node: &AstNode, scope: &mut Scope) -> Result<(), String> {
        match &node.kind {
            AstKind::VarDecl {
                name,
                value,
                is_const,
            } => {
                let v = match *value.clone() {
                    AstKind::Ident(n) if n == "input" => {
                        io::stdout().flush().unwrap();
                        let mut buf = String::new();
                        if io::stdin().read_line(&mut buf).is_err() {
                            FileHandler::show_warning(errors::C02);
                        }

                        Types::String(buf.trim().to_string())
                    }
                    _ => self.eval(value, scope)?,
                };

                scope.declare(name.to_owned(), v, *is_const);
            }

            AstKind::Delete(name) => {
                if let Some((_, is_const)) = scope.remove(name) {
                    if is_const {
                        return Err(errors::A28.to_owned());
                    }
                } else {
                    return Err(errors::A03.to_owned());
                }
            }

            AstKind::Assign { name, op, expr } => {
                let rhs = self.eval(expr, scope)?;

                let (cur, is_const) = if name.contains('[') {
                    let (name, index) = self.get_vector(scope, name)?;

                    if let Some((Types::Vector(vec), is_const)) = scope.get(&self.consts, &name) {
                        match index {
                            VectorIndex::Index(index) => {
                                (vec.get(index).ok_or(errors::A17.to_owned())?, *is_const)
                            }
                            _ => return Err(errors::A15.to_owned()),
                        }
                    } else {
                        return Err(errors::A30.to_owned());
                    }
                } else {
                    let (value, is_const) = scope.get(&self.consts, name).ok_or(errors::A03)?;
                    (value, *is_const)
                };

                if is_const {
                    return Err(errors::A07.to_owned());
                }

                let new = match op {
                    Token::Assign => rhs,
                    Token::PlusAssign => cur.clone().add(&rhs)?,
                    Token::MinusAssign => cur.sub(&rhs)?,
                    Token::MultiplyAssign => cur.mul(&rhs)?,
                    Token::DivideAssign => cur.div(&rhs)?,
                    Token::ModAssign => cur.rem(&rhs)?,
                    _ => return Err(errors::A15.to_owned()),
                };

                scope.declare(name.to_owned(), new, false);
            }

            AstKind::Echo(expr) => {
                let v = self.eval(expr, scope)?;

                println!("{}", v);
            }

            AstKind::Exit(code) => std::process::exit(*code),

            AstKind::MethodCall {
                receiver,
                name,
                args,
            } => {}

            AstKind::ProcCall { name, args } => {
                let (params, body) = self.procs.get(name).ok_or(errors::A24)?;

                if params.len() != args.len() {
                    return Err(errors::A27.to_owned());
                }

                let mut new_scope = Scope::new();

                for (i, p) in params.iter().enumerate() {
                    let val = match &args[i] {
                        Token::Value(v) => v.clone(),
                        Token::Ident(n) => scope.get(&self.consts, n).ok_or(errors::A03)?.0.clone(),
                        _ => return Err(errors::A15.to_owned()),
                    };

                    new_scope.declare(p.clone(), val.clone(), false);
                }

                for stmt in body {
                    if let Err(e) = self.exec(&stmt, &mut new_scope) {
                        self.handler.show_error(node.line, &e);
                    }
                }
            }

            AstKind::While { expr, body } => {
                while self.eval(expr, scope)?.as_bool()? {
                    for stmt in body {
                        self.exec(stmt, scope)?;
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
                let start = self.eval(start, scope)?;
                let end = self.eval(end, scope)?;

                let step = self.eval(
                    &step.clone().unwrap_or_else(|| {
                        if matches!(end, Types::Float(_)) {
                            if start < end {
                                AstKind::Value(Types::Float(1.0))
                            } else {
                                AstKind::Value(Types::Float(-1.0))
                            }
                        } else if start < end {
                            AstKind::Value(Types::Number(1))
                        } else {
                            AstKind::Value(Types::Number(-1))
                        }
                    }),
                    scope,
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

                scope.declare(var_name.clone(), start.clone(), false);

                loop {
                    if let Some((counter, _)) = scope.get(&self.consts, var_name).cloned() {
                        if if start < end {
                            counter < end
                        } else {
                            counter > end
                        } {
                            for stmt in body {
                                self.exec(stmt, scope)?;
                            }

                            let var = scope.get_mut(var_name).ok_or(errors::A32.to_owned())?;

                            *var = counter.add(&step)?;
                        } else {
                            break;
                        }
                    } else {
                        return Err(errors::A32.to_owned());
                    }
                }
            }

            AstKind::Condition { expr, yes, no } => {
                if self.eval(expr, scope)?.as_bool()? {
                    for s in yes {
                        self.exec(s, scope)?;
                    }
                } else if let Some(n) = no {
                    match n {
                        ElseBlock::Else(b) => {
                            for s in b {
                                self.exec(s, scope)?;
                            }
                        }
                        ElseBlock::ElseIf(n) => self.exec(n, scope)?,
                    }
                }
            }

            _ => return Err(errors::A15.to_owned()),
        }

        Ok(())
    }

    fn eval(&self, expr: &AstKind, scope: &Scope) -> Result<Types, String> {
        match expr {
            AstKind::Value(v) => Ok(v.clone()),

            AstKind::Ident(n) => {
                if matches!(
                    n.as_str(),
                    "number" | "float" | "bool" | "string" | "sqrt" | "cos" | "sin"
                ) {
                    Ok(Types::String(n.clone()))
                } else if n.as_str() == "vector" {
                    Ok(Types::Vector(Vec::new()))
                } else if n.contains('[') {
                    let (name, index) = self.get_vector(scope, n)?;

                    if let Some((Types::Vector(vec), _)) = scope.get(&self.consts, &name) {
                        match index {
                            VectorIndex::Index(index) => {
                                vec.get(index).ok_or(errors::A17.to_owned()).cloned()
                            }
                            VectorIndex::Length => Ok(Types::Number(vec.len() as i64)),
                            VectorIndex::Capacity => Ok(Types::Number(vec.capacity() as i64)),
                        }
                    } else {
                        Err(errors::A30.to_owned())
                    }
                } else {
                    Ok(scope
                        .get(&self.consts, n)
                        .cloned()
                        .ok_or(errors::A03.to_owned())?
                        .0)
                }
            }

            AstKind::UnaryOp { op, expr } => {
                let v = self.eval(expr, scope)?;

                match op {
                    Token::Not => Ok(Types::Boolean(!v.as_bool()?)),
                    Token::Minus => Ok(Types::Number(-v.as_number()?)),
                    _ => Err(errors::A15.to_owned()),
                }
            }

            AstKind::BinaryOp { left, op, right } => {
                let mut l = self.eval(left, scope)?;
                let r = self.eval(right, scope)?;

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

                    Token::Plus => l.add(&r),
                    Token::Minus => l.sub(&r),
                    Token::Multiply => l.mul(&r),
                    Token::Divide => l.div(&r),
                    Token::Mod => l.rem(&r),

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
