use std::{error::Error, cell::RefCell};

use crate::{front::Ir, WrapProgram};

mod allocate;
mod context;
mod gen;
mod memory;
mod risc;

use context::Context;
use self::{gen::Generate, memory::stack::StackMap, risc::{RiscItem as Item, RiscLabel, RiscDirc as Dirc}};

pub struct Target(pub String);

pub fn into_riscv(ir: Ir) -> Result<String, Box<dyn Error>> {
    let target: Target = ir.try_into()?;
    Ok(target.0)
}

impl TryFrom<Ir> for Target {
    type Error = Box<dyn Error>;
    fn try_from(ir: Ir) -> Result<Self, Self::Error> {
        let mut program = ir.0;
        let mut stack = RefCell::new(StackMap::new());
        let mut code = vec![];
        
        let data = program.borrow_values();
        code.extend(data.iter().flat_map(|(_, d)| {
            use koopa::ir::ValueKind::*;
            if let GlobalAlloc(a) = d.kind() {
                let label = RiscLabel::strip(d.name().clone().unwrap());
                let mut v = vec![
                    Item::Dirc(Dirc::Data),
                    Item::Dirc(Dirc::Global(label.clone())),
                    Item::Label(label),
                ];
                v.push(
                match program.borrow_value(a.init()).kind() {
                    Integer(i) => Item::Dirc(Dirc::Word(i.value())),
                    Undef(_) | ZeroInit(_) => Item::Dirc(Dirc::Zero(4)),
                    _ => unreachable!()
                });
                v.push(Item::Blank);
                v
            } else {
                vec![]
            }
        }));
        drop(data);

        let funcs = program.func_layout().to_vec();
        code.extend(funcs.into_iter().flat_map(|func| {
            if program.func(func).layout().entry_bb().is_none() {
                return vec![]
            }

            let ctx = Context::new(&mut program, &mut stack, func);
            let mut insts = vec![
                Item::Dirc(Dirc::Text),
                Item::Dirc(Dirc::Global(RiscLabel::new(ctx.name()))),
                Item::Label(RiscLabel::new(ctx.name())),
            ];
            ctx.stack_mut().new_frame(func);
            insts.extend(ctx.prologue().into_iter().map(Item::Inst));
            for (bb, node) in ctx.this_func().layout().bbs() {
                let name = ctx.bb(*bb).name().clone().unwrap();
                if name != "%entry" {
                    insts.push(Item::Label(ctx.label(&name)));
                }
                insts.extend(
                    node.insts()
                        .keys()
                        .flat_map(|&val| val.generate(&ctx))
                        .map(Item::Inst),
                );
            }
            insts.push(Item::Label(RiscLabel::new("end").with_prefix(ctx.name())));
            insts.extend(ctx.epilogue().into_iter().map(Item::Inst));
            insts.push(Item::Blank);
            insts
        }));

        Ok(Target(
            code.iter()
                .map(|i| i.to_string())
                .collect::<String>()
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
