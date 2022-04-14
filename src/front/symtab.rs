use std::collections::HashMap;

use koopa::ir;

pub type FuncTab = HashMap<String, ir::Function>;
pub type ValTab = HashMap<String, ir::Value>;

pub struct Symtab<'a> {
    pub func: &'a FuncTab,
    pub global: &'a ValTab,
    pub scope: Vec<ValTab>,
}

impl<'a> Symtab<'a> {
    pub fn new(func: &'a FuncTab, global: &'a mut ValTab) -> Symtab<'a> {
        Symtab {
            func,
            global,
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
            /* match scope.get(name) {
                Some(val) => return Some(*val),
                None => (),
            } */
            if let Some(val) = scope.get(name) {
                return Some(*val);
            }
        }
        if let Some(val) = self.global.get(name) {
            return Some(*val);
        }
        None
    }

    pub fn get_func(&self, name: &str) -> Option<ir::Function> {
        self.func.get(name).cloned()
    }
}

pub trait FetchVal<'a> {
    fn fetch_val(&self, name: &str) -> Option<ir::Value>;
    fn fetch_val_kind(&self, val: ir::Value) -> ir::entities::ValueKind;
    fn fetch_val_type(&self, val: ir::Value) -> ir::TypeKind;
}
