use std::collections::HashMap;

use koopa::ir;

#[derive(Debug, Clone, Copy)]
pub enum SymVal {
    GlobalConst(i32),
    Val(ir::Value)
}

pub type FuncTab = HashMap<String, ir::Function>;
pub type ValTab = HashMap<String, SymVal>;

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
            .insert(name.to_string(), SymVal::Val(value));
    }

    pub fn get_symval(&self, name: &str) -> Option<SymVal> {
        for scope in self.scope.iter().rev() {
            /* match scope.get(name) {
                Some(val) => return Some(*val),
                None => (),
            } */
            if let val @ Some(_) = scope.get(name) {
                return val.cloned();
            }
        }
        if let val @ Some(_) = self.global.get(name) {
            return val.cloned();
        }
        None
    }

    /* pub fn get_val(&self, name: &str, ctx: &'a mut Context) -> Option<ir::Value> {
        self.get_symval(name).map(|sv| {
            match sv {
                SymVal::Val(v) => v,
                SymVal::GlobalConst(i) => ctx.add_plain_value_integer(i),
            }
        })
    } */

    pub fn get_func(&self, name: &str) -> Option<ir::Function> {
        self.func.get(name).cloned()
    }
}

pub trait FetchVal<'a> {
    fn fetch_val(&self, name: &str) -> Option<SymVal>;
    fn fetch_val_kind(&self, val: ir::Value) -> ir::entities::ValueKind;
    fn fetch_val_type(&self, val: ir::Value) -> ir::TypeKind;
}
