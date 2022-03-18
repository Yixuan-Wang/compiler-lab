// #[macro_use] use super::context;
// use crate::auton;
use crate::{s, WrapProgram};

use super::context::Context;
use super::ast;
use koopa::ir::{self, builder_traits::*};

/// [`Generate`] 处理语句（[`ast::StmtKind`]），将每一条语句转化为 Koopa 内存形式
pub trait Generate<'f> {
    type Eval;
    fn generate(&self, ctx: &'f mut Context) -> Self::Eval;
}

impl<'f> Generate<'f> for ast::StmtKind {
    type Eval = ();
    fn generate(&self, ctx: &'f mut Context) {
        use ast::StmtKind::*;
        match self {
            Return(r) => {
                // let (curr, end) = (ctx.curr(), ctx.end());
                let entry = ctx.entry();
                // let ret_val = ctx.table.get_var("%ret");
                let ret_val = r.generate(ctx);                
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

impl<'f> Generate<'f> for ast::PrimaryExp {
    type Eval = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> ir::Value {
        match self {
            Self::Literal(i) => ctx.add_value(val!(integer(*i)), None),
            Self::Exp(b) => b.generate(ctx),
        }
    }
}

impl<'f> Generate<'f> for ast::Exp {
    type Eval = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Eval {
        self.0.generate(ctx)
    }
}

impl<'f> Generate<'f> for ast::UnaryExp {
    type Eval = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Eval {
        use ast::UnaryOp::*;
        match self {
            Self::Primary(p) => p.generate(ctx),
            Self::Unary(o, b) => {
                let v = b.generate(ctx);
                let zero = ctx.add_value(val!(integer(0)), None);
                let inst = match o {
                    Minus => ctx.add_mid_value(val!(binary(ir::BinaryOp::Sub, zero, v))),
                    LNot => ctx.add_mid_value(val!(binary(ir::BinaryOp::Eq, zero, v))),
                };
                ctx.insert_inst(inst, ctx.curr());
                inst
            }
        }
    }
}

impl<'f> Generate<'f> for ast::MulExp {
    type Eval = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Eval {
        use ast::MulOp::*;
        match self {
            Self::Unary(p) => p.generate(ctx),
            Self::Binary(b, o, u) => {
                let v = b.generate(ctx);
                let u = u.generate(ctx);
                let inst = match o {
                    Mul => ctx.add_mid_value(val!(binary(ir::BinaryOp::Mul, v, u))),
                    Div => ctx.add_mid_value(val!(binary(ir::BinaryOp::Div, v, u))),
                    Mod => ctx.add_mid_value(val!(binary(ir::BinaryOp::Mod, v, u))),
                };
                ctx.insert_inst(inst, ctx.curr());
                inst
            }
        }
    }
}

impl<'f> Generate<'f> for ast::AddExp {
    type Eval = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Eval {
        use ast::AddOp::*;
        match self {
            Self::Unary(p) => p.generate(ctx),
            Self::Binary(b, o, u) => {
                let v = b.generate(ctx);
                let u = u.generate(ctx);
                let inst = match o {
                    Add => ctx.add_mid_value(val!(binary(ir::BinaryOp::Add, v, u))),
                    Sub => ctx.add_mid_value(val!(binary(ir::BinaryOp::Sub, v, u))),
                };
                ctx.insert_inst(inst, ctx.curr());
                inst
            }
        }
    }
}
