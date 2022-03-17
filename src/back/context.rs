use koopa::ir;

use crate::WrapProgram;
use super::{Generate, risc::{RiscItem as Item, RiscInst as Inst, RiscReg as Reg}};

pub struct Context<'a> {
    pub program: &'a mut ir::Program,
    func: ir::Function
}

impl<'a> WrapProgram for Context<'a> {
    fn program(&self) -> &ir::Program { self.program }
    fn program_mut(&mut self) -> &mut ir::Program { self.program }
    fn func_handle(&self) -> ir::Function { self.func }
}

impl<'a> Context<'a> {
    pub fn new(program: &'a mut ir::Program, func: ir::Function) -> Context {
        Context { program, func }
    }

    pub fn generate(&self) -> Vec<Item> {
        let func_data = self.func();
        let name = unsafe { func_data.name().get_unchecked(1..) };
        dbg!(name);
        let mut insts = vec![
            Item::Label(name.to_string())
        ];
        for (_bb, node) in func_data.layout().bbs() {
            insts.extend(
                node.insts()
                    .keys()
                    .map(|&key| self.dfg().value(key))
                    .flat_map(|val| val.generate(self))
                    .map(|inst| Item::Inst(inst))
            );
        };
        insts.push(Item::Blank);
        insts
    }

}
