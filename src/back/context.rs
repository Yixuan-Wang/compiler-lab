use std::{cell::{RefCell, Ref, RefMut}, fmt::Display};

use koopa::ir;
use crate::WrapProgram;

use super::{allo::{AlloReg, AlloStack}, risc::{MAX_IMM, RiscInst, RiscReg}};

pub struct Context<'a> {
    pub program: &'a mut ir::Program,
    func: ir::Function,
    allo_reg: RefCell<AlloReg>,
    allo_stack: RefCell<AlloStack>,
}

impl<'a> WrapProgram for Context<'a> {
    fn program(&self) -> &ir::Program { self.program }
    fn program_mut(&mut self) -> &mut ir::Program { self.program }
    fn func_handle(&self) -> ir::Function { self.func }
}

impl<'a> Context<'a> {
    pub fn new(program: &'a mut ir::Program, func: ir::Function) -> Context {
        Context { 
            program,
            func,
            allo_reg: RefCell::new(AlloReg::new()),
            allo_stack: RefCell::new(AlloStack::new()),
        }
    }

    pub fn allo_reg(&self) -> Ref<AlloReg> {
        self.allo_reg.borrow()
    }

    pub fn allo_reg_mut(&self) -> RefMut<AlloReg> {
        self.allo_reg.borrow_mut()
    }

    pub fn with_allo_reg_mut<F, T>(&self, sth: F) -> T
    where
        F: Fn(RefMut<AlloReg>) -> T
    {
        sth(self.allo_reg.borrow_mut())
    }

    pub fn on_reg(&self, val: ir::Value) -> bool {
        self.allo_reg().contains_key(val)
    }

    pub fn allo_stack(&self) -> Ref<AlloStack> {
        self.allo_stack.borrow()
    }

    pub fn allo_stack_mut(&self) -> RefMut<AlloStack> {
        self.allo_stack.borrow_mut()
    }

    // pub fn on_stack(&self, val: ir::Value) -> bool {
    //     self.allo_stack().contains_key(val)
    // }

    pub fn name(&self) -> &str {
        unsafe { self.func().name().get_unchecked(1..) }
    }

    pub fn prefix_with_name(&self, string: &str) -> String {
        format!("{}_{}", unsafe { self.func().name().get_unchecked(1..) }, unsafe { string.get_unchecked(1..) })
    } 

    pub fn prologue(&self) -> Vec<RiscInst> {
        use RiscInst::*;
        let insts = self.func().layout().bbs().nodes().flat_map(|node| node.insts());
        insts.for_each(|(h, _)| {
            let d = self.value(*h);
            self.allo_stack_mut().insert(*h, d);
        });
        let size = self.allo_stack().size_aligned();
        if size == 0 {
            vec![]
        }
        else if size > MAX_IMM {
            vec![
                Li(RiscReg::T(0), -size),
                Add(RiscReg::Sp, RiscReg::Sp, RiscReg::T(0)),
            ]
        } else {
            vec![
                Addi(RiscReg::Sp, RiscReg::Sp, -size),
            ]
        }
    }

    pub fn epilogue(&self) -> Vec<RiscInst> {
        use RiscInst::*;
        let size = self.allo_stack().size_aligned();
        if size == 0 {
            vec![]
        }
        else if size > MAX_IMM {
            vec![
                Li(RiscReg::T(0), size),
                Add(RiscReg::Sp, RiscReg::Sp, RiscReg::T(0)),
            ]
        } else {
            vec![
                Addi(RiscReg::Sp, RiscReg::Sp, size),
            ]
        }
    }
}
