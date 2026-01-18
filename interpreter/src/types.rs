// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
};

use crate::{errors, show_warning};

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Types {
    Vector(Vec<Types>),
    String(String),
    Boolean(bool),
    Number(i32),
    Float(f32),
}

impl Types {
    pub fn create(value: &str) -> Self {
        if value.contains('.') {
            if let Ok(float) = value.parse::<f32>() {
                Self::Float(float)
            } else {
                Self::String(value.to_owned())
            }
        } else if let Ok(boolean) = value.parse::<bool>() {
            Self::Boolean(boolean)
        } else if let Ok(number) = value.parse::<i32>() {
            Self::Number(number)
        } else if value.contains(',') {
            Self::Vector(
                value
                    .split(',')
                    .map(|member| Self::create(member.trim()))
                    .collect(),
            )
        } else {
            Self::String(value.to_owned())
        }
    }

    pub fn as_number(&self) -> i32 {
        match self {
            Self::Number(value) => *value,
            _ => panic!(),
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            Self::Float(value) => *value,
            _ => panic!(),
        }
    }

    pub fn as_string(&self) -> &str {
        match self {
            Self::String(value) => value,
            _ => panic!(),
        }
    }

    pub fn convert_to_string(&mut self, line_number: usize, line: &str) {
        if let Self::Vector(value) = self {
            value
                .iter_mut()
                .for_each(|member| member.convert_to_string(line_number, line));
        } else {
            *self = Self::String(match self {
                Self::String(value) => {
                    show_warning(line_number, line, errors::C01);
                    value.to_owned()
                }
                Self::Boolean(value) => {
                    if *value {
                        "true".to_owned()
                    } else {
                        "false".to_owned()
                    }
                }
                Self::Number(value) => value.to_string(),
                Self::Float(value) => value.to_string(),
                _ => unreachable!(),
            });
        }
    }

    pub fn convert_to_bool(&mut self, line_number: usize, line: &str) {
        if let Self::Vector(value) = self {
            value
                .iter_mut()
                .for_each(|member| member.convert_to_bool(line_number, line));
        } else {
            *self = Self::Boolean(match self {
                Self::String(value) => value.parse::<bool>().unwrap(),
                Self::Boolean(value) => {
                    show_warning(line_number, line, errors::C01);
                    *value
                }
                Self::Number(value) => *value != 0,
                Self::Float(value) => *value != 0.0,
                _ => unreachable!(),
            });
        }
    }

    pub fn convert_to_vector(&mut self) {
        *self = Self::Vector(match self {
            Self::String(value) => value
                .chars()
                .map(|sym| Self::create(&sym.to_string()))
                .collect(),
            _ => panic!(),
        });
    }

    pub fn convert_to_number(&mut self, line_number: usize, line: &str) {
        if let Self::Vector(value) = self {
            value
                .iter_mut()
                .for_each(|member| member.convert_to_number(line_number, line));
        } else {
            *self = Self::Number(match self {
                Self::String(value) => dbg!(value).parse::<i32>().unwrap(),
                Self::Boolean(value) => {
                    if *value {
                        1
                    } else {
                        0
                    }
                }
                Self::Number(value) => {
                    show_warning(line_number, line, errors::C01);
                    *value
                }
                Self::Float(value) => value.round() as i32,
                _ => unreachable!(),
            });
        }
    }

    pub fn convert_to_float(&mut self, line_number: usize, line: &str) {
        if let Self::Vector(value) = self {
            value
                .iter_mut()
                .for_each(|member| member.convert_to_float(line_number, line));
        } else {
            *self = Self::Float(match self {
                Self::String(value) => value.parse::<f32>().unwrap(),
                Self::Boolean(value) => {
                    if *value {
                        1.0
                    } else {
                        0.0
                    }
                }
                Self::Number(value) => *value as f32,
                Self::Float(value) => {
                    show_warning(line_number, line, errors::C01);
                    *value
                }
                _ => unreachable!(),
            })
        }
    }
}

impl Display for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::String(value) => value.clone(),
                Self::Boolean(value) => value.to_string(),
                Self::Number(value) => value.to_string(),
                Self::Float(value) => value.to_string(),
                Self::Vector(value) => {
                    value
                        .iter()
                        .map(|member| format!("{}, ", member))
                        .collect::<Vec<String>>()
                        .concat()
                }
            }
        )
    }
}

impl RemAssign for Types {
    fn rem_assign(&mut self, rhs: Self) {
        match self {
            Self::Number(value) => *value %= rhs.as_number(),
            _ => panic!(),
        }
    }
}

impl DivAssign for Types {
    fn div_assign(&mut self, rhs: Self) {
        match self {
            Self::Number(value) => *value /= rhs.as_number(),
            Self::Float(value) => *value /= rhs.as_float(),
            _ => panic!(),
        }
    }
}

impl MulAssign for Types {
    fn mul_assign(&mut self, rhs: Self) {
        match self {
            Self::String(value) => {
                let mut result = String::new();

                for _ in 0..rhs.as_number() {
                    result.push_str(value);
                }

                *value = result;
            }
            Self::Number(value) => *value *= rhs.as_number(),
            Self::Float(value) => *value *= rhs.as_float(),
            _ => panic!(),
        }
    }
}

impl AddAssign for Types {
    fn add_assign(&mut self, rhs: Self) {
        match self {
            Self::String(value) => value.push_str(rhs.as_string()),
            Self::Number(value) => *value += rhs.as_number(),
            Self::Float(value) => *value += rhs.as_float(),
            _ => panic!(),
        }
    }
}

impl SubAssign for Types {
    fn sub_assign(&mut self, rhs: Self) {
        match self {
            Self::Number(value) => *value -= rhs.as_number(),
            Self::Float(value) => *value -= rhs.as_float(),
            _ => panic!(),
        }
    }
}

impl Rem for Types {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match self {
            Self::Number(value) => Self::Number(value % rhs.as_number()),
            _ => panic!(),
        }
    }
}

impl Div for Types {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Self::Number(value) => Self::Number(value / rhs.as_number()),
            Self::Float(value) => Self::Float(value / rhs.as_float()),
            _ => panic!(),
        }
    }
}

impl Mul for Types {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Self::String(value) => {
                let mut result = String::new();

                for _ in 0..rhs.as_number() {
                    result.push_str(&value);
                }

                Self::String(result)
            }
            Self::Number(value) => Self::Number(value * rhs.as_number()),
            Self::Float(value) => Self::Float(value * rhs.as_float()),
            _ => panic!(),
        }
    }
}

impl Add for Types {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::String(mut value) => {
                value.push_str(rhs.as_string());
                Self::String(value)
            }
            Self::Number(value) => Self::Number(value + rhs.as_number()),
            Self::Float(value) => Self::Float(value + rhs.as_float()),
            _ => panic!(),
        }
    }
}

impl Sub for Types {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Self::Number(value) => Self::Number(value - rhs.as_number()),
            Self::Float(value) => Self::Float(value - rhs.as_float()),
            _ => panic!(),
        }
    }
}
