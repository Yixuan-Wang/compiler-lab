use super::context::Context;
use super::risc;
use koopa::ir;

use crate::WrapProgram;
use crate::back::risc::RiscLabel;
use crate::frame;

mod to_reg;
use to_reg::ToReg;

// macro_rules! wrap_inst {
//     ($i: ident) => {
//         $i.into_iter().map(Item::Inst)
//     };
// }

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
            Return(r) => {
                let mut v = vec![];
                if let Some(val) = r.value() {
                    let (_, inst) = val.to_reg(ctx, Some(Reg::A(0)));
                    v.extend(inst);
                }
                v.push(Inst::J(RiscLabel::new("end").with_prefix(ctx.name())));
                // v.extend(ctx.epilogue());
                // v.push(Inst::Ret);
                v
            }
            Binary(bin) => {
                use ir::BinaryOp::*;
                let mut v: Vec<Inst> = vec![];
                if !ctx.on_reg(*self) {
                    let (l, r) = (bin.lhs(), bin.rhs());
                    let dreg = ctx.reg_map_mut().appoint_temp_reg(*self);
                    let (lreg, linst) = l.to_reg(ctx, None);
                    let (rreg, rinst) = r.to_reg(ctx, None);
                    v.extend(linst);
                    v.extend(rinst);
                    // let lreg = ctx.allo_reg_mut().allo_reg_t(l);
                    // let rreg = ctx.allo_reg_mut().allo_reg_t(r);

                    // for (val, reg) in [(l, lreg), (r, rreg)] {
                    //     if let Integer(i) = ctx.value(val).kind() {
                    //         v.push(Inst::Li(reg, i.value()))
                    //     } else {
                    //         v.push(Inst::Lw(reg, *ctx.allo_stack().get(val).unwrap(), Reg::Sp))
                    //     }
                    // }
                    let offset = frame!(ctx).get(*self);
                    match bin.op() {
                        Eq => v.extend([Inst::Xor(dreg, lreg, rreg), Inst::Seqz(dreg, dreg)]),
                        NotEq => v.extend([Inst::Xor(dreg, lreg, rreg), Inst::Snez(dreg, dreg)]),
                        And => v.push(Inst::And(dreg, lreg, rreg)),
                        Or => v.push(Inst::Or(dreg, lreg, rreg)),
                        Xor => v.push(Inst::Xor(dreg, lreg, rreg)),
                        Lt => v.push(Inst::Slt(dreg, lreg, rreg)),
                        Gt => v.push(Inst::Sgt(dreg, lreg, rreg)),
                        Le => v.extend([Inst::Sgt(dreg, lreg, rreg), Inst::Xori(dreg, dreg, 1)]),
                        Ge => v.extend([Inst::Slt(dreg, lreg, rreg), Inst::Xori(dreg, dreg, 1)]),
                        Add => v.push(Inst::Add(dreg, lreg, rreg)),
                        Sub => v.push(Inst::Sub(dreg, lreg, rreg)),
                        Mul => v.push(Inst::Mul(dreg, lreg, rreg)),
                        Div => v.push(Inst::Div(dreg, lreg, rreg)),
                        Mod => v.push(Inst::Rem(dreg, lreg, rreg)),
                        _ => todo!(),
                    };
                    v.extend(Inst::Sw(dreg, offset, Reg::Sp).expand_imm())
                }
                v
            }
            Integer(_) => vec![],
            /* {
                if !ctx.on_reg(*self) {
                    if i.value() == 0 {
                        ctx.allo_reg_mut().appoint_reg(*self, Reg::X0);
                        vec![]
                    } else {
                        vec![Inst::Li(ctx.allo_reg_mut().allo_reg_t(*self), i.value())]
                    }
                } else { vec![] }
            }, */
            Alloc(_) => vec![],
            GlobalAlloc(_) => vec![],
            Load(_) => vec![],
            Store(s) => {
                let mut v = vec![];
                if let Undef(_) = ctx.value(s.value()).kind() {
                    return v;
                }
                let (reg, inst) = s.value().to_reg(ctx, None);
                v.extend(inst);
                if !s.dest().is_global() {
                    let offset = frame!(ctx).get(s.dest());
                    v.extend(Inst::Sw(reg, offset, Reg::Sp).expand_imm());
                } else {
                    let label = RiscLabel::strip(ctx.value(s.dest()).name().clone().unwrap());
                    let temp_reg = ctx.reg_map_mut().appoint_temp_reg(s.dest());
                    v.extend([Inst::La(temp_reg, label), Inst::Sw(reg, 0, temp_reg)]);
                }
                v
            }
            Undef(_) => vec![],
            Branch(b) => {
                let mut v = vec![];
                let cond = b.cond();
                let (gate, gate_inst) = cond.to_reg(ctx, None);
                v.extend(gate_inst);
                let (true_block_name, false_block_name) = (
                    ctx.bb(b.true_bb()).name().clone().unwrap(),
                    ctx.bb(b.false_bb()).name().clone().unwrap(),
                );
                v.extend([
                    Inst::Bnez(gate, ctx.label(&true_block_name)),
                    Inst::J(ctx.label(&false_block_name)),
                ]);
                v
            }
            Jump(j) => {
                let target = j.target();
                let target_block_name = ctx.bb(target).name().clone().unwrap();
                vec![Inst::J(ctx.label(&target_block_name))]
            }
            Call(c) => {
                use crate::back::memory::stack::FrameObj::Slot;
                let mut v = vec![];
                c.args().iter().enumerate().for_each(|(i, val)| {
                    let (reg, insts) = val.to_reg(ctx, None);
                    v.extend(insts);
                    if i >= 8 {
                        v.extend(Inst::Sw(reg, frame!(ctx).get(Slot(i.try_into().unwrap())), Reg::Sp).expand_imm())
                    } else {
                        v.push(Inst::Mv(Reg::A(i.try_into().unwrap()), reg))
                    };
                });
                v.push(Inst::Call(RiscLabel::strip(ctx.func(c.callee()).name())));
                if !ctx.value(*self).ty().is_unit() {
                    v.extend(Inst::Sw(Reg::A(0), frame!(ctx).get(*self), Reg::Sp).expand_imm())
                }
                v
            }
            FuncArgRef(_) => vec![],
            _ => todo!("{:#?}", value_data.kind()),
        }
    }
}
