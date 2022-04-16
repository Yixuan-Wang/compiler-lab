use super::{gen::eval::Eval, symtab::FetchVal};
use koopa::ir;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Item {
    pub kind: ItemKind,
}

#[derive(Debug)]
pub enum ItemKind {
    /// Global const/variable declaration
    Global(Vec<Decl>),

    /// Function declaration
    Func(Func),
}

#[derive(Debug)]
pub struct Func {
    pub ident: String,
    pub output: Ty,
    pub params: Vec<Param>,
    pub block: Block,
}

impl Func {
    pub fn new(ident: String, output: Ty, params: Vec<Param>, block: Block) -> Func {
        Func {
            ident,
            output,
            params,
            block,
        }
    }
}

#[derive(Debug)]
pub enum Ty {
    Int,
    Array(Vec<Exp>),
    Void,
}

impl Ty {
    pub fn new(ty: &str) -> Ty {
        match ty {
            "int" => Ty::Int,
            "void" => Ty::Void,
            _ => unreachable!(),
        }
    }
}

impl From<&Ty> for ir::Type {
    fn from(t: &Ty) -> Self {
        match t {
            Ty::Int => ty!(i32),
            Ty::Void => ty!(()),
            Ty::Array(_) => unimplemented!(),
        }
    }
}

impl Ty {
    pub fn to<'a, C>(&self, ctx: &'a C) -> ir::Type
    where
        C: WrapProgram + FetchVal<'a>,
    {
        match self {
            Ty::Int => ty!(i32),
            Ty::Void => ty!(()),
            Ty::Array(d) => {
                let dim = d.eval(ctx).expect("SemanticsError[ArrayTypeFailure]: Array type cannot be evaluated during compile time.");
                let shape: Shape = dim.into();
                shape.try_into().unwrap()
            }
        }
    }
}

#[derive(Debug)]
pub enum BlockItem {
    Stmt(Stmt),
    Decl(Vec<Decl>),
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
    Unit,
    Exp(Exp),
    Block(Block),
    Assign(LVal, Exp),
    If(Exp, Box<Stmt>, Option<Box<Stmt>>),
    While(Exp, Box<Stmt>),
    Break,
    Continue,
    Return(Option<Exp>),
}

// pub fn to_int_literal<'ip>(src: &'ip str, radix: u32, prefix_len: usize) -> i32 {
//     unimplemented!()
// }

#[derive(Debug)]
pub enum SymKind {
    Var,
    Const,
}

#[derive(Debug)]
pub struct Block(pub Vec<BlockItem>);

#[derive(Debug)]
pub struct Decl {
    pub ident: String,
    pub ty: Ty,
    pub kind: SymKind,
    pub init: Option<Init>,
}

#[derive(Debug)]
pub enum Init {
    Initializer(Initializer),
    Exp(Exp),
}

#[derive(Debug)]
pub enum Def {
    Value(String, Option<Exp>),
    Array(String, Vec<Exp>, Option<Initializer>),
}

impl Decl {
    pub fn from(def: Def, kind: SymKind) -> Self {
        match def {
            Def::Value(ident, exp) => Decl {
                ident,
                init: exp.map(Init::Exp),
                ty: Ty::Int,
                kind,
            },
            Def::Array(ident, dim, init) => Decl {
                ident,
                init: init.map(Init::Initializer),
                ty: Ty::Array(dim),
                kind,
            },
        }
    }
}

#[derive(Debug)]
pub struct LVal(pub String, pub Vec<Exp>);

#[derive(Debug)]
pub enum AsLVal {
    L,
    R,
}

impl Display for LVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)?;
        self.1
            .iter()
            .map(|e| write!(f, "[{}]", e))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Param {
    pub ident: String,
    pub ty: Ty,
}

use crate::{ty, util::shape::Shape, WrapProgram};
mod exp;
pub use exp::*;

mod initializer;
pub use initializer::*;
