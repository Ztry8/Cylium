// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::errors;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Types {
    Vector(Vec<Types>),
    String(String),
    Boolean(bool),
    Number(i64),
    Float(f64),
}

impl Types {
    pub fn create(value: &str) -> Self {
        if let Ok(number) = value.parse::<i64>() {
            Self::Number(number)
        } else if let Ok(float) = value.parse::<f64>() {
            Self::Float(float)
        } else if value == "true" {
            Self::Boolean(true)
        } else if value == "false" {
            Self::Boolean(false)
        } else {
            Self::String(value.to_owned())
        }
    }

    #[inline(always)]
    pub fn as_number(&self) -> Result<i64, String> {
        match self {
            Self::Number(value) => Ok(*value),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn as_float(&self) -> Result<f64, String> {
        match self {
            Self::Float(value) => Ok(*value),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn as_string(&self) -> Result<&str, String> {
        match self {
            Self::String(value) => Ok(value),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Self::Boolean(value) => Ok(*value),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn convert_to_string(&mut self) -> Result<Option<String>, String> {
        let mut warning = None;

        if let Self::Vector(value) = self {
            for member in value {
                let result = member.convert_to_string()?;

                if result.is_some() {
                    warning = result;
                }
            }
        } else {
            *self = Self::String(match self {
                Self::String(value) => {
                    warning = Some(errors::C01.to_owned());
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

        Ok(warning)
    }

    #[inline(always)]
    pub fn convert_to_bool(&mut self) -> Result<Option<String>, String> {
        let mut warning = None;

        if let Self::Vector(value) = self {
            for member in value {
                let result = member.convert_to_bool()?;

                if result.is_some() {
                    warning = result;
                }
            }
        } else {
            *self = Self::Boolean(match self {
                Self::String(value) => value.parse::<bool>().unwrap(),
                Self::Boolean(value) => {
                    warning = Some(errors::C01.to_owned());
                    *value
                }
                Self::Number(value) => *value != 0,
                Self::Float(value) => *value != 0.0,
                _ => unreachable!(),
            });
        }

        Ok(warning)
    }

    #[inline(always)]
    pub fn convert_to_vector(&mut self) -> Result<(), String> {
        *self = Self::Vector(match self {
            Self::String(value) => value
                .chars()
                .map(|sym| Self::create(&sym.to_string()))
                .collect(),
            _ => return Err(errors::A19.to_owned()),
        });

        Ok(())
    }

    #[inline(always)]
    pub fn convert_to_number(&mut self) -> Result<Option<String>, String> {
        let mut warning = None;

        if let Self::Vector(value) = self {
            for member in value {
                let result = member.convert_to_number()?;

                if result.is_some() {
                    warning = result;
                }
            }
        } else {
            *self = Self::Number(match self {
                Self::String(value) => {
                    if let Ok(val) = value.parse::<i64>() {
                        val
                    } else {
                        return Err(errors::A02.to_owned());
                    }
                }
                Self::Boolean(value) => {
                    if *value {
                        1
                    } else {
                        0
                    }
                }
                Self::Number(value) => {
                    warning = Some(errors::C01.to_owned());
                    *value
                }
                Self::Float(value) => value.round() as i64,
                _ => unreachable!(),
            });
        }

        Ok(warning)
    }

    #[inline(always)]
    pub fn convert_to_float(&mut self) -> Result<Option<String>, String> {
        let mut warning = None;

        if let Self::Vector(value) = self {
            for member in value {
                let result = member.convert_to_number()?;

                if result.is_some() {
                    warning = result;
                }
            }
        } else {
            *self = Self::Float(match self {
                Self::String(value) => {
                    if let Ok(val) = value.parse::<f64>() {
                        val
                    } else {
                        return Err(errors::A02.to_owned());
                    }
                }
                Self::Boolean(value) => {
                    if *value {
                        1.0
                    } else {
                        0.0
                    }
                }
                Self::Number(value) => *value as f64,
                Self::Float(value) => {
                    warning = Some(errors::C01.to_owned());
                    *value
                }
                _ => unreachable!(),
            })
        }

        Ok(warning)
    }

    #[inline(always)]
    pub fn rem(&self, rhs: &Self) -> Result<Self, String> {
        match self {
            Self::Number(value) => Ok(Self::Number(value % rhs.as_number()?)),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn div(&self, rhs: &Self) -> Result<Self, String> {
        match self {
            Self::Number(value) => Ok(Self::Number(value / rhs.as_number()?)),
            Self::Float(value) => Ok(Self::Float(value / rhs.as_float()?)),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn mul(&self, rhs: &Self) -> Result<Self, String> {
        match self {
            Self::String(value) => {
                let mut result = String::new();

                for _ in 0..rhs.as_number()? {
                    result.push_str(&value);
                }

                Ok(Self::String(result))
            }
            Self::Number(value) => match rhs.as_number() {
                Ok(num) => Ok(Self::Number(value * num)),
                Err(_) => match rhs.as_string() {
                    Ok(string) => {
                        let mut result = String::new();

                        for _ in 0..*value {
                            result.push_str(string);
                        }

                        Ok(Self::String(result))
                    }
                    Err(e) => Err(e),
                },
            },
            Self::Float(value) => Ok(Self::Float(value * rhs.as_float()?)),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn add(self, rhs: &Self) -> Result<Self, String> {
        match self {
            Self::String(mut value) => {
                value.push_str(rhs.as_string()?);
                Ok(Self::String(value))
            }
            Self::Number(value) => Ok(Self::Number(value + rhs.as_number()?)),
            Self::Float(value) => Ok(Self::Float(value + rhs.as_float()?)),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn sub(&self, rhs: &Self) -> Result<Self, String> {
        match self {
            Self::Number(value) => Ok(Self::Number(value - rhs.as_number()?)),
            Self::Float(value) => Ok(Self::Float(value - rhs.as_float()?)),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn sqrt(&mut self) -> Result<(), String> {
        if let Self::Float(value) = self {
            *value = value.sqrt();
            Ok(())
        } else {
            Err(errors::A16.to_owned())
        }
    }

    #[inline(always)]
    pub fn sin(&mut self) -> Result<(), String> {
        if let Self::Float(value) = self {
            *value = value.sin();
            Ok(())
        } else {
            Err(errors::A16.to_owned())
        }
    }

    #[inline(always)]
    pub fn cos(&mut self) -> Result<(), String> {
        if let Self::Float(value) = self {
            *value = value.cos();
            Ok(())
        } else {
            Err(errors::A16.to_owned())
        }
    }

    #[inline(always)]
    pub fn is_neg(&self) -> Result<bool, String> {
        match self {
            Self::Number(value) => Ok(value.is_negative()),
            Self::Float(value) => Ok(value.is_sign_negative()),
            _ => Err(errors::A16.to_owned()),
        }
    }

    #[inline(always)]
    pub fn is_zero(&self) -> Result<bool, String> {
        match self {
            Self::Number(value) => Ok(*value == 0),
            Self::Float(value) => Ok(*value == 0.0),
            _ => Err(errors::A16.to_owned()),
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
