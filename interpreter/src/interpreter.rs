// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use crate::{
    bytecode::{Func, Instruction, Node},
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

pub fn execute(handler: &FileHandler, program: HashMap<String, Func>, const_nodes: Vec<Node>) {
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
                    handler.show_error(node.line_number, errors::A37);
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

    let code = match run_body(
        &program,
        &main.body,
        &mut scope,
        &mut consts,
        &mut stack,
        handler,
    ) {
        Some(Types::Scalar(Scalar::Number(n))) => n as i32,
        _ => 0,
    };

    std::process::exit(code);
}

fn run_body(
    program: &HashMap<String, Func>,
    body: &[Node],
    scope: &mut Scope,
    consts: &mut Scope,
    stack: &mut Vec<Types>,
    handler: &FileHandler,
) -> Option<Types> {
    let len = body.len();
    let mut ip: usize = 0;

    while ip < len {
        let node = &body[ip];
        match dispatch(
            handler,
            program,
            &node.instruction,
            scope,
            consts,
            stack,
            &mut ip,
        ) {
            Ok(None) => {}
            Ok(Some(val)) => return Some(val),
            Err(e) => handler.show_error(node.line_number, &e),
        }

        ip += 1;
    }

    None
}

#[inline(always)]
fn dispatch(
    handler: &FileHandler,
    program: &HashMap<String, Func>,
    instr: &Instruction,
    scope: &mut Scope,
    consts: &mut Scope,
    stack: &mut Vec<Types>,
    ip: &mut usize,
) -> Result<Option<Types>, String> {
    match instr {
        Instruction::Jump(target) => {
            *ip = target.wrapping_sub(1);
        }
        Instruction::JumpIfFalse(target) => {
            if !pop_bool(stack) {
                *ip = target.wrapping_sub(1);
            }
        }

        Instruction::Pop => {
            stack.pop();
        }

        Instruction::Return => return Ok(Some(Types::Void)),

        Instruction::ReturnValue => {
            let val = stack.pop().ok_or_else(|| errors::A15.to_owned())?;

            return Ok(Some(val));
        }

        Instruction::CallFunc(name, argc) => {
            match name.as_str() {
                "input" => {
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    let mut buf = String::new();
                    if std::io::stdin().read_line(&mut buf).is_err() {
                        FileHandler::show_warning(errors::C02);
                    }
                    stack.push(Types::String(buf.trim().to_string()));
                    return Ok(None);
                }
                "sin" => {
                    let a = pop_float(stack);
                    stack.push(flt(a.sin()));
                    return Ok(None);
                }
                "cos" => {
                    let a = pop_float(stack);
                    stack.push(flt(a.cos()));
                    return Ok(None);
                }
                "sqrt" => {
                    let a = pop_float(stack);
                    if a >= 0.0 {
                        stack.push(flt(a.sqrt()));
                    } else {
                        return Err(errors::A45.to_owned());
                    }
                    return Ok(None);
                }
                "len" => {
                    match stack.pop() {
                        Some(Types::Array(arr)) => {
                            stack.push(int(arr.len() as i64));
                        }
                        _ => return Err(errors::A48.to_owned()),
                    }

                    return Ok(None);
                }
                _ => {}
            }

            let func = program.get(name).ok_or_else(|| errors::A24.to_owned())?;

            let mut func_scope = Scope::new();
            for i in 0..*argc {
                func_scope.declare(
                    func.args[i].0.clone(),
                    stack.pop().ok_or_else(|| errors::A27.to_owned())?,
                    false,
                );
            }

            let mut func_stack: Vec<Types> = Vec::with_capacity(32);
            if let Some(val) = run_body(
                program,
                &func.body,
                &mut func_scope,
                consts,
                &mut func_stack,
                handler,
            ) && !matches!(val, Types::Void)
            {
                stack.push(val);
            }
        }

        Instruction::StoreLocal(name) => {
            let val = stack.pop().ok_or_else(|| errors::A15.to_owned())?;
            if let Some(slot) = scope.get_mut(name) {
                *slot = val;
            } else {
                scope.declare(name.to_string(), val, false);
            }
        }

        Instruction::StoreConst(name) => {
            if consts.exist(name) {
                return Err(errors::A37.to_owned());
            }

            consts.declare(
                name.to_owned(),
                stack.pop().ok_or_else(|| errors::A15.to_owned())?,
                true,
            );
        }

        Instruction::Load(name) => {
            stack.push(
                scope
                    .get(consts, name)
                    .ok_or_else(|| errors::A03.to_owned())?
                    .0
                    .clone(),
            );
        }

        Instruction::NewArray(name, _elem_type) => {
            let count = pop_int(stack) as usize;
            let mut arr: Vec<Types> = Vec::with_capacity(count);

            for _ in 0..count {
                arr.push(stack.pop().ok_or_else(|| errors::A15.to_owned())?);
            }

            arr.reverse();
            if let Some(slot) = scope.get_mut(name) {
                *slot = Types::Array(arr);
            } else {
                scope.declare(name.to_string(), Types::Array(arr), false);
            }
        }

        Instruction::ArrayStore(name) => {
            let index = pop_int(stack) as usize; 
            let value = stack.pop().ok_or_else(|| errors::A15.to_owned())?;

            match scope.get_mut(name) {
                Some(Types::Array(arr)) => {
                    if index >= arr.len() {
                        return Err(errors::A17.to_owned());
                    }
                    arr[index] = value;
                }
                _ => return Err(errors::A48.to_owned()),
            }
        }

        Instruction::ArrayLoadDeep(name, depth) => {
            let mut indices = Vec::with_capacity(*depth);
            for _ in 0..*depth {
                indices.push(pop_int(stack) as usize);
            }

            indices.reverse(); 

            let base = match scope.get(consts, name) {
                Some((v, _)) => v.clone(),
                None => return Err(errors::A03.to_owned()),
            };

            let mut cur = base;
            for idx in indices {
                match cur {
                    Types::Array(arr) => {
                        if idx >= arr.len() {
                            return Err(errors::A17.to_owned());
                        }
                        cur = arr[idx].clone();
                    }
                    _ => return Err(errors::A48.to_owned()),
                }
            }

            stack.push(cur);
        }

        Instruction::ArrayStoreDeep(name, depth) => {
            let mut indices = Vec::with_capacity(*depth);
            for _ in 0..*depth {
                indices.push(pop_int(stack) as usize);
            }

            indices.reverse();
            let value = stack.pop().ok_or_else(|| errors::A15.to_owned())?;

            fn set_deep(
                arr: &mut Vec<Types>,
                indices: &[usize],
                value: Types,
            ) -> Result<(), String> {
                let idx = indices[0];
                if idx >= arr.len() {
                    return Err(errors::A17.to_owned());
                }
                if indices.len() == 1 {
                    arr[idx] = value;
                } else {
                    match &mut arr[idx] {
                        Types::Array(inner) => set_deep(inner, &indices[1..], value)?,
                        _ => return Err(errors::A48.to_owned()),
                    }
                }
                Ok(())
            }

            match scope.get_mut(name) {
                Some(Types::Array(arr)) => set_deep(arr, &indices, value)?,
                _ => return Err(errors::A48.to_owned()),
            }
        }

        Instruction::ArrayLen(name) => match scope.get(consts, name) {
            Some((Types::Array(arr), _)) => {
                stack.push(int(arr.len() as i64));
            }
            _ => return Err(errors::A48.to_owned()),
        },

        Instruction::NewArrayLit(n) => {
            let mut arr = Vec::with_capacity(*n);
            for _ in 0..*n {
                arr.push(stack.pop().ok_or_else(|| errors::A15.to_owned())?);
            }

            arr.reverse();
            stack.push(Types::Array(arr));
        }

        Instruction::NewArrayFill => {
            let fill = stack.pop().ok_or_else(|| errors::A15.to_owned())?;
            let size = pop_int(stack) as usize;
            let arr: Vec<Types> = (0..size).map(|_| fill.clone()).collect();
            stack.push(Types::Array(arr));
        }

        Instruction::ArrayLoad(name) => {
            let index = pop_int(stack) as usize;
            let val = match scope.get(consts, name) {
                Some((Types::Array(arr), _)) => {
                    if index >= arr.len() {
                        return Err(errors::A17.to_owned());
                    }
                    arr[index].clone()
                }
                _ => return Err(errors::A48.to_owned()),
            };
            
            stack.push(val);
        }

        Instruction::PushInt(v) => stack.push(int(*v)),
        Instruction::PushFloat(v) => stack.push(flt(*v)),
        Instruction::PushBool(v) => stack.push(bl(*v)),
        Instruction::PushStr(v) => stack.push(str_(v.clone())),

        Instruction::AndInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a & b));
        }
        Instruction::OrInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a | b));
        }
        Instruction::XorInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a ^ b));
        }
        Instruction::NotInt => {
            let a = pop_int(stack);
            stack.push(int(!a));
        }
        Instruction::RightInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a >> b));
        }
        Instruction::LeftInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);
            stack.push(int(a << b));
        }
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

            if b == 0 {
                return Err(errors::A44.to_owned());
            } else {
                stack.push(int(a / b));
            }
        }
        Instruction::ModInt => {
            let a = pop_int(stack);
            let b = pop_int(stack);

            if b == 0 {
                return Err(errors::A44.to_owned());
            } else {
                stack.push(int(a % b));
            }
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

            if b == 0.0 {
                return Err(errors::A44.to_owned());
            } else {
                stack.push(flt(a / b));
            }
        }
        Instruction::ModFloat => {
            let a = pop_float(stack);
            let b = pop_float(stack);

            if b == 0.0 {
                return Err(errors::A44.to_owned());
            } else {
                stack.push(flt(a % b));
            }
        }
        Instruction::NegFloat => {
            let a = pop_float(stack);
            stack.push(flt(-a));
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
            stack.push(bl(v != 0));
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
            stack.push(bl(v != 0.0));
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
            _ => return Err(errors::A15.to_owned()),
        },

        Instruction::Exit(code) => std::process::exit(*code),
        Instruction::Delete(name) => {
            if consts.exist(name) {
                return Err(errors::A28.to_owned());
            } else if !scope.remove(name) {
                return Err(errors::A03.to_owned());
            }
        }
    }

    Ok(None)
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
    .map(|_| ())
}
