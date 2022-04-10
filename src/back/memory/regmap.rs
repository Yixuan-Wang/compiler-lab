use koopa::ir;
use std::collections::HashMap;

use crate::back::risc::RiscReg as Reg;
use crate::util::autonum::Autocount;

pub struct RegMap {
    reg_allo: HashMap<ir::Value, Reg>,

    /// Stores owners of temp regs.
    reg_owner: HashMap<Reg, ir::Value>,
    pub reg_t: Autocount,
}

impl RegMap {
    pub fn new() -> RegMap {
        RegMap {
            reg_t: Autocount::new(0, Some(7)),
            reg_allo: HashMap::new(),
            reg_owner: HashMap::new(),
        }
    }

    /// 分配临时寄存器
    pub fn appoint_temp_reg(&mut self, val: ir::Value) -> Reg {
        let reg = Reg::T(match self.reg_t.gen() {
            Ok(t) => t,
            Err(_) => {
                self.reg_t.reset();
                self.reg_t.gen().unwrap()
            }
        } as u8);
        self.appoint_reg(val, reg);
        reg
    }

    pub fn appoint_reg(&mut self, val: ir::Value, reg: Reg) -> Reg {
        self.reg_allo.insert(val, reg);
        if let Some(old) = self.reg_owner.insert(reg, val) {
            self.reg_allo.remove(&old);
        }
        reg
    }

    /* pub fn get(&self, val: ir::Value) -> Option<&Reg> {
        self.reg_allo.get(&val)
    } */

    // pub fn get_or_appoint(&mut self, val: ir::Value) -> Reg {
    //     match self.get(val) {
    //         Some(reg) => *reg,
    //         None => self.allo_reg_t(val),
    //     }
    // }

    pub fn contains_key(&self, val: ir::Value) -> bool {
        self.reg_allo.contains_key(&val)
    }
}
