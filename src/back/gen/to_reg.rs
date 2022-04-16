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

        if matches!(value_data.kind(), ZeroInit(_)) {
            return (Reg::Zero, vec![]);
        }

        let reg = match ideal {
            Some(reg) => reg,
            None => ctx.reg_map_mut().appoint_temp_reg(*self),
        };
        match value_data.kind() {
            Integer(i) => (reg, vec![Li(reg, i.value())]),
            ZeroInit(_) => unreachable!(),
            Binary(_) | Call(_) => {
                let offset = frame!(ctx).get(*self);
                (reg, Lw(reg, offset, Reg::Sp).expand_imm())
            }
            // --- ptr types ---
            Alloc(_) => {
                // ptr == 栈地址 + 偏移量
                let offset = frame!(ctx).get(*self);
                // // v.extend(Lw(reg, offset, Reg::Sp).expand_imm());
                (reg, Addi(reg, Reg::Sp, offset).expand_imm())
            }
            GlobalAlloc(_) => {
                // ptr == 标签地址
                let label = RiscLabel::strip(ctx.value(*self).name().clone().unwrap());
                (reg, vec![La(reg, label)])
            }
            // load ptr
            // where ptr: *T
            //       load: T
            Load(l) => {
                let mut v = vec![];
                v.push(Com(format!("load {:?} {:?}", value_data.name(), ctx.value(l.src()).kind())));

                // ptr kind
                let (_, ptr_insts) = l.src().to_reg(ctx, Some(reg));
                v.extend(ptr_insts);
                v.push(Lw(reg, 0, reg));
                (reg, v)
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
            GetElemPtr(_) => {
                let offset = frame!(ctx).get(*self);
                (reg, Lw(reg, offset, Reg::Sp).expand_imm())
            }
            _ => unimplemented!("{:#?}", value_data.kind()),
        }
    }
}
