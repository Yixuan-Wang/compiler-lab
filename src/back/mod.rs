use std::{cell::RefCell, error::Error, fmt::Display, borrow::Cow};

use crate::{front::Ir, util::merge::merge, WrapProgram};

mod allocate;
mod context;
mod gen;
mod memory;
mod risc;
mod perf;

use self::{
    gen::Generate,
    memory::stack::StackMap,
    risc::{RiscDirc as Dirc, RiscItem as Item, RiscLabel},
};
use context::Context;
use koopa::ir::{self};

pub struct Target {
    items: Vec<Item>,
    perf: bool
}

pub fn into_riscv(ir: Ir, perf: bool) -> Result<String, Box<dyn Error>> {
    let mut target: Target = ir.try_into()?;
    target.perf = perf;
    Ok(target.into())
}

impl From<Target> for String {
    fn from(target: Target) -> Self {
        use crate::back::perf::peephole::Peephole;

        if target.perf {
            target.items
                .remove_unnecessary_load_store()
                .iter()
                .map(|item| item.to_string())
                .collect()
        } else {
            target.items.iter().map(|i| i.to_string()).collect::<String>()
        }
    }
}

#[test]
fn test_window() {
    let x = [1, 2, 3, 4, 5, 6];
    let v: Vec<_> = x.as_slice().windows(2).map(|x| x.get(1).filter(|x| *x % 2 == 0)).flatten().collect();
    dbg!(v);
}

impl TryFrom<Ir> for Target {
    type Error = Box<dyn Error>;
    fn try_from(ir: Ir) -> Result<Self, Self::Error> {
        koopa::ir::Type::set_ptr_size(4);

        let mut program = ir.0;
        let mut stack = RefCell::new(StackMap::new());
        let mut code = vec![];

        let data = program.borrow_values();
        code.extend(
            data.iter()
                .filter(|(_, d)| matches!(d.kind(), koopa::ir::ValueKind::GlobalAlloc(_)))
                .flat_map(|(_h, d)| {
                    use koopa::ir::ValueKind::*;
                    let a = if let GlobalAlloc(a) = d.kind() {
                        a
                    } else {
                        unreachable!()
                    };
                    let label = RiscLabel::strip(d.name().clone().unwrap());
                    let mut v = vec![
                        Item::Dirc(Dirc::Data),
                        Item::Dirc(Dirc::Global(label.clone())),
                        Item::Label(label),
                    ];
                    if matches!(program.borrow_value(a.init()).kind(), Aggregate(_)) {
                        let stack = vec![];
                        let agg = write_aggregate(a.init(), stack, &program);
                        dbg!(&agg);
                        v.extend(
                            merge(
                                agg.into_iter(),
                                |d| {
                                    if let Dirc::Zero(z) = d {
                                        Some(*z)
                                    } else {
                                        None
                                    }
                                },
                                |acc, step| acc + step,
                                Dirc::Zero,
                            )
                            .map(Item::Dirc),
                        )
                    } else {
                        v.push(match program.borrow_value(a.init()).kind() {
                            Integer(i) => Item::Dirc(Dirc::Word(i.value())),
                            Undef(_) | ZeroInit(_) => Item::Dirc(Dirc::Zero(
                                program
                                    .borrow_value(a.init())
                                    .ty()
                                    .size()
                                    .try_into()
                                    .unwrap(),
                            )),
                            _ => unreachable!(),
                        });
                    }
                    v.push(Item::Blank);
                    v
                }),
        );
        drop(data);

        let funcs = program.func_layout().to_vec();
        code.extend(funcs.into_iter().flat_map(|func| {
            if program.func(func).layout().entry_bb().is_none() {
                return vec![];
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

        Ok(Target{
            items: code,
            perf: false,
        })
    }
}

fn write_aggregate(val: ir::Value, mut stack: Vec<Dirc>, program: &ir::Program) -> Vec<Dirc> {
    use ir::ValueKind::*;
    // let global_alloc = program.borrow_value(val);
    // let val = if let GlobalAlloc(a) = global_alloc.kind() {
    //     a.init()
    // } else { unreachable!("{:?}\n", global_alloc.kind()) };
    let data = program.borrow_value(val);

    match data.kind() {
        Aggregate(a) => {
            stack = a
                .elems()
                .iter()
                .fold(stack, |acc, elem| write_aggregate(*elem, acc, program));
        }
        Integer(i) => {
            stack.push(Dirc::Word(i.value()));
        }
        ZeroInit(_) | Undef(_) => {
            stack.push(Dirc::Zero(data.ty().size().try_into().unwrap()));
        }
        _ => unreachable!("{:?}\n", data.kind()),
    };
    stack
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
