use std::{collections::HashMap};

use koopa::ir;

pub struct Table {
    pub func: HashMap<String, ir::Function>,
    pub scope: Vec<HashMap<String, ir::Value>>,
}

impl Table {
    pub fn new() -> Table {
        Table {
            func: HashMap::new(),
            scope: vec![HashMap::new()],
        }
    }

    pub fn insert_val(&mut self, name: &str, value: ir::Value) {
        self.scope
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value);
    }

    pub fn get_val(&self, name: &str) -> Option<ir::Value> {
        for scope in self.scope.iter().rev() {
            match scope.get(name) {
                Some(val) => return Some(*val),
                None => (),
            }
        }
        None
    }
}
