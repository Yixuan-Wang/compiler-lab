use koopa::ir::Value;

use crate::back::{Context, risc::{RiscInst::{self as Inst, *}, RiscReg as Reg}};
use crate::WrapProgram;

/// 将一个值存入寄存器，并生成所需的 RISC-V 指令
pub trait ToReg<'a> {
    /// 将值尽可能存入 `ideal` 寄存器或随机分配临时寄存器，返回所在寄存器及所需的 RISC-V 指令。
    /// 
    /// 具有某种原子性，在调用期间不能有其他寄存器操作。
    fn to_reg(&self, ctx: &'a Context, ideal: Option<Reg>) -> (Reg, Vec<Inst>);
}

impl<'a> ToReg<'a> for Value {
    fn to_reg(&self, ctx: &'a Context, ideal: Option<Reg>) -> (Reg, Vec<Inst>) {
        use koopa::ir::ValueKind::*;
        let value_data = ctx.value(*self);
        let reg = match ideal {
            Some(reg) => reg,
            None => ctx.allo_reg_mut().allo_reg_t(*self)
        };
        match value_data.kind() {
            Integer(i) => {
                (reg, vec![Li(reg, i.value())])
            },
            Binary(_) => {
                let offset = *ctx.allo_stack().get(*self).unwrap();
                (reg, vec![Lw(reg, offset, Reg::Sp)])
            },
            Load(l) => {
                let offset = *ctx.allo_stack().get(l.src()).unwrap();
                (reg, vec![Lw(reg, offset, Reg::Sp)])
            }
            _ => todo!()
        }
    }
}