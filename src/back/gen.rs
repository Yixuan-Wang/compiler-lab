use super::allo::Allo;
use super::context::Context;
use super::risc;
use koopa::ir;

use crate::WrapProgram;

/// [`Generate`] 处理 [`ir::entities::ValueData`]，将每一条语句从 Koopa 内存形式转化为 RISC-V 指令
pub trait Generate<'a> {
    fn generate(&self, ctx: &'a Context, allo: &'a mut Allo) -> Vec<risc::RiscInst>;
}

impl<'a> Generate<'a> for ir::entities::Value {
    fn generate(&self, ctx: &'a Context, allo: &'a mut Allo) -> Vec<risc::RiscInst> {
        use ir::entities::ValueKind::*;
        use risc::{RiscInst as Inst, RiscReg as Reg};
        let value_data = ctx.value(*self);
        match value_data.kind() {
            Return(val) => match val.value() {
                Some(ret) => match ctx.value(ret).kind() {
                    Integer(i) => {
                        vec![Inst::Li(Reg::A(0), i.value()), Inst::Ret]
                    }
                    Binary(_) => {
                        let retreg = *allo.get_reg(ret).unwrap();
                        vec![Inst::Mv(Reg::A(0), retreg), Inst::Ret]
                    }
                    _ => todo!(),
                },
                None => todo!(),
            },
            Binary(bin) => {
                use ir::BinaryOp::*;
                let mut v = vec![];
                if let None = allo.get_reg(*self) {
                    v.extend(bin.lhs().generate(ctx, allo));
                    v.extend(bin.rhs().generate(ctx, allo));
                    let lreg = *allo.get_reg(bin.lhs()).unwrap();
                    let rreg = *allo.get_reg(bin.rhs()).unwrap();
                    let dreg = allo.allo_reg_t(*self);
                    match bin.op() {
                        Eq => v.extend([
                            Inst::Xor(dreg, lreg, rreg),
                            Inst::Seqz(dreg, dreg),
                        ]),
                        NotEq => v.extend([
                            Inst::Xor(dreg, lreg, rreg),
                            Inst::Snez(dreg, dreg),
                        ]),
                        And => v.extend([Inst::And(dreg, lreg, rreg)]),
                        Or => v.extend([Inst::Or(dreg, lreg, rreg)]),
                        Xor => v.extend([Inst::Xor(dreg, lreg, rreg)]),
                        Lt => v.extend([Inst::Slt(dreg, lreg, rreg)]),
                        Gt => v.extend([Inst::Sgt(dreg, lreg, rreg)]),
                        Le => v.extend([
                            Inst::Sgt(dreg, lreg, rreg),
                            Inst::Xori(dreg, dreg, 1),
                        ]),
                        Ge => v.extend([
                            Inst::Slt(dreg, lreg, rreg),
                            Inst::Xori(dreg, dreg, 1),
                        ]),
                        Add => v.extend([Inst::Add(dreg, lreg, rreg)]),
                        Sub => v.extend([Inst::Sub(dreg, lreg, rreg)]),
                        Mul => v.extend([Inst::Mul(dreg, lreg, rreg)]),
                        Div => v.extend([Inst::Div(dreg, lreg, rreg)]),
                        Mod => v.extend([Inst::Rem(dreg, lreg, rreg)]),
                        _ => todo!()
                    }
                }
                v
            }
            Integer(i) => {
                if let None = allo.get_reg(*self) {
                    if i.value() == 0 {
                        allo.appoint_reg(*self, Reg::X0);
                        vec![]
                    } else {
                        vec![Inst::Li(allo.allo_reg_t(*self), i.value())]
                    }
                } else { vec![] }
            }
            _ => todo!(),
        }
    }
}
