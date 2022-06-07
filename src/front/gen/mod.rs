use std::iter::zip;

use crate::front::ast::Ty;

use crate::util::shape::Shape;
use crate::{ty, WrapProgram};

use crate::front::{
    ast::{self},
    context::Context,
};
use koopa::ir::{self, builder_traits::*};

pub mod eval;
use self::eval::Eval;

pub mod prelude;

pub mod lazy;
pub use lazy::*;

use super::ast::{AsLVal, Init, RawAggregate};
use super::context::AddPlainValue;

/// [`Generate`] 在 AST 结点上指令式地生成对应 Koopa 内存形式
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
            Self::Decl(v) => v.iter().for_each(|d| d.generate(ctx)),
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
                    None => ctx.add_value(val!(ret(None)), None),
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
            let init_val = self.init.as_ref().and_then(|i| match i {
                Init::Initializer(_) => {
                    // let unevaled_shape = if let Ty::Array(a) = &self.ty { a } else { unreachable!() };
                    // let shape: Shape = unevaled_shape.eval(ctx)?.into();
                    // let shaped_initializer = ShapedInitializer(&shape, i);
                    // let evaled_aggregate = shaped_initializer.eval(ctx)?;
                    // Some(generate_aggregate(&evaled_aggregate, ctx, &shape, true))
                    unreachable!()
                }
                Init::Exp(e) => e.eval(ctx).map(|v| ctx.add_value(val!(integer(v)), None)),
            });
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
                        let unevaled_shape = if let Ty::Array(a) = &self.ty {
                            a
                        } else {
                            unreachable!()
                        };
                        let shape: Shape = unevaled_shape.eval(ctx).unwrap().into();
                        let raw_aggregate = i.build(&shape);// unwrap_aggregate(, &shape);
                        InitVal::Aggregate(raw_aggregate, shape)
                    }
                    Init::Exp(e) => {
                        let val = match e.eval(ctx) {
                            Some(v) => ctx.add_value(val!(integer(v)), None),
                            None => e.generate(ctx),
                        };
                        InitVal::One(val)
                    }
                },
                None => {
                    if ty.is_i32() {
                        InitVal::One(ctx.add_value(val!(zero_init(ty.clone())), None))
                    } else {
                        let unevaled_shape = if let Ty::Array(a) = &self.ty {
                            a
                        } else {
                            unreachable!()
                        };
                        let shape: Shape = unevaled_shape.eval(ctx).unwrap().into();
                        let raw_aggregate = RawAggregate::ZeroInitWhole(0);
                            // unwrap_aggregate(, &shape);
                        InitVal::Aggregate(raw_aggregate, shape)
                    }
                }
            };

            match init_val {
                InitVal::One(val) => {
                    let alloc =
                        ctx.add_value(val!(alloc(ty.clone())), Some(format!("@{}", &self.ident)));
                    ctx.table_mut().insert_val(&self.ident, alloc);
                    ctx.insert_inst(alloc, ctx.curr());
                    let store = ctx.add_value(val!(store(val, alloc)), None);
                    ctx.insert_inst(store, ctx.curr());
                }
                InitVal::Aggregate(raw_aggregate, shape) => {
                    let alloc =
                        ctx.add_value(val!(alloc(ty.clone())), Some(format!("@{}", &self.ident)));
                    ctx.insert_inst(alloc, ctx.curr());
                    ctx.table_mut().insert_val(&self.ident, alloc);
                    let zero_init = ctx.add_plain_value_zeroinit(ty);
                    let wipe = ctx.add_value(val!(store(zero_init, alloc)), None);
                    ctx.insert_inst(wipe, ctx.curr());

                    generate_aggregate(
                        &raw_aggregate,
                        alloc,
                        ctx,
                        &shape,
                        matches!(self.kind, SymKind::Const),
                    );
                }
            };
        }
    }
}

impl<'f> Generate<'f> for (&ast::Param, ir::Value) {
    type Val = ();
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        dbg!(ctx.value(self.1).ty());
        dbg!(self.0);
        let ty = ctx.value(self.1).ty().clone();
        let alloc = ctx.add_value(val!(alloc(ty)), Some(format!("@{}", &self.0.ident)));
        ctx.table_mut().insert_val(&self.0.ident, alloc);
        ctx.insert_inst(alloc, ctx.curr());
        let store = ctx.add_value(val!(store(self.1, alloc)), None);
        ctx.insert_inst(store, ctx.curr());
    }
}

impl<'f> Generate<'f> for (&ast::LVal, AsLVal) {
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        use crate::front::symtab::SymVal;
        let lval_handle = ctx.table().get_symval(&self.0 .0).unwrap_or_else(|| {
            panic!(
                "SemanticsError[UndefinedSymbol]: '{}' is used before definition.",
                &self.0
            )
        });
        let lval_handle = match lval_handle {
            SymVal::Val(v) => v,
            SymVal::GlobalConst(i) => ctx.add_plain_value_integer(i),
        };
        let lval = ctx.value(lval_handle);
        let lval_ty = lval.ty();
        if lval_ty == &ty!(i32) || lval_ty == &ty!(*i32) {
            println!("LVal is int");
            if lval.kind().is_const() || matches!(self.1, AsLVal::L) {
                lval_handle
            } else {
                println!("Load LVal::R {}: {}", ctx.val_name(lval_handle), lval_ty);
                let load = ctx.add_mid_value(val!(load(lval_handle)));
                ctx.insert_inst(load, ctx.curr());
                load
            }
        } else {
            let indices = (&self.0 .1).generate(ctx);
            let mut ptr = lval_handle;
            for index in indices {
                use ir::TypeKind::*;
                let ptr_to = if let Pointer(t) = ctx.value(ptr).ty().kind() { t.clone() } else { unreachable!() };
                let get_some_ptr = if matches!(ptr_to.kind(), Array(..)) {
                    ctx.add_mid_value(val!(get_elem_ptr(ptr, index)))
                } else {
                    let load = ctx.add_mid_value(val!(load(ptr)));
                    ctx.insert_inst(load, ctx.curr());
                    ctx.add_mid_value(val!(get_ptr(load, index)))
                };
                println!("ptr: {} {:?} {}", ctx.val_name(get_some_ptr), ctx.value(get_some_ptr).kind(), ctx.value(get_some_ptr).ty());
                ctx.insert_inst(get_some_ptr, ctx.curr());
                ptr = get_some_ptr;
            }
            if matches!(self.1, AsLVal::L) {
                ptr
            } else {
                use ir::TypeKind::*;
                let ptr_to = if let Pointer(t) = ctx.value(ptr).ty().kind() { t.clone() } else { unreachable!() };
                if let Array(..) = ptr_to.kind() {
                    // 得到 *[T; N], 则需要 *T 而非 [T; N]
                    // getelemptr *[T; 0] -> *T
                    let zero = ctx.zero;
                    println!("should arr_ptr_to_ptr: {}", ctx.value(ptr).ty());
                    let arr_ptr_to_ptr = ctx.add_mid_value(val!(get_elem_ptr(ptr, zero)));
                    println!("arr_ptr_to_ptr: {}", ctx.value(arr_ptr_to_ptr).ty());
                    ctx.insert_inst(arr_ptr_to_ptr, ctx.curr());
                    arr_ptr_to_ptr
                } else {
                    let load = ctx.add_mid_value(val!(load(ptr)));
                    ctx.insert_inst(load, ctx.curr());
                    load
                }
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

/* fn unwrap_aggregate<'a>(raw: RawAggregate<'a>, shape: &Shape) -> RawAggregate<'a> {
    match raw {
        RawAggregate::Agg(v) => {
            RawAggregate::Agg(v.into_iter().map(|a| unwrap_aggregate(a, shape)).collect())
        }
        RawAggregate::Value(_) => raw,
        RawAggregate::ZeroInitOne(u) | RawAggregate::ZeroInitWhole(u) => {
            use std::iter::repeat;
            if u == shape.len() {
                RawAggregate::ZeroInitOne(u)
            } else {
                shape[u..]
                    .iter()
                    .rev()
                    .fold(RawAggregate::ZeroInitOne(shape.len()), |a, i| {
                        RawAggregate::Agg(repeat(a).take(*i as usize).collect())
                    })
            }
        }
    }
} */

fn generate_aggregate<'f>(
    raw: &RawAggregate,
    ptr: ir::Value,
    ctx: &'f mut Context,
    shape: &Shape,
    should_eval: bool,
) {
    match raw {
        RawAggregate::Agg(v) => {
            for (i, a) in v.iter().enumerate() {
                if !matches!(a, RawAggregate::ZeroInitOne(_) | RawAggregate::ZeroInitWhole(_)) {
                    let idx = ctx.add_plain_value_integer(i as i32);
                    let p = ctx.add_mid_value(val!(get_elem_ptr(ptr, idx)));
                    ctx.insert_inst(p, ctx.curr());
                    generate_aggregate(a, p, ctx, shape, should_eval);
                }
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
        RawAggregate::ZeroInitOne(_) | RawAggregate::ZeroInitWhole(_) => {
            /* let zero = ctx.add_plain_value_zeroinit(ty!(i32));
            let store = ctx.add_value(val!(store(zero, ptr)), None);
            ctx.insert_inst(store, ctx.curr()); */
        }
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
            }
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
                for (p, v) in zip(params.iter(), param_values.iter()) {
                    println!("{}: {}", p, ctx.value(*v).ty())
                }
                println!("{}", ctx.func(func).ty());
                let call = if ret_unit {
                    ctx.add_value(val!(call(func, param_values)), None)
                } else {
                    ctx.add_mid_value(val!(call(func, param_values)))
                };
                ctx.insert_inst(call, ctx.curr());
                call
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
