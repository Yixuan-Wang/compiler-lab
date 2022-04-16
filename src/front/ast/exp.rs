use std::fmt::Display;

use super::*;

#[derive(Debug)]
pub struct Exp(pub LOrExp);

impl Display for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    Literal(i32),
    LVal(LVal),
}

impl PrimaryExp {
    pub fn literal(src: &str, radix: u32, prefix_len: usize) -> PrimaryExp {
        PrimaryExp::Literal(
            i32::from_str_radix(unsafe { src.get_unchecked(prefix_len..) }, radix).unwrap(),
        )
    }
}

impl Display for PrimaryExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PrimaryExp::*;
        match self {
            Exp(e) => write!(f, "({e})"),
            Literal(i) => write!(f, "{i}"),
            LVal(l) => write!(f, "{l}"),
        }
    }
}

#[derive(Debug)]
pub enum UnaryExp {
    Primary(PrimaryExp),
    Unary(UnaryOp, Box<UnaryExp>),
    Call(String, Vec<Box<Exp>>),
}

impl Display for UnaryExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnaryExp::*;
        match self {
            Primary(p) => write!(f, "{p}"),
            Unary(o, u) => write!(f, "{o}{u}"),
            Call(l, e) => {
                write!(f, "{}", l)?;
                f.debug_tuple("").field(e).finish()?;
                write!(f, "")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    LNot,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnaryOp::*;
        match self {
            Minus => write!(f, "-"),
            LNot => write!(f, "!"),
        }
    }
}

#[derive(Debug)]
pub enum MulExp {
    Unary(UnaryExp),
    Binary(Box<MulExp>, MulOp, UnaryExp),
}

impl Display for MulExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MulExp::*;
        match self {
            Unary(e) => write!(f, "{e}"),
            Binary(l, o, r) => write!(f, "{l} {o} {r}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MulOp {
    Mul,
    Div,
    Mod,
}

impl Display for MulOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MulOp::*;
        match self {
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
            Mod => write!(f, "%"),
        }
    }
}

#[derive(Debug)]
pub enum AddExp {
    Unary(MulExp),
    Binary(Box<AddExp>, AddOp, MulExp),
}

impl Display for AddExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AddExp::*;
        match self {
            Unary(e) => write!(f, "{e}"),
            Binary(l, o, r) => write!(f, "{l} {o} {r}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddOp {
    Add,
    Sub,
}

impl Display for AddOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AddOp::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
        }
    }
}

#[derive(Debug)]
pub enum LOrExp {
    Unary(LAndExp),
    Binary(Box<LOrExp>, LAndExp),
}

impl Display for LOrExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LOrExp::*;
        match self {
            Unary(e) => write!(f, "{e}"),
            Binary(l, r) => write!(f, "{l} || {r}"),
        }
    }
}

#[derive(Debug)]
pub enum LAndExp {
    Unary(EqExp),
    Binary(Box<LAndExp>, EqExp),
}

impl Display for LAndExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LAndExp::*;
        match self {
            Unary(e) => write!(f, "{e}"),
            Binary(l, r) => write!(f, "{l} && {r}"),
        }
    }
}

#[derive(Debug)]
pub enum EqExp {
    Unary(RelExp),
    Binary(Box<EqExp>, EqOp, RelExp),
}

impl Display for EqExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EqExp::*;
        match self {
            Unary(e) => write!(f, "{e}"),
            Binary(l, o, r) => write!(f, "{l} {o} {r}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EqOp {
    Eq,
    Ne,
}

impl Display for EqOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EqOp::*;
        match self {
            Eq => write!(f, "=="),
            Ne => write!(f, "!="),
        }
    }
}

#[derive(Debug)]
pub enum RelExp {
    Unary(AddExp),
    Binary(Box<RelExp>, RelOp, AddExp),
}

impl Display for RelExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RelExp::*;
        match self {
            Unary(e) => write!(f, "{e}"),
            Binary(l, o, r) => write!(f, "{l} {o} {r}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RelOp {
    Lt,
    Gt,
    Le,
    Ge,
}

impl Display for RelOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RelOp::*;
        match self {
            Lt => write!(f, "<"),
            Gt => write!(f, ">"),
            Le => write!(f, "<="),
            Ge => write!(f, ">="),
        }
    }
}
