use crate::WrapProgram;

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
where C: WrapProgram + FetchVal<'f>
{
    fn eval(&self, ctx: &'f C) -> Option<T>;
}

impl<'f, C> Eval<'f, C, i32> for Exp
where C: WrapProgram + FetchVal<'f>
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        self.0.eval(ctx)
    }
}

impl<'f, C> Eval<'f, C, i32> for LOrExp
where C: WrapProgram + FetchVal<'f>
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
where C: WrapProgram + FetchVal<'f>
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
where C: WrapProgram + FetchVal<'f>
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
where C: WrapProgram + FetchVal<'f>
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
where C: WrapProgram + FetchVal<'f>
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
where C: WrapProgram + FetchVal<'f>
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
where C: WrapProgram + FetchVal<'f>
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
where C: WrapProgram + FetchVal<'f>
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
where C: WrapProgram + FetchVal<'f>
{
    fn eval(&self, ctx: &'f C) -> Option<i32> {
        use koopa::ir::entities::ValueKind;
        if let Some(v) = ctx.fetch_val(&self.0) {
            println!("{} found: {:?}", &self.0, v);
            println!("kind: {:?}", ctx.fetch_val_kind(v));
        }
        match ctx.fetch_val(&self.0) {
            Some(v) => match ctx.fetch_val_kind(v) {
                ValueKind::Integer(v) => Some(v.value()),
                ValueKind::GlobalAlloc(a) => {
                    match ctx.fetch_val_kind(a.init()) {
                        ValueKind::Integer(v) => Some(v.value()),
                        _ => unreachable!(),
                    }
                }
                _ => None,
            },
            None => None,
        }
    }
}
