use std::cell::{Ref, RefCell, RefMut};

use crate::WrapProgram;
use koopa::ir;

use super::{
    risc::{RiscInst, RiscReg as Reg, MAX_IMM, RiscLabel},
    memory::{stack::StackMap, regmap::RegMap},
};

pub struct Context<'a> {
    pub program: &'a mut ir::Program,
    name: String,
    stack: &'a mut RefCell<StackMap>,
    func: ir::Function,
    is_leaf: RefCell<Option<bool>>,
    reg_map: RefCell<RegMap>,
    // stack_map: RefCell<FrameMap>,
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

#[macro_export]
/// 获得当前函数的栈帧
macro_rules! frame {
    ($ctx:ident) => {
        $ctx.stack().frame($ctx.this_func_handle())
    };
    ($ctx:ident ._mut) => {
        $ctx.stack_mut().frame_mut($ctx.this_func_handle())
    };
}

impl<'a> Context<'a> {
    pub fn new(program: &'a mut ir::Program, stack: &'a mut RefCell<StackMap>, func: ir::Function) -> Context<'a> {
        let name = unsafe { program.func(func).name().get_unchecked(1..) }.into();
        Context {
            program,
            name,
            stack,
            func,
            is_leaf: RefCell::new(None),
            reg_map: RefCell::new(RegMap::new()),
            // stack_map: RefCell::new(FrameMap::new()),
        }
    }

    pub fn reg_map(&self) -> Ref<RegMap> {
        self.reg_map.borrow()
    }

    pub fn reg_map_mut(&self) -> RefMut<RegMap> {
        self.reg_map.borrow_mut()
    }

    /* pub fn with_allo_reg_mut<F, T>(&self, sth: F) -> T
    where
        F: Fn(RefMut<AlloReg>) -> T
    {
        sth(self.allo_reg.borrow_mut())
    } */

    pub fn on_reg(&self, val: ir::Value) -> bool {
        self.reg_map().contains_key(val)
    }

    pub fn stack(&self) -> Ref<StackMap> {
        self.stack.borrow()
    }

    pub fn stack_mut(&self) -> RefMut<StackMap> {
        self.stack.borrow_mut()
    }

    // pub fn stack_map(&self) -> Ref<StackMap> {
    //     self.stack_map.borrow()
    // }

    // pub fn stack_map_mut(&self) -> RefMut<StackMap> {
    //     self.stack_map.borrow_mut()
    // }

    // pub fn on_stack(&self, val: ir::Value) -> bool {
    //     self.allo_stack().contains_key(val)
    // }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn label<T>(&self, ir_name: T) -> RiscLabel
    where T: ToString
    {
        RiscLabel::strip(ir_name.to_string()).with_prefix(self.name())
    }

    pub fn prologue(&self) -> Vec<RiscInst> {
        use ir::ValueKind;
        use RiscInst::*;
        use crate::back::memory::FrameObj::Slot;
        let mut v = vec![];

        let mut ir_insts: Vec<ir::Value> = vec![];
        for (_bb, node) in self.this_func().layout().bbs() {
            ir_insts.extend(node.insts().keys())
        }

        // 统计所有函数调用的参数长度，每个参数
        let calls: Vec<_> = ir_insts
            .iter()
            .filter(|i| matches!(self.value(**i).kind(), ValueKind::Call(..)))
            .map(|i| {
                if let ValueKind::Call(call) = self.value(*i).kind() {
                    let args = call.args();
                    // ? 简单规定所有参数空间都是 4
                    args.len()
                } else {
                    unreachable!()
                }
            })
            .collect();

        // 确定是否保存 `ra` 寄存器，并分配空间
        let is_leaf = calls.len() == 0;
        *self.is_leaf.borrow_mut() = Some(is_leaf);
        if !is_leaf {
            let ra = Reg::Ra;
            frame!(self._mut).insert_high(ra, &ra);
        }

        // 该函数的参数若超过 8 个，将存在前一个栈帧中
        // 此处在本栈帧中插入一个**不实际分配空间的**，指向前一栈帧的偏移量 Allocate::Prev
        //   键：参数
        //   值：偏移量 Allocate::Prev
        let params = self.this_func().params();
        params.iter().enumerate().for_each(|(i, h)| {
            if i >= 8 {
                // ? 简单规定所有参数空间都是 4
                frame!(self._mut).insert_prev(*h, &4);
            } else {
                self.reg_map_mut().appoint_reg(*h, Reg::A(i.try_into().unwrap()));
            }
        });

        // 分配局部变量
        ir_insts.iter().for_each(|h| {
            let d = self.value(*h);
            frame!(self._mut).insert_high(*h, d);
        });

        // 分配额外的用于本函数调用**其他函数**传参所需的空间
        // 寄存器上最多存储 8 个 i32，多余的参数要想传给下一个函数，需要存在本栈帧的底部
        // 此处在本栈帧中插入一个指向本栈帧中从底部向上分配的空间
        //   键：一个留待使用的槽，FrameObj::Slot
        //   值：
        let call_param_count = calls.iter().map(|a| (*a as isize - 7).max(0)).max().unwrap_or(0) * 4;
        let call_param_count: i32 = call_param_count.try_into().unwrap();
        for i in 0..call_param_count {
            frame!(self._mut).insert_low(Slot(i + 8), &4);
        }

        // 移动栈指针
        frame!(self._mut).align();
        let size = frame!(self).size_aligned();
        if size > MAX_IMM {
            v.extend([Li(Reg::T(0), -size), Add(Reg::Sp, Reg::Sp, Reg::T(0))])
        } else if size != 0 {
            v.push(Addi(Reg::Sp, Reg::Sp, -size))
        }

        // 保存 `ra` 寄存器
        if !is_leaf {
            v.push(Sw(Reg::Ra, frame!(self).get(Reg::Ra), Reg::Sp));
        }

        v
    }

    pub fn epilogue(&self) -> Vec<RiscInst> {
        use RiscInst::*;
        let mut v = vec![];

        // 复原返回地址
        if !self.is_leaf.borrow().unwrap() {
            let ra = Reg::Ra;
            v.push(Lw(ra, frame!(self).get(ra), Reg::Sp));
        }

        // 移动栈指针
        let size = frame!(self).size_aligned();
        if size > MAX_IMM {
            v.extend([Li(Reg::T(0), size), Add(Reg::Sp, Reg::Sp, Reg::T(0))])
        } else if size != 0 {
            v.push(Addi(Reg::Sp, Reg::Sp, size))
        }

        v.push(Ret);

        v
    }
}
