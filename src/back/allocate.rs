use koopa::ir;

use super::risc;

pub trait Allocate {
    /// 获得该类型对应的空间大小
    fn allocate(&self) -> i32 {
        0
    }
}

impl Allocate for ir::Type {
    fn allocate(&self) -> i32 {
        // use ir::TypeKind::*;
        // match self.kind() {
        //     Int32 => 4,
        //     Unit => 0,
        //     Array(_, _) => self.size(),
        //     Pointer(t) => match t.kind() {
        //         Int32 => 4,
        //         Unit => unreachable!(),
        //         _ => unimplemented!("Future"),
        //     },
        //     Function(_, _) => unimplemented!("Function size unknown"),
        // }
        self.size().try_into().unwrap()
    }
}

impl Allocate for ir::entities::ValueData {
    /// 将所有的值 spill 到栈上
    fn allocate(&self) -> i32 {
        use ir::{ValueKind::*, TypeKind::*};
        match self.kind() {
            Binary(_) | Call(_) | GetElemPtr(_) => self.ty().allocate(),
            Alloc(_) => {
                { if let Pointer(t) = self.ty().kind() { t } else { unreachable!() } }
                .allocate()
            }
            _ => 0,
        }
    }
}

impl Allocate for risc::RiscReg {
    fn allocate(&self) -> i32 {
        4
    }
}

impl Allocate for i32 {
    fn allocate(&self) -> i32 {
        *self
    }
}
