use std::{collections::HashMap};

use koopa::ir;

#[derive(PartialEq, Eq)]
pub enum Symbol {
    // Func(ir::Function),
    Var(ir::Value),
}

pub struct Table(pub Vec<HashMap<String, Symbol>>);

impl Table {
    pub fn new() -> Table {
        Table(vec![HashMap::new()])
    }

    pub fn insert_var(&mut self, name: String, value: ir::Value) {
        self.0
            .last_mut()
            .unwrap()
            .insert(name, Symbol::Var(value));
    }

    pub fn get_var(self, name: &str) -> Option<ir::Value> {
        use Symbol::*;
        for scope in self.0.iter().rev() {
            match scope.get(name) {
                Some(sym) => {
                    if let Var(val) = sym { return Some(*val); }
                },
                None => (),
            }
        }
        None
    }
}
