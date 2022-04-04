use std::borrow::{BorrowMut, Borrow};
use std::panic;

use super::allo::AlloReg;
use super::context::Context;
use super::risc;
use koopa::ir;

use crate::WrapProgram;

/// [`Generate`] 处理 [`ir::entities::ValueData`]，将每一条语句从 Koopa 内存形式转化为 RISC-V 指令
pub trait Generate<'a> {
    fn generate(&self, ctx: &'a Context) -> Vec<risc::RiscInst>;
}

impl<'a> Generate<'a> for ir::entities::Value {
    fn generate(&self, ctx: &'a Context) -> Vec<risc::RiscInst> {
        use ir::entities::ValueKind::*;
        use risc::{RiscInst as Inst, RiscReg as Reg};
        let value_data = ctx.value(*self);
        match value_data.kind() {
            Return(val) => {
                let mut v = match val.value() {
                    Some(ret) => match ctx.value(ret).kind() {
                        Integer(i) => {
                            vec![Inst::Li(Reg::A(0), i.value())]
                        }
                        Binary(_) | Load(_) => {
                            if !ctx.on_reg(ret) {
                                ret.generate(ctx);
                            };
                            let retreg = *ctx.allo_reg().get(ret).unwrap();
                            vec![Inst::Mv(Reg::A(0), retreg)]
                        }
                        _ => todo!(),
                    },
                    None => todo!(),
                };
                v.extend(ctx.epilogue());
                v.push(Inst::Ret);
                v
            }
            Binary(bin) => {
                use ir::BinaryOp::*;
                let mut v = vec![];
                if !ctx.on_reg(*self) {
                    v.extend(bin.lhs().generate(ctx));
                    v.extend(bin.rhs().generate(ctx));
                    let lreg = *ctx.allo_reg().get(bin.lhs()).unwrap();
                    let rreg = *ctx.allo_reg().get(bin.rhs()).unwrap();
                    let dreg = ctx.allo_reg_mut().allo_reg_t(*self);
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
            },
            Integer(i) => {
                if !ctx.on_reg(*self) {
                    if i.value() == 0 {
                        ctx.allo_reg_mut().appoint_reg(*self, Reg::X0);
                        vec![]
                    } else {
                        vec![Inst::Li(ctx.allo_reg_mut().allo_reg_t(*self), i.value())]
                    }
                } else { vec![] }
            },
            Alloc(_) => vec![],
            Load(l) => {
                if !ctx.on_reg(*self) {
                    let offset = *ctx.allo_stack().get(l.src()).unwrap();
                    vec![
                        // Inst::Com(format!("load by {:?}", ctx.value(*self).kind())),
                        Inst::Lw(ctx.allo_reg_mut().allo_reg_t(*self), offset, Reg::Sp),
                    ]
                } else { vec![] }
            },
            Store(s) => {
                let mut v = vec![];
                if let Undef(_) = ctx.value(s.value()).kind() {
                    return v;
                }
                v.extend(s.value().generate(ctx));
                let offset = *ctx.allo_stack().get(s.dest()).unwrap();
                v.push(Inst::Sw(*ctx.allo_reg().get(s.value()).expect(&format!("{:#?}", ctx.value(s.value()))), offset, Reg::Sp));
                v
            },
            Undef(_) => vec![],
            _ => todo!("{:#?}", value_data.kind()),
        }
    }
}
