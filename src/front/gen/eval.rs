use crate::WrapProgram;

use super::{ast::*, Context};

macro_rules! eval {
    (( $v:expr ) => $f:expr) => {
        match ($v) {
            Some(v) => Some($f(v)),
            _ => None,
        }
    };
    (( $l:expr, $r:expr ) => $f:expr) => {
        match ($l, $r) {
            (Some(x), Some(y)) => Some($f(x, y)),
            _ => None,
        }
    };
}

pub trait Eval<'f, T> {
    fn eval(&self, ctx: &'f Context) -> Option<T>;
}

impl<'f> Eval<'f, i32> for Exp {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        self.0.eval(ctx)
    }
}

impl<'f> Eval<'f, i32> for LOrExp {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, r) => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x | y != 0 { 1 } else { 0 }),
        }
    }
}

impl<'f> Eval<'f, i32> for LAndExp {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, r) => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x != 0 && y != 0 { 1 } else { 0 })
        }
    }
}

impl<'f> Eval<'f, i32> for EqExp {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, o, r) => match o {
                EqOp::Eq => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x == y { 1 } else { 0 }),
                EqOp::Ne => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x != y { 1 } else { 0 }),
            }
        }
    }
}

impl<'f> Eval<'f, i32> for RelExp {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, o, r) => match o {
                RelOp::Lt => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x < y { 1 } else { 0 }),
                RelOp::Gt => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x > y { 1 } else { 0 }),
                RelOp::Le => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x <= y { 1 } else { 0 }),
                RelOp::Ge => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| if x >= y { 1 } else { 0 }),
            }
        }
    }
}

impl<'f> Eval<'f, i32> for AddExp {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, o, r) => match o {
                AddOp::Add => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x + y),
                AddOp::Sub => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x - y),
            }
        }
    }
}

impl<'f> Eval<'f, i32> for MulExp {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        match self {
            Self::Unary(e) => e.eval(ctx),
            Self::Binary(l, o, r) => match o {
                MulOp::Mul => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x * y),
                MulOp::Div => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x / y),
                MulOp::Mod => eval!((l.eval(ctx), r.eval(ctx)) => |x, y| x % y),
            }
        }
    }
}

impl<'f> Eval<'f, i32> for UnaryExp {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        match self {
            Self::Primary(e) => e.eval(ctx),
            Self::Unary(o, v) => match o {
                UnaryOp::Minus => eval!((v.eval(ctx)) => |x: i32| -x),
                UnaryOp::LNot => eval!((v.eval(ctx)) => |x| if x != 0 { 1 } else { 0 }),
            }
        }
    }
}

impl<'f> Eval<'f, i32> for PrimaryExp {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        match self {
            Self::Exp(e) => e.eval(ctx),
            Self::Literal(i) => Some(*i),
            Self::LVal(l) => l.eval(ctx),
        }
    }
}

impl<'f> Eval<'f, i32> for LVal {
    fn eval(&self, ctx: &'f Context) -> Option<i32> {
        use koopa::ir::entities::ValueKind;
        match ctx.table().get_val(&self.0) {
            Some(v) => {
                match ctx.value(v).kind() {
                    ValueKind::Integer(v) => Some(v.value()),
                    _ => None,
                }
            },
            None => None,
        }
    }
}
 