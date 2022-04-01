pub mod ast;
#[macro_use] mod context;
mod symtab;
mod gen;

use lalrpop_util::lalrpop_mod;
lalrpop_mod! {
    #[allow(clippy::all)]
    pub parser
}

use std::{result, ops::{Deref, DerefMut}, io, error::Error};
use koopa::{ir::{Program, builder_traits::*}, back::KoopaGenerator};

use self::symtab::Symtab;
use self::context::{Context};
use self::gen::Generate;

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
        let mut globals = Symtab::new();
        for item in value {
            item.declare(&mut program, &mut globals)
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

/// [`Declare`] 处理 AST 中的条目（[`ast::Item`]）：全局常量、变量声明和函数，并为每一个函数生成上下文（[`Context`]）
trait Declare<'a> {
    fn declare(&self, program: &'a mut Program, globals: &'a mut Symtab);
}

impl<'a> Declare<'a> for ast::Item {
    fn declare(&self, program: &'a mut Program, globals: &'a mut Symtab) {
        use ast::ItemKind::*;
        match self.kind {
            Global() => unimplemented!(),
            Func(ref func) => {
                let mut ctx = Context::new(program, globals, &func);
                // let cur = ctx.add_block("temp");
                // ctx.insert_block(cur);
                // let jump_cur = ctx.add_value(val!(jump(cur)), None);
                // ctx.insert_inst(jump_cur, ctx.entry());
                for stmt in &func.block {
                    stmt.generate(&mut ctx);
                }
            }
        };
    }
}