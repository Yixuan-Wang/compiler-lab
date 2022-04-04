use koopa::ir::Value;

use crate::back::{Context, risc::{RiscInst::{self as Inst, *}, RiscReg as Reg}};
use crate::WrapProgram;

pub trait ToReg<'a> {
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