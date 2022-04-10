pub mod ast;
#[macro_use]
mod context;
mod declare;
mod gen;
mod symtab;

use lalrpop_util::lalrpop_mod;
lalrpop_mod! {
    #[allow(clippy::all)]
    pub parser
}

use koopa::{
    back::KoopaGenerator,
    ir::Program,
};
use std::{
    error::Error,
    io,
    ops::{Deref, DerefMut},
    result,
};

use self::{symtab::{FuncTab, ValTab}, gen::prelude::with_prelude};
use self::declare::Declare;


pub fn into_ast(source: String) -> Vec<ast::Item> {
    let parser = parser::CompUnitParser::new();
    let ast = parser.parse(&source);
    ast.unwrap()
}

pub fn into_ir(source: String) -> Ir {
    let parser = parser::CompUnitParser::new();
    let ast = parser.parse(&source);
    let ir: Ir = ast.unwrap().try_into().unwrap();
    ir
}

pub fn into_ir_text(ir: Ir) -> result::Result<String, Box<dyn Error>> {
    Ok(ir.try_into()?)
}

pub struct Ir(pub Program);

impl Deref for Ir {
    type Target = Program;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<Vec<ast::Item>> for Ir {
    type Error = Box<dyn Error>;

    fn try_from(value: Vec<ast::Item>) -> result::Result<Self, Self::Error> {
        let mut program = Program::new();
        let mut func_tab = FuncTab::new();
        let mut global_val_tab = ValTab::new();
        with_prelude(&mut program, &mut func_tab);
        for item in value {
            item.declare(&mut program, &mut func_tab, &mut global_val_tab);
        }
        Ok(Ir(program))
    }
}

impl TryFrom<Ir> for String {
    type Error = io::Error;
    fn try_from(value: Ir) -> result::Result<Self, Self::Error> {
        let mut gen = KoopaGenerator::new(Vec::new());
        gen.generate_on(&value.0)?;
        Ok(std::str::from_utf8(&gen.writer()).unwrap().to_string())
    }
}
