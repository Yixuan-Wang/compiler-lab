use std::collections::HashMap;
use std::collections::hash_map::Entry;
use koopa::ir;

use crate::util::autonum::Autocount;
use super::risc::RiscReg as Reg;

pub struct AlloReg {
    reg_allo: HashMap<ir::Value, Reg>,
    pub reg_t: Autocount,
}

impl AlloReg {
    pub fn new() -> AlloReg {
        AlloReg {
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

    pub fn get(&self, val: ir::Value) -> Option<&Reg> {
        self.reg_allo.get(&val)
    }

    pub fn contains_key(&self, val: ir::Value) -> bool {
        self.reg_allo.contains_key(&val)
    }
}

#[derive(Debug)]
pub struct AlloStack {
    stack_allo: HashMap<ir::Value, i32>,
    size: i32,
}

impl AlloStack {
    pub fn new() -> AlloStack {
        AlloStack {
            stack_allo: HashMap::new(),
            size: 0,
        }
    }

    pub fn size_aligned(&self) -> i32 {
        self.size + (16 - self.size % 16) % 16
    }

    pub fn insert(&mut self, val: ir::Value, data: &ir::entities::ValueData) {
        use ir::{ValueKind::*, TypeKind::*};
        let ty_size = match data.ty().kind() {
            Int32 => 4,
            Unit => 0,
            Array(_, _) => unimplemented!(),
            Pointer(t) => {
                match t.kind() {
                    Int32 => 4,
                    Unit => unreachable!(),
                    _ => unimplemented!()
                }
            },
            Function(_, _) => unimplemented!(),
        };
        let size = match data.kind() {
            Alloc(_) => ty_size,
            _ => 0,
        };
        if size > 0 {
            self.stack_allo.insert(val, self.size + size);
            self.size += size;
        }
    }

    pub fn get(&self, val: ir::Value) -> Option<&i32> {
        self.stack_allo.get(&val)
    }

    // pub fn contains_key(&self, val: ir::Value) -> bool {
    //     self.stack_allo.contains_key(&val)
    // }
}