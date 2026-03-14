// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use crate::types::Types;

#[derive(Debug)]
pub struct Scope {
    vars: Vec<(String, Types, bool)>,
}

impl Scope {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            vars: Vec::with_capacity(16),
        }
    }

    #[inline(always)]
    fn get_raw(&self, name: &str) -> Option<(&Types, &bool)> {
        self.vars
            .iter()
            .rev()
            .find(|(n, _, _)| n == name)
            .map(|(_, v, c)| (v, c))
    }

    #[inline(always)]
    pub fn exist(&self, name: &str) -> bool {
        self.vars.iter().any(|(n, _, _)| n == name)
    }

    #[inline(always)]
    pub fn get<'a>(&'a self, consts: &'a Self, name: &str) -> Option<(&'a Types, &'a bool)> {
        self.vars
            .iter()
            .rev()
            .find(|(n, _, _)| n == name)
            .map(|(_, v, c)| (v, c))
            .or_else(|| consts.get_raw(name))
    }

    #[inline(always)]
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(pos) = self.vars.iter().rposition(|(n, _, _)| n == name) {
            self.vars.swap_remove(pos);
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn declare(&mut self, name: String, value: Types, is_const: bool) {
        if let Some(entry) = self.vars.iter_mut().find(|(n, _, _)| n == &name) {
            entry.1 = value;
            entry.2 = is_const;
        } else {
            self.vars.push((name, value, is_const));
        }
    }
}
