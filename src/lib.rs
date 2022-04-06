pub mod back;
pub mod cli;
pub mod front;
pub mod util;

use koopa::ir;

trait WrapProgram {
    fn program(&self) -> &ir::Program;
    fn program_mut(&mut self) -> &mut ir::Program;
    fn this_func_handle(&self) -> ir::Function;

    fn func(&self, func: ir::Function) -> &ir::FunctionData {
        self.program().func(func)
    }

    fn func_mut(&mut self, func: ir::Function) -> &mut ir::FunctionData {
        self.program_mut().func_mut(func)
    }

    fn this_func(&self) -> &ir::FunctionData {
        self.func(self.this_func_handle())
    }

    fn this_func_mut(&mut self) -> &mut ir::FunctionData {
        let func = self.this_func_handle();
        self.func_mut(func)
    }

    fn dfg_mut(&mut self) -> &mut ir::dfg::DataFlowGraph {
        self.this_func_mut().dfg_mut()
    }

    fn dfg(&self) -> &ir::dfg::DataFlowGraph {
        self.this_func().dfg()
    }

    fn layout_mut(&mut self) -> &mut ir::layout::Layout {
        self.this_func_mut().layout_mut()
    }

    fn layout(&self) -> &ir::layout::Layout {
        self.this_func().layout()
    }

    fn bb(&self, bb: ir::BasicBlock) -> &ir::entities::BasicBlockData {
        self.dfg().bb(bb)
    }

    fn bb_mut(&mut self, bb: ir::BasicBlock) -> &mut ir::entities::BasicBlockData {
        self.dfg_mut().bb_mut(bb)
    }

    fn bb_node(&self, bb: ir::BasicBlock) -> &ir::layout::BasicBlockNode {
        self.layout().bbs().node(&bb).expect("`bb` does not exist")
    }

    fn bb_node_mut(&mut self, bb: ir::BasicBlock) -> &mut ir::layout::BasicBlockNode {
        self.layout_mut().bb_mut(bb)
    }

    fn value(&self, value: ir::Value) -> &ir::entities::ValueData {
        self.dfg().value(value)
    }
}
