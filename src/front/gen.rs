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

impl<'f> Generate<'f> for ast::RelExp {
    type Eval = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Eval {
        use ast::RelOp::*;
        match self {
            Self::Unary(p) => p.generate(ctx),
            Self::Binary(b, o, u) => {
                let v = b.generate(ctx);
                let u = u.generate(ctx);
                let inst = match o {
                    Lt => ctx.add_mid_value(val!(binary(ir::BinaryOp::Lt, v, u))),
                    Gt => ctx.add_mid_value(val!(binary(ir::BinaryOp::Gt, v, u))),
                    Le => ctx.add_mid_value(val!(binary(ir::BinaryOp::Le, v, u))),
                    Ge => ctx.add_mid_value(val!(binary(ir::BinaryOp::Ge, v, u))),
                };
                ctx.insert_inst(inst, ctx.curr());
                inst
            }
        }
    }
}

impl<'f> Generate<'f> for ast::EqExp {
    type Eval = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Eval {
        use ast::EqOp::*;
        match self {
            Self::Unary(p) => p.generate(ctx),
            Self::Binary(b, o, u) => {
                let v = b.generate(ctx);
                let u = u.generate(ctx);
                let inst = match o {
                    Eq => ctx.add_mid_value(val!(binary(ir::BinaryOp::Eq, v, u))),
                    Ne => ctx.add_mid_value(val!(binary(ir::BinaryOp::NotEq, v, u))),
                };
                ctx.insert_inst(inst, ctx.curr());
                inst
            }
        }
    }
}

impl<'f> Generate<'f> for ast::LAndExp {
    type Eval = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Eval {
        match self {
            Self::Unary(p) => p.generate(ctx),
            Self::Binary(b, u) => {
                let v = b.generate(ctx);
                let u = u.generate(ctx);
                let zero = ctx.add_value(val!(integer(0)), None);
                let inst1 = ctx.add_mid_value(val!(binary(ir::BinaryOp::NotEq, v, zero)));
                ctx.insert_inst(inst1, ctx.curr());
                let inst2 = ctx.add_mid_value(val!(binary(ir::BinaryOp::NotEq, u, zero)));
                ctx.insert_inst(inst2, ctx.curr());
                let inst3 = ctx.add_mid_value(val!(binary(ir::BinaryOp::And, inst1, inst2)));
                ctx.insert_inst(inst3, ctx.curr());
                inst3
            }
        }
    }
}

impl<'f> Generate<'f> for ast::LOrExp {
    type Eval = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Eval {
        match self {
            Self::Unary(p) => p.generate(ctx),
            Self::Binary(b, u) => {
                let v = b.generate(ctx);
                let u = u.generate(ctx);
                let zero = ctx.add_value(val!(integer(0)), None);
                let inst1 = ctx.add_mid_value(val!(binary(ir::BinaryOp::Or, v, u)));
                ctx.insert_inst(inst1, ctx.curr());
                let inst2 = ctx.add_mid_value(val!(binary(ir::BinaryOp::NotEq, inst1, zero)));
                ctx.insert_inst(inst2, ctx.curr());
                inst2
            }
        }
    }
}