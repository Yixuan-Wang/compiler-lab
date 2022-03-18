use koopa::ir;
use crate::WrapProgram;

pub struct Context<'a> {
    pub program: &'a mut ir::Program,
    func: ir::Function,
}

impl<'a> WrapProgram for Context<'a> {
    fn program(&self) -> &ir::Program { self.program }
    fn program_mut(&mut self) -> &mut ir::Program { self.program }
    fn func_handle(&self) -> ir::Function { self.func }
}

impl<'a> Context<'a> {
    pub fn new(program: &'a mut ir::Program, func: ir::Function) -> Context {
        Context { 
            program,
            func,
        }
    }
}
