use std::collections::HashMap;

use koopa::ir;

use super::risc::{RiscReg as Reg};

#[derive(Debug)]
enum FrameObj {
    Val(ir::Value),
    Reg(Reg),
}

impl Into<FrameObj> for ir::Value {
    fn into(self) -> FrameObj {
        FrameObj::Val(self)
    }
}

impl Into<FrameObj> for Reg {
    fn into(self) -> FrameObj {
        FrameObj::Reg(self)
    }
}

#[derive(Debug)]
pub struct StackMap {
    stack_allo: HashMap<ir::Value, i32>,
    size: i32,
}

impl StackMap {
    pub fn new() -> StackMap {
        StackMap {
            stack_allo: HashMap::new(),
            size: 0,
        }
    }

    pub fn size_aligned(&self) -> i32 {
        self.size + (16 - self.size % 16) % 16
    }

    pub fn insert(&mut self, val: ir::Value, data: &ir::entities::ValueData) {
        use ir::{TypeKind::*, ValueKind::*};
        let ty_size = match data.ty().kind() {
            Int32 => 4,
            Unit => 0,
            Array(_, _) => unimplemented!(),
            Pointer(t) => match t.kind() {
                Int32 => 4,
                Unit => unreachable!(),
                _ => unimplemented!(),
            },
            Function(_, _) => unimplemented!(),
        };
        let size = match data.kind() {
            Alloc(_) | Binary(_) => ty_size,
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
