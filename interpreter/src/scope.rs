// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use crate::types::Types;

#[derive(Debug)]
pub struct Scope {
    vars: HashMap<String, (Types, bool)>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    #[inline(always)]
    fn get_raw(&self, name: &str) -> Option<&(Types, bool)> {
        self.vars.get(name)
    }

    #[inline(always)]
    pub fn exist(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    #[inline(always)]
    pub fn get<'a>(&'a self, consts: &'a Self, name: &str) -> Option<&'a (Types, bool)> {
        self.vars.get(name).or(consts.get_raw(name))
    }

    #[inline(always)]
    pub fn remove(&mut self, name: &str) -> bool {
        self.vars.remove(name).is_some()
    }

    #[inline(always)]
    pub fn declare(&mut self, name: String, value: Types, is_const: bool) {
        self.vars.insert(name, (value, is_const));
    }
}
