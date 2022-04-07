use std::cell::{Ref, RefCell, RefMut};

use crate::WrapProgram;
use koopa::ir;

use super::{
    regmap::RegMap,
    risc::{RiscInst, RiscReg, MAX_IMM},
    stackmap::StackMap,
};

pub struct Context<'a> {
    pub program: &'a mut ir::Program,
    func: ir::Function,
    allo_reg: RefCell<RegMap>,
    allo_stack: RefCell<StackMap>,
}

impl<'a> WrapProgram for Context<'a> {
    fn program(&self) -> &ir::Program {
        self.program
    }
    fn program_mut(&mut self) -> &mut ir::Program {
        self.program
    }
    fn this_func_handle(&self) -> ir::Function {
        self.func
    }
}

impl<'a> Context<'a> {
    pub fn new(program: &'a mut ir::Program, func: ir::Function) -> Context {
        Context {
            program,
            func,
            allo_reg: RefCell::new(RegMap::new()),
            allo_stack: RefCell::new(StackMap::new()),
        }
    }

    pub fn allo_reg(&self) -> Ref<RegMap> {
        self.allo_reg.borrow()
    }

    pub fn allo_reg_mut(&self) -> RefMut<RegMap> {
        self.allo_reg.borrow_mut()
    }

    /* pub fn with_allo_reg_mut<F, T>(&self, sth: F) -> T
    where
        F: Fn(RefMut<AlloReg>) -> T
    {
        sth(self.allo_reg.borrow_mut())
    } */

    pub fn on_reg(&self, val: ir::Value) -> bool {
        self.allo_reg().contains_key(val)
    }

    pub fn allo_stack(&self) -> Ref<StackMap> {
        self.allo_stack.borrow()
    }

    pub fn allo_stack_mut(&self) -> RefMut<StackMap> {
        self.allo_stack.borrow_mut()
    }

    // pub fn on_stack(&self, val: ir::Value) -> bool {
    //     self.allo_stack().contains_key(val)
    // }

    pub fn name(&self) -> &str {
        unsafe { self.this_func().name().get_unchecked(1..) }
    }

    pub fn prefix_with_name(&self, string: &str) -> String {
        format!(
            "{}_{}",
            unsafe { self.this_func().name().get_unchecked(1..) },
            unsafe { string.get_unchecked(1..) }
        )
    }

    pub fn prologue(&self) -> Vec<RiscInst> {
        use RiscInst::*;
        let mut insts: Vec<ir::Value> = vec![];
        for (_bb, node) in self.this_func().layout().bbs() {
            insts.extend(node.insts().keys())
        }
        insts.iter().for_each(|h| {
            let d = self.value(*h);
            self.allo_stack_mut().insert(*h, d);
        });
        let size = self.allo_stack().size_aligned();
        if size == 0 {
            vec![]
        } else if size > MAX_IMM {
            vec![
                Li(RiscReg::T(0), -size),
                Add(RiscReg::Sp, RiscReg::Sp, RiscReg::T(0)),
            ]
        } else {
            vec![Addi(RiscReg::Sp, RiscReg::Sp, -size)]
        }
    }

    pub fn epilogue(&self) -> Vec<RiscInst> {
        use RiscInst::*;
        let size = self.allo_stack().size_aligned();
        if size == 0 {
            vec![]
        } else if size > MAX_IMM {
            vec![
                Li(RiscReg::T(0), size),
                Add(RiscReg::Sp, RiscReg::Sp, RiscReg::T(0)),
            ]
        } else {
            vec![Addi(RiscReg::Sp, RiscReg::Sp, size)]
        }
    }
}
