use koopa::ir::{Type, Value};

use crate::WrapProgram;

use crate::front::context::AddPlainValue;
use crate::front::{ast::*, symtab::FetchVal};

macro_rules! eval {
    (( $l:expr, $r:expr ) => $f:expr) => {
        match ($l, $r) {
            (Some(x), Some(y)) => Some($f(x, y)),
            _ => None,
        }
    };
}

pub trait Eval<'f, C, T>
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<T>;
}

impl<'f, C> Eval<'f, C, i32> for Exp
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        self.0.eval(ctx)
    }
}

impl<'f, C> Eval<'f, C, i32> for LOrExp
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, r) => {
                eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x | y != 0 { 1 } else { 0 })
            }
        }
    }
}

impl<'f, C> Eval<'f, C, i32> for LAndExp
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, r) => {
                eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x != 0 && y != 0 { 1 } else { 0 })
            }
        }
    }
}

impl<'f, C> Eval<'f, C, i32> for EqExp
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, o, r) => match o {
                EqOp::Eq => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x == y { 1 } else { 0 }),
                EqOp::Ne => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x != y { 1 } else { 0 }),
            },
        }
    }
}

impl<'f, C> Eval<'f, C, i32> for RelExp
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, o, r) => match o {
                RelOp::Lt => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x < y { 1 } else { 0 }),
                RelOp::Gt => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x > y { 1 } else { 0 }),
                RelOp::Le => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x <= y { 1 } else { 0 }),
                RelOp::Ge => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x >= y { 1 } else { 0 }),
            },
        }
    }
}

impl<'f, C> Eval<'f, C, i32> for AddExp
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, o, r) => match o {
                AddOp::Add => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x + y),
                AddOp::Sub => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x - y),
            },
        }
    }
}

impl<'f, C> Eval<'f, C, i32> for MulExp
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, o, r) => match o {
                MulOp::Mul => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x * y),
                MulOp::Div => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x / y),
                MulOp::Mod => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x % y),
            },
        }
    }
}

impl<'f, C> Eval<'f, C, i32> for UnaryExp
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        match self {
            Self::Primary(e) => e.eval(ctx),
            Self::Unary(o, v) => match o {
                UnaryOp::Minus => v.eval(ctx).map(|x: i32| -x),
                UnaryOp::LNot => v.eval(ctx).map(|x| if x != 0 { 1 } else { 0 }),
            },
            Self::Call(..) => None,
        }
    }
}

impl<'f, C> Eval<'f, C, i32> for PrimaryExp
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        match self {
            Self::Exp(e) => e.eval(ctx),
            Self::Literal(i) => Some(*i),
            Self::LVal(l) => l.eval(ctx),
        }
    }
}

impl<'f, C> Eval<'f, C, i32> for LVal
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        fn eval<'f, C>(o: Option<Value>, ctx: &'f C) -> Option<i32>
        where
            C: WrapProgram + FetchVal<'f>,
        {
            use koopa::ir::entities::ValueKind::*;
            match o {
                Some(v) => match ctx.fetch_val_kind(v) {
                    Integer(i) => Some(i.value()),
                    // 编译期不对数组求值
                    // Aggregate(ag) => {
                    //     let shape: Shape = (&ctx.fetch_val_type(v)).try_into().unwrap();
                    //     let x: usize = shape.index(indices).try_into().unwrap();
                    //     ag.elems().get(x).map(|val| eval(Some(*val), ctx, indices)).flatten()
                    // }
                    GlobalAlloc(a) => eval(Some(a.init()), ctx),
                    _ => None,
                },
                None => {
                    panic!("SemanticsError[UndefinedSymbol]: A symbol is used before definition.")
                }
            }
        }
        // let indices = (&self.1).eval(ctx);
        match ctx.fetch_val(&self.0) {
            v @ Some(_) => eval(v, ctx),
            None => panic!(
                "SemanticsError[UndefinedSymbol]: '{}' is used before definition.",
                &self.0
            ),
        }
    }
}

impl<'f, C> Eval<'f, C, Vec<i32>> for &Vec<Exp>
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<Vec<i32>> {
        let shape: Vec<_> = self.iter().map(|e| e.eval(ctx)).collect();
        if !shape.iter().all(Option::is_some) {
            None
        } else {
            Some(shape.into_iter().map(Option::unwrap).collect::<Vec<i32>>())
        }
    }
}

impl<'f, C> Eval<'f, C, EvaledAggregate> for ShapedInitializer<'_>
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<EvaledAggregate> {
        let array = self.1.build(self.0);
        array.eval(ctx)
    }
}

impl<'f, C> Eval<'f, C, EvaledAggregate> for RawAggregate<'_>
where
    C: WrapProgram + FetchVal<'f>,
{
    fn eval(&self, ctx: &'f C) -> Option<EvaledAggregate> {
        match self {
            RawAggregate::Agg(v) => {
                let v: Option<Vec<_>> = v.iter().map(|u| u.eval(ctx)).collect();
                Some(EvaledAggregate::Agg(v?))
            }
            RawAggregate::Value(e) => Some(EvaledAggregate::Value(e.eval(ctx)?)),
            RawAggregate::ZeroInitOne(u) | RawAggregate::ZeroInitWhole(u) => {
                Some(EvaledAggregate::ZeroInit(*u))
            }
        }
    }
}

pub fn generate_evaled_aggregate<'a, 'f: 'a, 'b: 'a, C>(
    agg: &'a EvaledAggregate,
    ctx: &'f mut C,
    tys: &'b [Type],
) -> Value
where
    C: AddPlainValue,
{
    use EvaledAggregate::*;
    match agg {
        Agg(v) => {
            let v = v
                .iter()
                .map(|a| generate_evaled_aggregate(a, ctx, tys))
                .collect();
            ctx.add_plain_value_aggregate(v)
        }
        Value(v) => ctx.add_plain_value_integer(*v),
        ZeroInit(u) => ctx.add_plain_value_zeroinit(tys[*u].clone()),
    }
}
