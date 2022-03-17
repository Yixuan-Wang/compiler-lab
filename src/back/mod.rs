use std::error::Error;

use crate::{front::Ir, WrapProgram};
use koopa::ir;

mod context;
mod risc;

use context::Context;
use risc::RiscItem;

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
        let mut text = vec![RiscItem::Text, RiscItem::Global("main".to_string())];
        text.extend(funcs.into_iter().flat_map(|func| {
            let ctx = Context::new(&mut program, func);
            ctx.generate()
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
//               value_data.instruct();
//             }
//         }
//     }
// }

/// [`Instruct`] 处理 [`ir::entities::ValueData`]，将每一条语句从 Koopa 内存形式转化为 RISC-V 指令
trait Instruct<'a> {
    fn instruct(&self, ctx: &'a Context) -> Vec<risc::RiscInst>;
}

impl<'a> Instruct<'a> for ir::entities::ValueData {
    fn instruct(&self, ctx: &'a Context) -> Vec<risc::RiscInst> {
        use ir::entities::ValueKind::*;
        use risc::{RiscInst as Inst, RiscReg as Reg};
        match self.kind() {
            Return(v) => match v.value() {
                Some(v) => match ctx.value(v).kind() {
                    Integer(i) => {
                        vec![Inst::Li(Reg::A0, i.value()), Inst::Ret]
                    }
                    _ => todo!(),
                },
                None => todo!(),
            },
            Integer(_) => unimplemented!("Integer is not instruction"),
            _ => todo!(),
        }
    }
}
