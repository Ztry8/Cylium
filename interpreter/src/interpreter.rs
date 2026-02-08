// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use crate::{
    bytecode::{Instruction, Node, Proc},
    errors,
    file_handler::FileHandler,
    scope::Scope,
    types::Types,
};

pub fn execute(handler: &FileHandler, program: HashMap<String, Proc>, const_nodes: Vec<Node>) {
    if let Some(main) = program.get("main") {
        let mut scope = Scope::new();
        let mut consts = Scope::new();
        let mut stack = Vec::new();

        consts.declare("PI".to_owned(), Types::Float(std::f64::consts::PI), true);
        consts.declare("E".to_owned(), Types::Float(std::f64::consts::E), true);

        for node in const_nodes {
            match node.instruction {
                Instruction::StoreConst(name) => {
                    if consts.exist(&name) {
                        handler.show_error(node.line_number, errors::A07);
                    } else {
                        consts.declare(
                            name,
                            match stack.pop() {
                                Some(value) => value,
                                None => handler.show_error(node.line_number, errors::A15),
                            },
                            true,
                        )
                    }
                }
                _ => {
                    if let Err(e) = expr_execute(&node.instruction, &mut stack) {
                        handler.show_error(node.line_number, &e);
                    }
                }
            }
        }

        stack.clear();

        let mut i = 0;
        while i < main.body.len() as i32 {
            let node = &main.body[i as usize];

            if let Err(e) = main_execute(
                &program,
                &node.instruction,
                &mut scope,
                &mut consts,
                &mut stack,
                &mut i,
            ) {
                handler.show_error(node.line_number, &e);
            };

            i += 1;
        }
    } else {
        handler.show_error(0, errors::A22);
    }
}

fn main_execute(
    program: &HashMap<String, Proc>,
    instruction: &Instruction,
    scope: &mut Scope,
    consts: &mut Scope,
    stack: &mut Vec<Types>,
    line_number: &mut i32,
) -> Result<(), String> {
    match instruction {
        Instruction::Jump(number) => {
            *line_number = *number as i32 - 1;
        }
        Instruction::JumpIfFalse(number) => match stack.pop() {
            Some(Types::Boolean(value)) => {
                if !value {
                    *line_number = *number as i32 - 1;
                }
            }
            _ => return Err(errors::A15.to_owned()),
        },
        Instruction::StoreConst(name) => {
            if consts.exist(name) {
                return Err(errors::A07.to_owned());
            } else {
                consts.declare(
                    name.to_owned(),
                    stack.pop().ok_or(errors::A15.to_owned())?,
                    true,
                )
            }
        }
        Instruction::StoreLocal(name) => scope.declare(
            name.to_owned(),
            stack.pop().ok_or(errors::A15.to_owned())?,
            false,
        ),
        Instruction::Push(value) => {
            stack.push(value.clone());
        }
        Instruction::Load(name) => stack.push(if name == "input" {
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            let mut buf = String::new();
            if std::io::stdin().read_line(&mut buf).is_err() {
                FileHandler::show_warning(errors::C02);
            }

            Types::String(buf.trim().to_string())
        } else {
            scope
                .get(consts, name)
                .ok_or(errors::A03.to_owned())?
                .0
                .clone()
        }),
        Instruction::Exit(code) => std::process::exit(*code),
        Instruction::Delete(name) => {
            if consts.exist(name) {
                return Err(errors::A28.to_owned());
            } else if !scope.remove(name) {
                return Err(errors::A03.to_owned());
            }
        }
        Instruction::Call(name, argc) => {
            if let Some(proc) = program.get(name) {
                let mut scope = Scope::new();

                for i in 0..*argc {
                    scope.declare(
                        proc.args[i].clone().0,
                        stack.pop().ok_or(errors::A27.to_owned())?,
                        false,
                    )
                }

                let mut stack = Vec::new();

                let mut i = 0;
                while i < proc.body.len() as i32 {
                    main_execute(
                        program,
                        &proc.body[i as usize].instruction,
                        &mut scope,
                        consts,
                        &mut stack,
                        &mut i,
                    )?;

                    i += 1;
                }
            } else {
                return Err(errors::A24.to_owned());
            }
        }
        Instruction::Echo => {
            if let Some(value) = stack.pop() {
                match value {
                    Types::String(value) => println!("{}", value),
                    Types::Boolean(value) => println!("{}", value),
                    Types::Number(value) => println!("{}", value),
                    Types::Float(value) => println!("{}", value),
                }
            } else {
                return Err(errors::A15.to_owned());
            }
        }
        _ => expr_execute(instruction, stack)?,
    }

    Ok(())
}

fn expr_execute(instruction: &Instruction, stack: &mut Vec<Types>) -> Result<(), String> {
    match instruction {
        Instruction::Neg => match stack.pop() {
            Some(Types::Number(value)) => stack.push(Types::Number(-value)),
            Some(Types::Float(value)) => stack.push(Types::Float(-value)),
            _ => unreachable!(),
        },
        Instruction::Not => match stack.pop() {
            Some(Types::Boolean(value)) => stack.push(Types::Boolean(!value)),
            _ => unreachable!(),
        },
        Instruction::Or => match (stack.pop(), stack.pop()) {
            (Some(Types::Boolean(a)), Some(Types::Boolean(b))) => {
                stack.push(Types::Boolean(a || b))
            }
            _ => unreachable!(),
        },
        Instruction::And => match (stack.pop(), stack.pop()) {
            (Some(Types::Boolean(a)), Some(Types::Boolean(b))) => {
                stack.push(Types::Boolean(a && b))
            }
            _ => unreachable!(),
        },
        Instruction::Equal => match (stack.pop(), stack.pop()) {
            (Some(Types::Boolean(a)), Some(Types::Boolean(b))) => {
                stack.push(Types::Boolean(a == b))
            }
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Boolean(a == b)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Boolean(a == b)),
            (Some(Types::String(a)), Some(Types::String(b))) => stack.push(Types::Boolean(a == b)),
            _ => unreachable!(),
        },
        Instruction::NotEqual => match (stack.pop(), stack.pop()) {
            (Some(Types::Boolean(a)), Some(Types::Boolean(b))) => {
                stack.push(Types::Boolean(a != b))
            }
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Boolean(a != b)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Boolean(a != b)),
            (Some(Types::String(a)), Some(Types::String(b))) => stack.push(Types::Boolean(a != b)),
            _ => unreachable!(),
        },
        Instruction::Greater => match (stack.pop(), stack.pop()) {
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Boolean(b > a)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Boolean(b > a)),
            (Some(Types::String(a)), Some(Types::String(b))) => stack.push(Types::Boolean(b > a)),
            _ => unreachable!(),
        },
        Instruction::Less => match (stack.pop(), stack.pop()) {
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Boolean(b < a)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Boolean(b < a)),
            (Some(Types::String(a)), Some(Types::String(b))) => stack.push(Types::Boolean(b < a)),
            _ => unreachable!(),
        },
        Instruction::GreaterEqual => match (stack.pop(), stack.pop()) {
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Boolean(b >= a)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Boolean(b >= a)),
            (Some(Types::String(a)), Some(Types::String(b))) => stack.push(Types::Boolean(b >= a)),
            _ => unreachable!(),
        },
        Instruction::LessEqual => match (stack.pop(), stack.pop()) {
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Boolean(b <= a)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Boolean(b <= a)),
            (Some(Types::String(a)), Some(Types::String(b))) => stack.push(Types::Boolean(b <= a)),
            _ => unreachable!(),
        },
        Instruction::Plus => match (stack.pop(), stack.pop()) {
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Number(a + b)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Float(a + b)),
            (Some(Types::String(mut a)), Some(Types::String(b))) => stack.push(Types::String({
                a.push_str(&b);
                a
            })),
            _ => unreachable!(),
        },
        Instruction::Minus => match (stack.pop(), stack.pop()) {
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Number(a - b)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Float(a - b)),
            _ => unreachable!(),
        },
        Instruction::Multiply => match (stack.pop(), stack.pop()) {
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Number(a * b)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Float(a * b)),
            (Some(Types::String(a)), Some(Types::Number(b))) => stack.push(Types::String({
                let mut t = String::new();
                for _ in 0..b {
                    t.push_str(&a);
                }
                t
            })),
            (Some(Types::Number(a)), Some(Types::String(b))) => stack.push(Types::String({
                let mut t = String::new();
                for _ in 0..a {
                    t.push_str(&b);
                }
                t
            })),
            _ => unreachable!(),
        },
        Instruction::Divide => match (stack.pop(), stack.pop()) {
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Number(a / b)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Float(a / b)),
            _ => unreachable!(),
        },
        Instruction::Mod => match (stack.pop(), stack.pop()) {
            (Some(Types::Number(a)), Some(Types::Number(b))) => stack.push(Types::Number(a % b)),
            (Some(Types::Float(a)), Some(Types::Float(b))) => stack.push(Types::Float(a % b)),
            _ => unreachable!(),
        },
        Instruction::CastToString => match stack.pop() {
            Some(Types::Number(value)) => stack.push(Types::String(value.to_string())),
            Some(Types::Float(value)) => stack.push(Types::String(value.to_string())),
            Some(Types::Boolean(value)) => stack.push(Types::String(if value {
                "true".to_owned()
            } else {
                "false".to_owned()
            })),
            _ => return Err(errors::A36.to_owned()),
        },
        Instruction::CastToNumber => match stack.pop() {
            Some(Types::String(value)) => stack.push(Types::Number(
                value.parse::<i64>().map_err(|_| errors::A02)?,
            )),
            Some(Types::Float(value)) => stack.push(Types::Number(value as i64)),
            Some(Types::Boolean(value)) => stack.push(Types::Number(if value { 1 } else { 0 })),
            _ => return Err(errors::A02.to_owned()),
        },
        Instruction::CastToFloat => match stack.pop() {
            Some(Types::String(value)) => {
                stack.push(Types::Float(value.parse::<f64>().map_err(|_| errors::A02)?))
            }
            Some(Types::Number(value)) => stack.push(Types::Float(value as f64)),
            Some(Types::Boolean(value)) => stack.push(Types::Float(if value { 1.0 } else { 0.0 })),
            _ => return Err(errors::A02.to_owned()),
        },
        Instruction::CastToBoolean => match stack.pop() {
            Some(Types::String(value)) => stack.push(Types::Boolean(match value.as_str() {
                "true" => true,
                "false" => false,
                _ => return Err(errors::A35.to_owned()),
            })),
            Some(Types::Number(value)) => stack.push(Types::Boolean(value == 1)),
            Some(Types::Float(value)) => stack.push(Types::Boolean(value == 1.0)),
            _ => return Err(errors::A35.to_owned()),
        },
        Instruction::Sin => match stack.pop() {
            Some(Types::Float(value)) => stack.push(Types::Float(value.sin())),
            _ => unreachable!(),
        },
        Instruction::Cos => match stack.pop() {
            Some(Types::Float(value)) => stack.push(Types::Float(value.cos())),
            _ => unreachable!(),
        },
        Instruction::Sqrt => match stack.pop() {
            Some(Types::Float(value)) => stack.push(Types::Float(value.sqrt())),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }

    Ok(())
}
