use std::collections::HashMap;
use std::collections::hash_map::Entry;
use koopa::ir;

use crate::util::autonum::Autocount;
use super::risc::RiscReg as Reg;

pub struct Allo {
    reg_allo: HashMap<ir::Value, Reg>,
    pub reg_t: Autocount,
}

impl Allo {
    pub fn new() -> Allo {
        Allo {
            reg_t: Autocount::new(Some(7)),
            reg_allo: HashMap::new(),
        }
    }

    /// 分配 t 寄存器，可能覆盖
    pub fn allo_reg_t(&mut self, val: ir::Value) -> Reg {
        let reg = self.reg_allo.entry(val);
        match reg {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let r = Reg::T(match self.reg_t.next() {
                    Ok(t) => t,
                    Err(_) => {
                        self.reg_t.reset();
                        self.reg_t.next().unwrap()
                    }
                } as u8);
                e.insert(r);
                r
            }
        }
    }

    pub fn appoint_reg(&mut self, val: ir::Value, reg: Reg) -> Reg {
        self.reg_allo.insert(val, reg);
        reg
    }

    pub fn get_reg(&self, val: ir::Value) -> Option<&Reg> {
        self.reg_allo.get(&val)
    }
}