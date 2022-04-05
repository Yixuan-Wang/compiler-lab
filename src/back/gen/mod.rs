use super::context::Context;
use super::risc;
use koopa::ir;

use crate::WrapProgram;

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
        use risc::{RiscItem as Item, RiscInst as Inst, RiscReg as Reg};
        let value_data = ctx.value(*self);
        match value_data.kind() {
            Return(r) => {
                let mut v = vec![];
                if let Some(val) = r.value() {
                    let (_, inst) = val.to_reg(ctx, Some(Reg::A(0)));
                    v.extend(inst);
                }
                v.extend(ctx.epilogue());
                v.push(Inst::Ret);
                v
            }
            Binary(bin) => {
                use ir::BinaryOp::*;
                let mut v: Vec<Inst> = vec![];
                if !ctx.on_reg(*self) {
                    let (l, r) = (bin.lhs(), bin.rhs());
                    v.extend(l.generate(ctx));
                    v.extend(r.generate(ctx));
                    let dreg = ctx.allo_reg_mut().allo_reg_t(*self);
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
                    let offset = *ctx.allo_stack().get(*self).unwrap();
                    match bin.op() {
                        Eq => v.extend([
                            Inst::Xor(dreg, lreg, rreg),
                            Inst::Seqz(dreg, dreg),
                        ]),
                        NotEq => v.extend([
                            Inst::Xor(dreg, lreg, rreg),
                            Inst::Snez(dreg, dreg),
                        ]),
                        And => v.push(Inst::And(dreg, lreg, rreg)),
                        Or => v.push(Inst::Or(dreg, lreg, rreg)),
                        Xor => v.push(Inst::Xor(dreg, lreg, rreg)),
                        Lt => v.push(Inst::Slt(dreg, lreg, rreg)),
                        Gt => v.push(Inst::Sgt(dreg, lreg, rreg)),
                        Le => v.extend([
                            Inst::Sgt(dreg, lreg, rreg),
                            Inst::Xori(dreg, dreg, 1),
                        ]),
                        Ge => v.extend([
                            Inst::Slt(dreg, lreg, rreg),
                            Inst::Xori(dreg, dreg, 1),
                        ]),
                        Add => v.push(Inst::Add(dreg, lreg, rreg)),
                        Sub => v.push(Inst::Sub(dreg, lreg, rreg)),
                        Mul => v.push(Inst::Mul(dreg, lreg, rreg)),
                        Div => v.push(Inst::Div(dreg, lreg, rreg)),
                        Mod => v.push(Inst::Rem(dreg, lreg, rreg)),
                        _ => todo!()
                    };
                    v.push(Inst::Sw(dreg, offset, Reg::Sp))
                }
                v
            },
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
            Load(_) => vec![],
            Store(s) => {
                let mut v = vec![];
                if let Undef(_) = ctx.value(s.value()).kind() {
                    return v;
                }
                v.extend(s.value().generate(ctx));
                let (reg, inst) = s.value().to_reg(ctx, None);
                v.extend(inst);
                let offset = *ctx.allo_stack().get(s.dest()).expect(&format!("BackendError: {} not found", ctx.value(s.dest()).name().clone().unwrap()));
                v.push(Inst::Sw(reg, offset, Reg::Sp));
                v
            },
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
                    Inst::Bnez(gate, ctx.prefix_with_name(&true_block_name)),
                    Inst::J(ctx.prefix_with_name(&false_block_name)),
                ]);
                v
            },
            Jump(j) => {
                let target = j.target();
                let target_block_name = ctx.bb(target).name().clone().unwrap();
                vec![Inst::J(ctx.prefix_with_name(&target_block_name))]
            }
            _ => todo!("{:#?}", value_data.kind()),
        }
    }
}
