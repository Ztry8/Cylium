// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use crate::{
    bytecode::{Instruction, Node, Proc},
    errors,
    file_handler::FileHandler,
    scope::Scope,
    types::{Scalar, Types},
};

#[inline(always)]
fn int(v: i64) -> Types {
    Types::Scalar(Scalar::Number(v))
}
#[inline(always)]
fn flt(v: f64) -> Types {
    Types::Scalar(Scalar::Float(v))
}
#[inline(always)]
fn bl(v: bool) -> Types {
    Types::Scalar(Scalar::Boolean(v))
}
#[inline(always)]
fn str_(v: String) -> Types {
    Types::String(v)
}

#[inline(always)]
fn pop_int(stack: &mut Vec<Types>) -> i64 {
    match stack.pop() {
        Some(Types::Scalar(Scalar::Number(v))) => v,
        _ => unreachable!(),
    }
}

#[inline(always)]
fn pop_float(stack: &mut Vec<Types>) -> f64 {
    match stack.pop() {
        Some(Types::Scalar(Scalar::Float(v))) => v,
        _ => unreachable!(),
    }
}

#[inline(always)]
fn pop_bool(stack: &mut Vec<Types>) -> bool {
    match stack.pop() {
        Some(Types::Scalar(Scalar::Boolean(v))) => v,
        _ => unreachable!(),
    }
}

#[inline(always)]
fn pop_str(stack: &mut Vec<Types>) -> String {
    match stack.pop() {
        Some(Types::String(v)) => v,
        _ => unreachable!(),
    }
}

pub fn execute(handler: &FileHandler, program: HashMap<String, Proc>, const_nodes: Vec<Node>) {
    let Some(main) = program.get("main") else {
        handler.show_error(0, errors::A22);
    };

    let mut scope = Scope::new();
    let mut consts = Scope::new();
    let mut stack: Vec<Types> = Vec::with_capacity(64);

    consts.declare("PI".to_owned(), flt(std::f64::consts::PI), true);
    consts.declare("E".to_owned(), flt(std::f64::consts::E), true);

    for node in const_nodes {
        match node.instruction {
            Instruction::StoreConst(name) => {
                if consts.exist(&name) {
                    handler.show_error(node.line_number, errors::A07);
                }
                consts.declare(
                    name,
                    stack
                        .pop()
                        .unwrap_or_else(|| handler.show_error(node.line_number, errors::A15)),
                    true,
                );
            }
            _ => {
                if let Err(e) = expr_execute(handler, &node.instruction, &mut stack) {
                    handler.show_error(node.line_number, &e);
                }
            }
        }
    }
    stack.clear();

    run_body(
        &program,
        &main.body,
        &mut scope,
        &mut consts,
        &mut stack,
        handler,
    );
}

fn run_body(
    program: &HashMap<String, Proc>,
    body: &[Node],
    scope: &mut Scope,
    consts: &mut Scope,
    stack: &mut Vec<Types>,
    handler: &FileHandler,
) {
    let len = body.len();
    let mut ip: usize = 0;

    while ip < len {
        let node = &body[ip];
        if let Err(e) = dispatch(
            handler,
            program,
            &node.instruction,
            scope,
            consts,
            stack,
            &mut ip,
        ) {
            handler.show_error(node.line_number, &e);
        }

        ip += 1;
    }
}

#[inline(always)]
fn dispatch(
    handler: &FileHandler,
    program: &HashMap<String, Proc>,
    instr: &Instruction,
    scope: &mut Scope,
    consts: &mut Scope,
    stack: &mut Vec<Types>,
    ip: &mut usize,
) -> Result<(), String> {
    match instr {
        Instruction::Jump(target) => {
            *ip = target.wrapping_sub(1);
        }
        Instruction::JumpIfFalse(target) => {
            if !pop_bool(stack) {
                *ip = target.wrapping_sub(1);
            }
        }

        Instruction::StoreLocal(name) => {
            let val = stack.pop().ok_or_else(|| errors::A15.to_owned())?;
            scope.declare(name.to_string(), val, false);
        }

        Instruction::StoreConst(name) => {
            if consts.exist(name) {
                return Err(errors::A07.to_owned());
            }

            consts.declare(
                name.to_owned(),
                stack.pop().ok_or_else(|| errors::A15.to_owned())?,
                true,
            );
        }

        Instruction::Load(name) => {
            stack.push(if name == "input" {
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                let mut buf = String::new();

                if std::io::stdin().read_line(&mut buf).is_err() {
                    FileHandler::show_warning(errors::C02);
                }

                Types::String(buf.trim().to_string())
            } else {
                scope
                    .get(consts, name)
                    .ok_or_else(|| errors::A03.to_owned())?
                    .0
                    .clone()
            });
        }

        Instruction::PushInt(v) => stack.push(int(*v)),
        Instruction::PushFloat(v) => stack.push(flt(*v)),
        Instruction::PushBool(v) => stack.push(bl(*v)),
        Instruction::PushStr(v) => stack.push(str_(v.clone())),

        Instruction::AddInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a + b));
        }
        Instruction::SubInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a - b));
        }
        Instruction::MulInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a * b));
        }
        Instruction::DivInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a / b));
        }
        Instruction::ModInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a % b));
        }
        Instruction::NegInt => {
            let a = pop_int(stack);
            stack.push(int(-a));
        }

        Instruction::AddFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(flt(a + b));
        }
        Instruction::SubFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(flt(a - b));
        }
        Instruction::MulFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(flt(a * b));
        }
        Instruction::DivFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(flt(a / b));
        }
        Instruction::ModFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(flt(a % b));
        }
        Instruction::NegFloat => {
            let a = pop_float(stack);
            stack.push(flt(-a));
        }
        Instruction::SinFloat => {
            let a = pop_float(stack);
            stack.push(flt(a.sin()));
        }
        Instruction::CosFloat => {
            let a = pop_float(stack);
            stack.push(flt(a.cos()));
        }
        Instruction::SqrtFloat => {
            let a = pop_float(stack);
            stack.push(flt(a.sqrt()));
        }

        Instruction::ConcatStr => {
            let b = pop_str(stack);
            let a = pop_str(stack);
            stack.push(str_(a + &b));
        }
        Instruction::ConcatStrInt => {
            let a = pop_str(stack);
            let b = pop_int(stack);
            stack.push(str_(a + &b.to_string()));
        }
        Instruction::ConcatStrFloat => {
            let a = pop_str(stack);
            let b = pop_float(stack);
            stack.push(str_(a + &b.to_string()));
        }
        Instruction::ConcatIntStr => {
            let a = pop_int(stack);
            let b = pop_str(stack);
            stack.push(str_(a.to_string() + &b));
        }
        Instruction::ConcatFloatStr => {
            let a = pop_float(stack);
            let b = pop_str(stack);
            stack.push(str_(a.to_string() + &b));
        }
        Instruction::RepeatStr => {
            let a = pop_str(stack);
            let b = pop_int(stack);
            stack.push(str_(a.repeat(b as usize)));
        }
        Instruction::RepeatStrRev => {
            let a = pop_int(stack);
            let b = pop_str(stack);
            stack.push(str_(b.repeat(a as usize)));
        }

        Instruction::EqInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(bl(a == b));
        }
        Instruction::NeInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(bl(a != b));
        }
        Instruction::GtInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(bl(a > b));
        }
        Instruction::LtInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(bl(a < b));
        }
        Instruction::GeInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(bl(a >= b));
        }
        Instruction::LeInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(bl(a <= b));
        }

        Instruction::EqFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(bl(a == b));
        }
        Instruction::NeFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(bl(a != b));
        }
        Instruction::GtFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(bl(a > b));
        }
        Instruction::LtFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(bl(a < b));
        }
        Instruction::GeFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(bl(a >= b));
        }
        Instruction::LeFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);
            stack.push(bl(a <= b));
        }

        Instruction::EqBool => {
            let a = pop_bool(stack);
            let b = pop_bool(stack);
            stack.push(bl(a == b));
        }
        Instruction::NeBool => {
            let a = pop_bool(stack);
            let b = pop_bool(stack);
            stack.push(bl(a != b));
        }
        Instruction::AndBool => {
            let a = pop_bool(stack);
            let b = pop_bool(stack);
            stack.push(bl(a && b));
        }
        Instruction::OrBool => {
            let a = pop_bool(stack);
            let b = pop_bool(stack);
            stack.push(bl(a || b));
        }
        Instruction::NotBool => {
            let a = pop_bool(stack);
            stack.push(bl(!a));
        }

        Instruction::EqStr => {
            let a = pop_str(stack);
            let b = pop_str(stack);
            stack.push(bl(a == b));
        }
        Instruction::NeStr => {
            let a = pop_str(stack);
            let b = pop_str(stack);
            stack.push(bl(a != b));
        }
        Instruction::GtStr => {
            let a = pop_str(stack);
            let b = pop_str(stack);
            stack.push(bl(a > b));
        }
        Instruction::LtStr => {
            let a = pop_str(stack);
            let b = pop_str(stack);
            stack.push(bl(a < b));
        }
        Instruction::GeStr => {
            let a = pop_str(stack);
            let b = pop_str(stack);
            stack.push(bl(a >= b));
        }
        Instruction::LeStr => {
            let a = pop_str(stack);
            let b = pop_str(stack);
            stack.push(bl(a <= b));
        }

        Instruction::IntToFloat => {
            let v = pop_int(stack);
            stack.push(flt(v as f64));
        }
        Instruction::IntToBool => {
            let v = pop_int(stack);
            stack.push(bl(v == 1));
        }
        Instruction::IntToStr => {
            let v = pop_int(stack);
            stack.push(str_(v.to_string()));
        }
        Instruction::FloatToInt => {
            let v = pop_float(stack);
            stack.push(int(v as i64));
        }
        Instruction::FloatToBool => {
            let v = pop_float(stack);
            stack.push(bl(v == 1.0));
        }
        Instruction::FloatToStr => {
            let v = pop_float(stack);
            stack.push(str_(v.to_string()));
        }
        Instruction::BoolToInt => {
            let v = pop_bool(stack);
            stack.push(int(if v { 1 } else { 0 }));
        }
        Instruction::BoolToFloat => {
            let v = pop_bool(stack);
            stack.push(flt(if v { 1.0 } else { 0.0 }));
        }
        Instruction::BoolToStr => {
            let v = pop_bool(stack);
            stack.push(str_(if v {
                "true".to_owned()
            } else {
                "false".to_owned()
            }));
        }
        Instruction::StrToInt => {
            let v = pop_str(stack);
            stack.push(int(v.parse::<i64>().map_err(|_| errors::A02.to_owned())?));
        }
        Instruction::StrToFloat => {
            let v = pop_str(stack);
            stack.push(flt(v.parse::<f64>().map_err(|_| errors::A02.to_owned())?));
        }
        Instruction::StrToBool => {
            let v = pop_str(stack);
            stack.push(bl(match v.as_str() {
                "true" => true,
                "false" => false,
                _ => return Err(errors::A35.to_owned()),
            }));
        }

        Instruction::Echo => match stack.pop() {
            Some(Types::Scalar(Scalar::Number(v))) => println!("{}", v),
            Some(Types::Scalar(Scalar::Float(v))) => println!("{}", v),
            Some(Types::Scalar(Scalar::Boolean(v))) => println!("{}", v),
            Some(Types::String(v)) => println!("{}", v),
            None => return Err(errors::A15.to_owned()),
        },

        Instruction::Exit(code) => std::process::exit(*code),
        Instruction::Delete(name) => {
            if consts.exist(name) {
                return Err(errors::A28.to_owned());
            } else if !scope.remove(name) {
                return Err(errors::A03.to_owned());
            }
        }

        Instruction::Call(name, argc) => {
            let proc = program.get(name).ok_or_else(|| errors::A24.to_owned())?;
            let mut proc_scope = Scope::new();
            for i in 0..*argc {
                proc_scope.declare(
                    proc.args[i].0.clone(),
                    stack.pop().ok_or_else(|| errors::A27.to_owned())?,
                    false,
                );
            }

            let mut proc_stack: Vec<Types> = Vec::with_capacity(32);
            run_body(
                program,
                &proc.body,
                &mut proc_scope,
                consts,
                &mut proc_stack,
                handler,
            );
        }
    }
    Ok(())
}

fn expr_execute(
    handler: &FileHandler,
    instruction: &Instruction,
    stack: &mut Vec<Types>,
) -> Result<(), String> {
    dispatch(
        handler,
        &HashMap::new(),
        instruction,
        &mut Scope::new(),
        &mut Scope::new(),
        stack,
        &mut 0,
    )
}
