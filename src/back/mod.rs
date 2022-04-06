use std::error::Error;

use crate::{front::Ir, WrapProgram};

mod allo;
mod context;
mod gen;
mod risc;

use context::Context;
use risc::RiscItem as Item;

use self::gen::Generate;

pub struct Target(pub String);

pub fn into_riscv(ir: Ir) -> Result<String, Box<dyn Error>> {
    let target: Target = ir.try_into()?;
    Ok(target.0)
}

impl TryFrom<Ir> for Target {
    type Error = Box<dyn Error>;
    fn try_from(ir: Ir) -> Result<Self, Self::Error> {
        let mut program = ir.0;
        let funcs = program.func_layout().to_vec();
        let mut text = vec![Item::Text, Item::Global("main".to_string())];

        text.extend(funcs.into_iter().flat_map(|func| {
            let ctx = Context::new(&mut program, func);
            let mut insts = vec![Item::Label(ctx.name().to_string())];
            insts.extend(ctx.prologue().into_iter().map(Item::Inst));
            for (bb, node) in ctx.this_func().layout().bbs() {
                let name = ctx.bb(*bb).name().clone().unwrap();
                if name != "%entry" {
                    insts.push(Item::Label(ctx.prefix_with_name(&name)));
                }
                insts.extend(
                    node.insts()
                        .keys()
                        .flat_map(|&val| val.generate(&ctx))
                        .map(Item::Inst),
                );
            }
            insts.push(Item::Blank);
            insts
        }));

        Ok(Target(
            text.iter()
                .map(|i| format!("{i}"))
                .collect::<Vec<_>>()
                .join("\n"),
        ))
    }
}

// /// [`Declare`] 处理 Koopa AST 中的条目：全局常量、变量声明和函数，并为每一个函数生成上下文（[`Context`]）
// trait Declare<'a> {
//     fn declare(&self, program: &'a mut ir::Program);
// }

// impl<'a> Declare<'a> for ir::FunctionData {
//     fn declare(&self, program: &'a mut ir::Program) {
//         for (_, node) in self.layout().bbs() {
//             for &inst in node.insts().keys() {
//               let value_data = self.dfg().value(inst);
//               value_data.generate();
//             }
//         }
//     }
// }
