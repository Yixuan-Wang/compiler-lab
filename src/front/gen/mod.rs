use crate::front::ast::{Ty, ShapedInitializer, Initializer};
use crate::front::gen::eval::{generate_evaled_aggregate};
use crate::util::shape::Shape;
use crate::{WrapProgram, ty};

use crate::front::{
    ast::{self, EvaledAggregate},
    context::Context,
};
use koopa::ir::{self, builder_traits::*};

pub mod eval;
use self::eval::Eval;

pub mod prelude;

pub mod lazy;
pub use lazy::*;


use super::ast::{Init, RawAggregate, GeneratedAggregate, AsLVal};
use super::context::AddPlainValue;

/// [`Generate`] 处理语句（[`ast::StmtKind`]），将每一条语句转化为 Koopa 内存形式
pub trait Generate<'f> {
    type Val;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val;
}

impl<'f> Generate<'f> for ast::Block {
    type Val = ();
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        for item in &self.0 {
            item.generate(ctx);
        }
    }
}

impl<'f> Generate<'f> for ast::BlockItem {
    type Val = ();
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        match self {
            Self::Stmt(s) => s.generate(ctx),
            Self::Decl(v) => v.iter().for_each(|d| d.generate(ctx))
        }
    }
}

impl<'f> Generate<'f> for ast::StmtKind {
    type Val = ();
    fn generate(&self, ctx: &'f mut Context) {
        use ast::StmtKind::*;
        match self {
            Unit => {}
            Exp(e) => {
                e.generate(ctx);
            }
            Block(b) => {
                ctx.table_mut().push_scope();
                b.generate(ctx);
                ctx.table_mut().pop_scope();
            }
            Assign(l, e) => {
                /* let lval_handle = ctx.table().get_val(&l.0).unwrap_or_else(|| {
                    panic!(
                        "SemanticsError[UndefinedSymbol]: '{}' is used before definition.",
                        &l.0
                    )
                }); */
                let exp_handle = e.generate(ctx);
                let lval_handle = (l, AsLVal::L).generate(ctx);
                let lval = ctx.value(lval_handle);
                assert!(
                    !lval.kind().is_const(),
                    "SemanticsError[InvalidLValAssignment]: '{}' cannot be assigned to.",
                    &l.0
                );
                println!(
                    "Assign \t {} = {} \t {}: {} <- {}: {}",
                    l,
                    e,
                    ctx.val_name(lval_handle),
                    ctx.value(lval_handle).ty(),
                    ctx.val_name(exp_handle),
                    ctx.value(exp_handle).ty(),
                );
                let store = ctx.add_value(val!(store(exp_handle, lval_handle)), None); // TODO 🐞
                ctx.insert_inst(store, ctx.curr());
            }
            If(exp, then, alt) => {
                let block_name_then = ctx.block_namer.gen("then");
                let block_name_else = ctx.block_namer.gen("else");
                let block_name_endif = ctx.block_namer.gen("endif");

                let block_then = ctx.add_block(&block_name_then);
                let block_endif = ctx.add_block(&block_name_endif);
                let block_else = if alt.is_none() {
                    block_endif
                } else {
                    ctx.add_block(&block_name_else)
                };

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
            }
            While(exp, body) => {
                let block_name_while = ctx.block_namer.gen("while");
                let block_name_loop = ctx.block_namer.gen("loop");
                let block_name_endwhile = ctx.block_namer.gen("endwhile");

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
                    let branch =
                        ctx.add_value(val!(branch(gate, block_loop, block_endwhile)), None);
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
            }
            Break => {
                let (_, block_dest) = ctx.curr_loop();
                let jump = ctx.add_value(val!(jump(block_dest)), None);
                ctx.insert_inst(jump, ctx.curr());
                ctx.seal_block(ctx.curr());
            }
            Continue => {
                let (block_dest, _) = ctx.curr_loop();
                let jump = ctx.add_value(val!(jump(block_dest)), None);
                ctx.insert_inst(jump, ctx.curr());
                ctx.seal_block(ctx.curr());
            }
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
        let ty: ir::Type = self.ty.to(ctx);
        if matches!(self.kind, SymKind::Const) && matches!(self.ty, Ty::Int) {
            let init_val = self.init.as_ref().map(|i| match i {
                Init::Initializer(_) => {
                    // let unevaled_shape = if let Ty::Array(a) = &self.ty { a } else { unreachable!() };
                    // let shape: Shape = unevaled_shape.eval(ctx)?.into();
                    // let shaped_initializer = ShapedInitializer(&shape, i);
                    // let evaled_aggregate = shaped_initializer.eval(ctx)?;
                    // Some(generate_aggregate(&evaled_aggregate, ctx, &shape, true))
                    unreachable!()
                }
                Init::Exp(e) => e.eval(ctx).map(|v| ctx.add_value(val!(integer(v)), None))
            }).flatten();
            let init_val = init_val.unwrap_or_else(|| panic!("SemanticsError[ConstEvalFailure]: '{}' cannot be evaluated during compile time.", self.ident));
            ctx.table_mut().insert_val(&self.ident, init_val);
        } else {
            enum InitVal<'a> {
                One(ir::Value),
                Aggregate(RawAggregate<'a>, Shape),
            }

            let init_val = match &self.init {
                Some(i) => match i {
                    Init::Initializer(i) => {
                        let unevaled_shape = if let Ty::Array(a) = &self.ty { a } else { unreachable!() };
                        let shape: Shape = unevaled_shape.eval(ctx).unwrap().into();
                        let raw_aggregate = unwrap_aggregate(i.build(&shape), &shape);
                        println!("{}", raw_aggregate);
                        InitVal::Aggregate(raw_aggregate, shape)
                    }
                    Init::Exp(e) => {
                        let val = match e.eval(ctx) {
                            Some(v) => ctx.add_value(val!(integer(v)), None),
                            None => e.generate(ctx)
                        };
                        InitVal::One(val)
                    }
                },
                None => if ty.is_i32() {
                    InitVal::One(
                        ctx.add_value(val!(zero_init(ty.clone())), None)
                    )
                } else {
                    let unevaled_shape = if let Ty::Array(a) = &self.ty { a } else { unreachable!() };
                    let shape: Shape = unevaled_shape.eval(ctx).unwrap().into();
                    let raw_aggregate = unwrap_aggregate(RawAggregate::ZeroInitWhole(0), &shape);
                    InitVal::Aggregate(raw_aggregate, shape)
                }
            };

            match init_val {
                InitVal::One(val) => {
                    let alloc = ctx.add_value(
                        val!(alloc(ty.clone())),
                        Some(format!("@{}", &self.ident)),
                    );
                    ctx.table_mut().insert_val(&self.ident, alloc);
                    ctx.insert_inst(alloc, ctx.curr());
                    let store = ctx.add_value(val!(store(val, alloc)), None);
                    ctx.insert_inst(store, ctx.curr());
                },
                InitVal::Aggregate(raw_aggregate, shape) => {
                    let alloc = ctx.add_value(val!(alloc(ty.clone())), Some(format!("@{}", &self.ident)));
                        ctx.insert_inst(alloc, ctx.curr());
                        ctx.table_mut().insert_val(&self.ident, alloc);

                        generate_aggregate(&raw_aggregate, alloc, ctx, &shape, matches!(self.kind, SymKind::Const));
                }
            };
        }
    }
}

impl<'f> Generate<'f> for (&ast::Param, ir::Value) {
    type Val = ();
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        let alloc = ctx.add_value(
            val!(alloc(ty!(i32))),
            Some(format!("@{}", &self.0.ident)),
        );
        ctx.table_mut().insert_val(&self.0.ident, alloc);
        ctx.insert_inst(alloc, ctx.curr());
        let store = ctx.add_value(val!(store(self.1, alloc)), None);
        ctx.insert_inst(store, ctx.curr());
    }
}

impl<'f> Generate<'f> for (&ast::LVal, AsLVal) {
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        let lval_handle = ctx.table().get_val(&self.0.0).unwrap_or_else(|| {
            panic!(
                "SemanticsError[UndefinedSymbol]: '{}' is used before definition.",
                &self.0
            )
        });
        let lval = ctx.value(lval_handle);
        if self.0.1.is_empty() {
            println!("LVal {}", self.0);
            if lval.kind().is_const() {
                // const or var
                return lval_handle;
            } else {
                if matches!(self.1, AsLVal::L) {
                    return lval_handle;
                } else {
                    let load = ctx.add_mid_value(val!(load(lval_handle)));
                    ctx.insert_inst(load, ctx.curr());
                    return load;
                }
            }
        } else {
            let indices = (&self.0.1).generate(ctx);
            print!("LVal[] {} -> ", self.0);
            let mut ptr = lval_handle;
            for index in indices {
                print!("[{}]", ctx.val_name(ptr));
                let get_element_ptr = ctx.add_mid_value(val!(get_elem_ptr(ptr, index)));
                ctx.insert_inst(get_element_ptr, ctx.curr());
                ptr = get_element_ptr;
            }
            println!();
            if matches!(self.1, AsLVal::L) {
                ptr
            } else {
                let load = ctx.add_mid_value(val!(load(ptr)));
                ctx.insert_inst(load, ctx.curr());
                load
            }
        }
        /*
        Aggregate(_) => {
                let mut lval = lval;
                let indices = (&self.1).eval(ctx).unwrap();
                for dim in indices {
                    lval = {
                        if let Aggregate(a) = ctx.value(lval).kind() {
                            *a.elems().get(dim as usize).unwrap()
                        } else { panic!("SemanticsError[ConstArrayIndexError]: Invalid index [{dim}].") }
                    }
                }
                lval
            }, */
    }
}

fn unwrap_aggregate<'a>(raw: RawAggregate<'a>, shape: &Shape) -> RawAggregate<'a> {
    match raw {
        RawAggregate::Agg(v) => {
            RawAggregate::Agg(v.into_iter().map(|a| unwrap_aggregate(a, shape)).collect())
        }
        RawAggregate::Value(_) => raw,
        RawAggregate::ZeroInitOne(u) | RawAggregate::ZeroInitWhole(u) => {
            use std::iter::repeat;
            if u == shape.len() { RawAggregate::ZeroInitOne(u) }
            else {
                shape[u..].iter().rev().fold(
                    RawAggregate::ZeroInitOne(shape.len()),
                    |a, i| RawAggregate::Agg(repeat(a).take(*i as usize).collect())
                )
            }
        }
    }
}

fn generate_aggregate<'f>(raw: &RawAggregate, ptr: ir::Value, ctx: &'f mut Context, shape: &Shape, should_eval: bool) -> () {
    match raw {
        RawAggregate::Agg(v) => {
            for (i, a) in v.iter().enumerate() {
                let idx = ctx.add_plain_value_integer(i as i32);
                let p = ctx.add_mid_value(val!(get_elem_ptr(ptr, idx)));
                ctx.insert_inst(p, ctx.curr());
                generate_aggregate(a, p, ctx, shape, should_eval);
            }
        }
        RawAggregate::Value(e) => {
            let val = if should_eval {
                ctx.add_plain_value_integer(e.eval(ctx).unwrap())
            } else {
                e.generate(ctx)
            };
            let store = ctx.add_value(val!(store(val, ptr)), None);
            ctx.insert_inst(store, ctx.curr());
        }
        RawAggregate::ZeroInitOne(u) => {
            let mut zero = ctx.add_plain_value_zeroinit(ty!(i32));
            let store = ctx.add_value(val!(store(zero, ptr)), None);
            ctx.insert_inst(store, ctx.curr());
        }
        RawAggregate::ZeroInitWhole(_) => unimplemented!()
    }
}

impl<'f> Generate<'f> for ast::PrimaryExp {
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> ir::Value {
        match self {
            Self::LVal(l) => (l, AsLVal::R).generate(ctx),
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

impl<'f> Generate<'f> for &Vec<ast::Exp> {
    type Val = Vec<ir::Value>;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        self.iter().map(|e| e.generate(ctx)).collect()
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
            },
            Self::Call(ident, params) => {
                let func = ctx.table().get_func(ident).unwrap_or_else(|| {
                    panic!(
                        "SemanticsError[UndefinedFunc]: '{}' is called before definition.",
                        ident
                    )
                });
                let ret_unit = match ctx.func(func).ty().kind() {
                    ir::TypeKind::Function(_, ret_ty) => ret_ty.is_unit(),
                    _ => unreachable!(),
                };
                let param_values: Vec<_> = params.iter().map(|p| p.generate(ctx)).collect();
                let call = if ret_unit {
                    ctx.add_value(val!(call(func, param_values)), None)
                } else {
                    ctx.add_mid_value(val!(call(func, param_values)))
                };
                ctx.insert_inst(call, ctx.curr());
                call
            },
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
