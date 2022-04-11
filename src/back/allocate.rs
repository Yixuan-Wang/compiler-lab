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
        use ir::TypeKind::*;
        match self.kind() {
            Int32 => 4,
            Unit => 0,
            Array(_, _) => unimplemented!("Future"),
            Pointer(t) => match t.kind() {
                Int32 => 4,
                Unit => unreachable!(),
                _ => unimplemented!("Future"),
            },
            Function(_, _) => unimplemented!("Function size unknown"),
        }
    }
}

impl Allocate for ir::entities::ValueData {
    /// 将所有的值 spill 到栈上
    fn allocate(&self) -> i32 {
        use ir::ValueKind::*;
        let ty_size = self.ty().allocate();
        match self.kind() {
            Alloc(_) | Binary(_) | Call(_) => ty_size,
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
