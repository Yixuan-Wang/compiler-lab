// #[macro_use] use super::context;
// use crate::auton;
use crate::WrapProgram;

use super::ast;
use super::context::Context;
use koopa::ir::{self, builder_traits::*};

mod eval;

pub mod lazy;
pub use lazy::*;

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
            If(exp, then, alt) => {
                let block_name_then = ctx.inst_namer.gen(Some("then".to_string()));
                let block_name_else = ctx.inst_namer.gen(Some("else".to_string()));
                let block_name_endif = ctx.inst_namer.gen(Some("endif".to_string()));

                let block_then = ctx.add_block(&block_name_then);
                let block_endif = ctx.add_block(&block_name_endif);
                let block_else = if alt.is_none() { block_endif } else { ctx.add_block(&block_name_else) };

                {
                    let gate = exp.generate(ctx);               
                    let branch = ctx.add_value(val!(branch(gate, block_then, block_else)), None);
                    ctx.insert_inst(branch, ctx.curr());
                    ctx.seal_block(ctx.curr());
                }

                {
                    ctx.insert_block(block_then);
                    ctx.set_curr(block_then);
                    then.generate(ctx);
                    let jump = ctx.add_value(val!(jump(block_endif)), None);
                    ctx.insert_inst(jump, ctx.curr());
                    ctx.seal_block(ctx.curr());
                }

                if let Some(alt) = alt {
                    ctx.insert_block(block_else);
                    ctx.set_curr(block_else);
                    alt.generate(ctx);
                    let jump = ctx.add_value(val!(jump(block_endif)), None);
                    ctx.insert_inst(jump, ctx.curr());
                    ctx.seal_block(ctx.curr());
                }

                ctx.insert_block(block_endif);
                ctx.set_curr(block_endif);
            },
            While(exp, body) => {
                let block_name_while = ctx.inst_namer.gen(Some("while".to_string()));
                let block_name_loop = ctx.inst_namer.gen(Some("loop".to_string()));
                let block_name_endwhile = ctx.inst_namer.gen(Some("endwhile".to_string()));

                let block_while = ctx.add_block(&block_name_while);
                let block_loop = ctx.add_block(&block_name_loop);
                let block_endwhile = ctx.add_block(&block_name_endwhile);

                let jump_in = ctx.add_value(val!(jump(block_while)), None);
                ctx.insert_inst(jump_in, ctx.curr());
                ctx.seal_block(ctx.curr());

                {
                    ctx.insert_block(block_while);
                    ctx.set_curr(block_while);
                    let gate = exp.generate(ctx);
                    let branch = ctx.add_value(val!(branch(gate, block_loop, block_endwhile)), None);
                    ctx.insert_inst(branch, ctx.curr());
                    ctx.seal_block(ctx.curr());
                }

                {
                    ctx.insert_block(block_loop);
                    ctx.set_curr(block_loop);
                    ctx.enter_loop((block_while, block_endwhile));
                    body.generate(ctx);
                    let jump_back = ctx.add_value(val!(jump(block_while)), None);
                    ctx.insert_inst(jump_back, ctx.curr());
                    ctx.seal_block(ctx.curr());
                    ctx.exit_loop();
                }

                ctx.insert_block(block_endwhile);
                ctx.set_curr(block_endwhile);
            },
            Break => {
                let (_, block_dest) = ctx.curr_loop();
                let jump = ctx.add_value(val!(jump(block_dest)), None);
                ctx.insert_inst(jump, ctx.curr());
                ctx.seal_block(ctx.curr());
            },
            Continue => {
                let (block_dest, _) = ctx.curr_loop();
                let jump = ctx.add_value(val!(jump(block_dest)), None);
                ctx.insert_inst(jump, ctx.curr());
                ctx.seal_block(ctx.curr());
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
                ctx.seal_block(ctx.curr());
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
                let zero = ctx.zero;
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
