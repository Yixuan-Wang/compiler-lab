use std::error::Error;

use koopa::ir::{
    self,
    builder_traits::*,
};

use crate::{WrapProgram, util::autonum::Autonum};

use super::{ast, table::Table};

/// Context is a high-level [`koopa::ir::Program`] wrapper around a [`koopa::ir::Function`]
/// with its symbol table [`Table`].
pub struct Context<'a> {
    pub program: &'a mut ir::Program,
    pub globals: &'a mut Table,
    pub func: ir::Function,
    pub table: Table,
    pub auton: Autonum,
    entry: Option<ir::BasicBlock>,
    end: Option<ir::BasicBlock>,
}

#[macro_export]
macro_rules! val {
    ($t:ident ( $($e: expr),* )) => {
        |b| { b.$t($($e,)*) }
    }
}

impl<'a> WrapProgram for Context<'a> {
    fn program(&self) -> &ir::Program { self.program }
    fn program_mut(&mut self) -> &mut ir::Program { self.program }
    fn func_handle(&self) -> ir::Function { self.func }
}

impl<'a: 'f, 'f> Context<'a> {
    pub fn new(
        program: &'a mut ir::Program,
        globals: &'a mut Table,
        func: &'f ast::Func,
    ) -> Context<'a> {
        let mut this = Context::from(program, globals, func).unwrap();
        this.init();
        this
    }

    fn from(
        program: &'a mut ir::Program,
        globals: &'a mut Table,
        func: &'f ast::Func,
    ) -> Result<Self, Box<dyn Error>> {
        // let ty: ir::Type = (&func.output).into();
        // let ty_kind = ty.kind().clone();
        // let block = func.block;

        let func_data = ir::FunctionData::new(
            format!("@{}", func.ident),
         vec![], 
         (&func.output).into(),
        );
        let func = program.new_func(func_data);

        Ok(Context {
            program,
            globals,
            func,
            entry: None,
            end: None,
            table: Table::new(),
            auton: Autonum::new(),
        })
    }

    /// Init the function wrapped inside the context
    /// Must be called after [`Context::from`]
    fn init(&mut self) {
        use ir::Type;
        use ir::TypeKind::*;
        let entry = self.add_block("entry");
        // let end = self.add_block("end");
        self.insert_block(entry);
        // self.insert_block(end);
        /* let ret_alloc = match self.kind() {
            Int32 => {
                let val = self.add_value(val!(alloc(Type::get_i32())), Some("%ret"));
                self.insert_inst(val, entry);
                self.table.insert_var("%ret".to_string(), val);
                Some(val)
            }
            Unit => None,
            _ => unimplemented!(),
        }; */

        /* if let Some(ret_alloc) = ret_alloc {
            let ret_load = self.add_value(val!(load(ret_alloc)), Some("%ret_load"));
            let ret_ret = self.add_value(val!(ret(Some(ret_load))), None);
            self.insert_inst(ret_load, end);
            self.insert_inst(ret_ret, end);
        } else {
            let ret_ret = self.add_value(val!(ret(None)), None);
            self.insert_inst(ret_ret, end);
        } */

        self.entry = Some(entry);
        // self.end = Some(end);
    }

    /* pub fn func_mut(&mut self) -> &mut ir::FunctionData {
        self.program.func_mut(self.func)
    }

    pub fn func(&self) -> &ir::FunctionData {
        self.program.func(self.func)
    }

    pub fn dfg_mut(&mut self) -> &mut ir::dfg::DataFlowGraph {
        self.func_mut().dfg_mut()
    }

    pub fn dfg(&self) -> &ir::dfg::DataFlowGraph {
        self.func().dfg()
    }

    pub fn layout_mut(&mut self) -> &mut ir::layout::Layout {
        self.func_mut().layout_mut()
    }

    pub fn layout(&self) -> &ir::layout::Layout {
        self.func().layout()
    } */

    pub fn add_block(&mut self, name: &str) -> ir::BasicBlock {
        self.dfg_mut()
            .new_bb()
            .basic_block(Some(format!("%{}", name)))
    }

    pub fn add_value<F>(&mut self, builder_fn: F, name: Option<String>) -> ir::Value
    where
        F: FnOnce(ir::builder::LocalBuilder) -> ir::Value,
    {
        let val = builder_fn(self.dfg_mut().new_value());
        if let Some(_) = name {
            self.dfg_mut()
                .set_value_name(val, name);
        }
        val
    }

    pub fn add_mid_value<F>(&mut self, builder_fn: F) -> ir::Value
    where
        F: FnOnce(ir::builder::LocalBuilder) -> ir::Value
    {
        let name = self.auton.gen(None);
        self.add_value(builder_fn, Some(name))
    }

    pub fn insert_block(&mut self, block: ir::BasicBlock) {
        self.layout_mut()
            .bbs_mut()
            .push_key_back(block)
            .unwrap();
    }

    pub fn insert_inst(&mut self, val: ir::Value, block: ir::BasicBlock) {
        self.layout_mut()
            .bb_mut(block)
            .insts_mut()
            .push_key_back(val)
            .unwrap();
    }

    pub fn kind(&self) -> &ir::TypeKind {
        match self.func().ty().kind() {
            ir::TypeKind::Function(_, out) => out.kind(),
            _ => unreachable!(),
        }
    }

    /// Return the entry block. 
    pub fn entry(&self) -> ir::BasicBlock {
        self.entry.unwrap()
    }

    /// Return the end block. 
    pub fn end(&self) -> ir::BasicBlock {
        // self.end.unwrap()
        unimplemented!()
    }

    /// Return the current block
    pub fn curr(&self) -> ir::BasicBlock {
        *self.layout()
             .bbs()
             .back_key()
             .unwrap()
    }
}