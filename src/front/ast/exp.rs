use super::*;

#[derive(Debug)]
pub struct Exp(pub LOrExp);

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

#[derive(Debug)]
pub enum UnaryExp {
    Primary(PrimaryExp),
    Unary(UnaryOp, Box<UnaryExp>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    LNot,
}
#[derive(Debug)]
pub enum MulExp {
    Unary(UnaryExp),
    Binary(Box<MulExp>, MulOp, UnaryExp),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MulOp {
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
pub enum AddExp {
    Unary(MulExp),
    Binary(Box<AddExp>, AddOp, MulExp),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddOp {
    Add,
    Sub,
}

#[derive(Debug)]
pub enum LOrExp {
    Unary(LAndExp),
    Binary(Box<LOrExp>, LAndExp),
}

#[derive(Debug)]
pub enum LAndExp {
    Unary(EqExp),
    Binary(Box<LAndExp>, EqExp),
}

#[derive(Debug)]
pub enum EqExp {
    Unary(RelExp),
    Binary(Box<EqExp>, EqOp, RelExp),
}

#[derive(Debug, PartialEq, Eq)]
pub enum EqOp {
    Eq,
    Ne,
}

#[derive(Debug)]
pub enum RelExp {
    Unary(AddExp),
    Binary(Box<RelExp>, RelOp, AddExp),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RelOp {
    Lt,
    Gt,
    Le,
    Ge,
}
