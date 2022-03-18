use koopa::ir;
use std::{ops::{Deref, DerefMut}};

#[derive(Debug)]
pub struct Item {
    pub kind: ItemKind
}

#[derive(Debug)]
pub enum ItemKind {
    /// Global const/variable declaration
    Global(),

    /// Function declaration
    Func(Func),
}

#[derive(Debug)]
pub struct Func {
    pub ident: String,
    pub output: Ty,
    pub block: Vec<Stmt>,
}

impl Func {
    pub fn new(ident: String, output: String, block: Vec<Stmt>) -> Func {
        Func {
            ident,
            output: Ty::new(&output),
            block,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    Int,
    Void,
}

impl Ty {
    pub fn new(ty: &str) -> Ty {
        dbg!(&ty);
        match ty {
            "int" => Ty::Int,
            "void" => Ty::Void,
            _ => unreachable!()
        }
    }
}

impl From<&Ty> for ir::Type {
    fn from(t: &Ty) -> Self {
        match t {
            Ty::Int => ir::Type::get_i32(),
            Ty::Void => ir::Type::get_unit(),
        }
    }
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
}

impl Deref for Stmt {
    type Target = StmtKind;
    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl DerefMut for Stmt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

#[derive(Debug)]
pub enum StmtKind {
    Return(Exp),
}

// pub fn to_int_literal<'ip>(src: &'ip str, radix: u32, prefix_len: usize) -> i32 {
//     unimplemented!()
// }

#[derive(Debug)]
pub struct Exp(pub AddExp);

#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    Literal(i32),
}

impl PrimaryExp {
    pub fn literal<'ip>(src: &'ip str, radix: u32, prefix_len: usize) -> PrimaryExp {
        PrimaryExp::Literal(i32::from_str_radix(unsafe { &src.get_unchecked(prefix_len..) }, radix).unwrap())
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
