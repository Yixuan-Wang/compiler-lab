// #[macro_use] use super::context;
// use crate::auton;
use crate::{s, WrapProgram};

use super::context::Context;
use super::ast;
use koopa::ir::{self, builder_traits::*};

/// [`Instruct`] 处理语句（[`ast::StmtKind`]），将每一条语句转化为 Koopa 内存形式
pub trait Instruct<'f> {
    type Eval;
    fn instruct(&self, ctx: &'f mut Context) -> Self::Eval;
}

impl<'f> Instruct<'f> for ast::StmtKind {
    type Eval = ();
    fn instruct(&self, ctx: &'f mut Context) {
        use ast::StmtKind::*;
        match self {
            Return(r) => {
                // let (curr, end) = (ctx.curr(), ctx.end());
                let entry = ctx.entry();
                // let ret_val = ctx.table.get_var("%ret");
                let ret_val = r.instruct(ctx);
                let val = ctx.value(ret_val);
                
                // let store = ctx.add_value(val!(store(return_cnst, ret_val)), None);
                // let jump = ctx.add_value(val!(jump(end)), None);
                // ctx.insert_inst(store, curr);
                // ctx.insert_inst(jump, curr);
                let ret = ctx.add_value(val!(ret(Some(ret_val))), None);
                ctx.insert_inst(ret, entry);
            }
        };
    }
}

impl<'f> Instruct<'f> for ast::PrimaryExp {
    type Eval = ir::Value;
    fn instruct(&self, ctx: &'f mut Context) -> ir::Value {
        match self {
            Self::Literal(i) => ctx.add_value(val!(integer(*i)), None),
            Self::Exp(b) => b.instruct(ctx),
        }
    }
}

impl<'f> Instruct<'f> for ast::Exp {
    type Eval = ir::Value;
    fn instruct(&self, ctx: &'f mut Context) -> Self::Eval {
        self.0.instruct(ctx)
    }
}

impl<'f> Instruct<'f> for ast::UnaryExp {
    type Eval = ir::Value;
    fn instruct(&self, ctx: &'f mut Context) -> Self::Eval {
        use ast::UnaryOp::*;
        match self {
            Self::Primary(p) => p.instruct(ctx),
            Self::Unary(o, b) => {
                let v = b.instruct(ctx);
                let zero = ctx.add_value(val!(integer(0)), None);
                match o {
                    Minus => {
                        let minus = ctx.add_mid_value(val!(binary(ir::BinaryOp::Sub, zero, v)));
                        ctx.insert_inst(minus, ctx.curr());
                        minus
                    }
                    LNot => {
                        let lnot = ctx.add_mid_value(val!(binary(ir::BinaryOp::Eq, zero, v)));
                        ctx.insert_inst(lnot, ctx.curr());
                        lnot
                    }
                }
            }
        }
    }
}
