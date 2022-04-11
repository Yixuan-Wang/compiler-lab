use koopa::ir::Value;

use crate::back::{
    risc::{
        RiscInst::{self as Inst, *},
        RiscReg as Reg, RiscLabel,
    },
    Context,
};
use crate::WrapProgram;
use crate::frame;

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
            None => ctx.reg_map_mut().appoint_temp_reg(*self),
        };
        match value_data.kind() {
            Integer(i) => (reg, vec![Li(reg, i.value())]),
            Binary(_) | Call(_) => {
                let offset = frame!(ctx).get(*self);
                (reg, Lw(reg, offset, Reg::Sp).expand_imm())
            }
            Load(l) => {
                if !l.src().is_global() {
                    let offset = frame!(ctx).get(l.src());
                    (reg, Lw(reg, offset, Reg::Sp).expand_imm())
                } else {
                    let label = RiscLabel::strip(ctx.value(l.src()).name().clone().unwrap());
                    (reg, vec![La(reg, label), Lw(reg, 0, reg)])
                }
            }
            FuncArgRef(a) => {
                let i = a.index();
                if i >= 8 {
                    (
                        reg,
                        Inst::Lw(reg, frame!(ctx).get(*self), Reg::Sp).expand_imm(),
                    )
                } else {
                    (
                        reg,
                        vec![
                            Inst::Mv(reg, Reg::A(i.try_into().unwrap())),
                        ]
                    )
                }
            }
            _ => todo!(),
        }
    }
}
