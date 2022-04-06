pub mod back;
pub mod cli;
pub mod front;
pub mod util;

use koopa::ir;

trait WrapProgram {
    fn program(&self) -> &ir::Program;
    fn program_mut(&mut self) -> &mut ir::Program;
    fn func_handle(&self) -> ir::Function;

    fn func_mut(&mut self) -> &mut ir::FunctionData {
        let func = self.func_handle();
        self.program_mut().func_mut(func)
    }

    fn func(&self) -> &ir::FunctionData {
        self.program().func(self.func_handle())
    }

    fn dfg_mut(&mut self) -> &mut ir::dfg::DataFlowGraph {
        self.func_mut().dfg_mut()
    }

    fn dfg(&self) -> &ir::dfg::DataFlowGraph {
        self.func().dfg()
    }

    fn layout_mut(&mut self) -> &mut ir::layout::Layout {
        self.func_mut().layout_mut()
    }

    fn layout(&self) -> &ir::layout::Layout {
        self.func().layout()
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
