use std::collections::HashMap;

use koopa::ir;

pub struct Symtab {
    pub func: HashMap<String, ir::Function>,
    pub scope: Vec<HashMap<String, ir::Value>>,
}

impl Symtab {
    pub fn new() -> Symtab {
        Symtab {
            func: HashMap::new(),
            scope: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scope.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scope.pop();
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
