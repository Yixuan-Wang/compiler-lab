// #[macro_use] use super::context;
// use crate::auton;
use crate::{s, WrapProgram};

use super::ast;
use super::context::Context;
use koopa::ir::{self, builder_traits::*};

mod eval;

/// [`Generate`] 处理语句（[`ast::StmtKind`]），将每一条语句转化为 Koopa 内存形式
pub trait Generate<'f> {
    type Val;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val;
}

impl<'f> Generate<'f> for ast::Block {
    type Val = ();
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        for stmt in &self.0 {
            stmt.generate(ctx);
        }
    }
}

impl<'f> Generate<'f> for ast::StmtKind {
    type Val = ();
    fn generate(&self, ctx: &'f mut Context) {
        use ast::StmtKind::*;
        match self {
            Unit => {},
            Exp(e) => { e.generate(ctx); },
            Block(b) => {
                ctx.table_mut().push_scope();
                b.generate(ctx);
                ctx.table_mut().pop_scope();
            },
            Decl(v) => v.iter().for_each(|d| d.generate(ctx)),
            Assign(l, e) => {
                let lval_handle = ctx.table().get_val(&l.0).expect(&format!(
                    "SemanticsError[UndefinedSymbol]: '{}' is used before definition.",
                    &l.0
                ));
                let lval = ctx.value(lval_handle);
                assert!(!lval.kind().is_const(), "SemanticsError[InvalidLValAssignment]: '{}' cannot be assigned to.", &l.0);
                let exp_handle = e.generate(ctx);
                let store = ctx.add_value(val!(store(exp_handle, lval_handle)), None);
                ctx.insert_inst(store, ctx.curr());
            },
            Return(option_r) => {
                let ret = match option_r {
                    Some(r) => {
                        let ret_val = r.generate(ctx);
                        ctx.add_value(val!(ret(Some(ret_val))), None)
                    }
                    None => {
                        ctx.add_value(val!(ret(None)), None)
                    }
                };
                ctx.insert_inst(ret, ctx.curr());
            }
        };
    }
}

impl<'f> Generate<'f> for ast::Decl {
    type Val = ();
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        use ast::SymKind;
        use eval::Eval;
        match self.kind {
            SymKind::Const => {
                let val = self.exp.as_ref().unwrap().eval(ctx).expect(&format!("SemanticsError[ConstEvalFailure]: '{}' cannot be evaluated during compile time.", self.ident));
                let const_val =
                    ctx.add_value(val!(integer(val)), None);
                ctx.table_mut().insert_val(&self.ident, const_val);
            }
            SymKind::Var => {
                let v = match &self.exp {
                    Some(e) => {
                        match e.eval(ctx) {
                            Some(v) => ctx.add_value(val!(integer(v)), None),
                            None => e.generate(ctx),
                        }
                    }
                    None => ctx.add_value(val!(undef(ir::Type::get_i32())), None),
                };
                let alloc = ctx.add_value(val!(alloc(ir::Type::get_i32())), Some(format!("@{}", &self.ident)));
                ctx.table_mut().insert_val(&self.ident, alloc);
                ctx.insert_inst(alloc, ctx.curr());
                let store = ctx.add_value(val!(store(v, alloc)), None);
                ctx.insert_inst(store, ctx.curr());
            },
        };
    }
}

impl<'f> Generate<'f> for ast::LVal {
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        let lval_handle = ctx.table().get_val(&self.0).expect(&format!(
            "SemanticsError[UndefinedSymbol]: '{}' is used before definition.",
            &self.0
        ));
        let lval = ctx.value(lval_handle);
        if lval.kind().is_const() {
            lval_handle
        } else {
            let load = ctx.add_mid_value(val!(load(lval_handle)));
            ctx.insert_inst(load, ctx.curr());
            load
        }
    }
}

impl<'f> Generate<'f> for ast::PrimaryExp {
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> ir::Value {
        match self {
            Self::LVal(l) => l.generate(ctx),
            Self::Literal(i) => ctx.add_value(val!(integer(*i)), None),
            Self::Exp(b) => b.generate(ctx),
        }
    }
}

impl<'f> Generate<'f> for ast::Exp {
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        self.0.generate(ctx)
    }
}

impl<'f> Generate<'f> for ast::UnaryExp {
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
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
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
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
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
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
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
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
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
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
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
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
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
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
