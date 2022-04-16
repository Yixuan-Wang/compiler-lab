use std::{collections::HashSet, error::Error};

use koopa::ir::{self, builder_traits::*};

use crate::{util::autonum::Autonum, WrapProgram};

use super::symtab::{Symtab, FuncTab, ValTab, FetchVal};

/// Context is a high-level [`koopa::ir::Program`] wrapper around a [`koopa::ir::Function`]
/// with its symbol table [`Table`].
pub struct Context<'a> {
    pub program: &'a mut ir::Program,
    // pub globals: &'a mut Symtab,
    pub func: ir::Function,
    table: Symtab<'a>,
    loop_stack: Vec<(ir::BasicBlock, ir::BasicBlock)>,
    pub variable_namer: Autonum,
    pub block_namer: Autonum,
    sealed: HashSet<ir::BasicBlock>,
    entry: Option<ir::BasicBlock>,
    // end: Option<ir::BasicBlock>,
    curr: Option<ir::BasicBlock>,
    pub zero: ir::Value,
    pub one: ir::Value,
}

#[macro_export]
macro_rules! val {
    ($t:ident ( $($e: expr),* )) => {
        |b| { b.$t($($e,)*) }
    }
}

impl<'a> WrapProgram for Context<'a> {
    fn program(&self) -> &ir::Program {
        self.program
    }
    fn program_mut(&mut self) -> &mut ir::Program {
        self.program
    }
    fn this_func_handle(&self) -> ir::Function {
        self.func
    }
}

impl<'a> FetchVal<'a> for Context<'a> {
    fn fetch_val(&self, name: &str) -> Option<ir::Value> {
        self.table().get_val(name)
    }

    fn fetch_val_kind(&self, val: ir::Value) -> ir::entities::ValueKind {
        self.value(val).kind().clone()
    }

    fn fetch_val_type(&self, val: ir::Value) -> ir::TypeKind {
        self.value(val).ty().kind().clone()
    }
}

impl<'a: 'f, 'f> Context<'a> {
    pub fn new(
        program: &'a mut ir::Program,
        func_tab: &'a mut FuncTab,
        global_val_tab: &'a mut ValTab,
        func: ir::Function,
    ) -> Context<'a> {
        let mut this = Context::from(program, func_tab, global_val_tab, func).unwrap();
        this.init();
        this
    }

    fn from(
        program: &'a mut ir::Program,
        func_tab: &'a mut FuncTab,
        global_val_tab: &'a mut ValTab,
        func: ir::Function,
    ) -> Result<Self, Box<dyn Error>> {
        // let ty: ir::Type = (&func.output).into();
        // let ty_kind = ty.kind().clone();
        // let block = func.block;

        let dfg_handle = program.func_mut(func).dfg_mut();
        let zero = dfg_handle.new_value().integer(0);
        let one = dfg_handle.new_value().integer(1);

        Ok(Context {
            program,
            // globals,
            func,
            entry: None,
            // end: None,
            curr: None,
            zero,
            one,
            sealed: HashSet::new(),
            table: Symtab::new(func_tab, global_val_tab),
            loop_stack: Vec::new(),
            variable_namer: Autonum::new(),
            block_namer: Autonum::new(),
        })
    }

    /// Init the function wrapped inside the context
    /// Must be called after [`Context::from`]
    fn init(&mut self) {
        let entry = self.add_block("entry");
        // let end = self.add_block("end");
        self.insert_block(entry);
        // self.insert_block(end);
        /* let ret_alloc = match self.kind() {
            Int32 => {
                let val = self.add_value(val!(alloc(ty!(i32))), Some("%ret"));
                self.insert_inst(val, entry);
                self.table.Mod("%ret".to_string(), val);
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
        self.curr = Some(entry);
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
        if name.is_some() {
            self.dfg_mut().set_value_name(val, name);
        }
        val
    }
    
    pub fn add_mid_value<F>(&mut self, builder_fn: F) -> ir::Value
    where
        F: FnOnce(ir::builder::LocalBuilder) -> ir::Value,
    {
        let name = self.variable_namer.gen_temp();
        self.add_value(builder_fn, Some(format!("%{}", name)))
    }

    pub fn insert_block(&mut self, block: ir::BasicBlock) {
        self.layout_mut().bbs_mut().push_key_back(block).unwrap();
    }

    pub fn seal_block(&mut self, block: ir::BasicBlock) {
        self.sealed.insert(block);
    }

    pub fn insert_inst(&mut self, val: ir::Value, block: ir::BasicBlock) {
        if !self.sealed.contains(&block) {
            self.layout_mut()
                .bb_mut(block)
                .insts_mut()
                .push_key_back(val)
                .unwrap();
        } else {
            let ghost_block_name = self.block_namer.gen("ghost");
            let ghost_block = self.add_block(&ghost_block_name);
            self.insert_block(ghost_block);
            self.set_curr(ghost_block);
            self.insert_inst(val, self.curr());
        }
    }

    pub fn val_name(&self, v: ir::Value) -> String {
        self.value(v).name().clone().unwrap_or("{?}".to_string())
    }

    /* pub fn kind(&self) -> &ir::TypeKind {
        match self.func().ty().kind() {
            ir::TypeKind::Function(_, out) => out.kind(),
            _ => unreachable!(),
        }
    } */

    /// Return the entry block.
    /* pub fn entry(&self) -> ir::BasicBlock {
        self.entry.unwrap()
    } */

    /// Return the end block.
    /* pub fn end(&self) -> ir::BasicBlock {
        // self.end.unwrap()
        unimplemented!()
    } */

    /// Return the latest block
    /* pub fn latest_block(&self) -> ir::BasicBlock {
        *self.layout()
             .bbs()
             .back_key()
             .unwrap()
    } */

    /// Return the current block
    pub fn curr(&self) -> ir::BasicBlock {
        self.curr.unwrap()
    }

    /// Set current block
    pub fn set_curr(&mut self, bb: ir::BasicBlock) {
        self.curr = Some(bb);
    }

    /// Return the current symbol table
    pub fn table(&self) -> &Symtab {
        &self.table
    }

    /// Return the mutable variant of current symbol table
    pub fn table_mut(&mut self) -> &mut Symtab<'a> {
        &mut self.table
    }

    pub fn enter_loop(&mut self, loop_blocks: (ir::BasicBlock, ir::BasicBlock)) {
        self.loop_stack.push(loop_blocks)
    }

    pub fn exit_loop(&mut self) {
        self.loop_stack.pop();
    }

    pub fn curr_loop(&mut self) -> (ir::BasicBlock, ir::BasicBlock) {
        *self.loop_stack.last().unwrap()
    }
}

pub struct GlobalContext<'a> {
    pub program: &'a mut ir::Program,
    global: &'a mut ValTab,
}

impl<'a> WrapProgram for GlobalContext<'a> {
    fn program(&self) -> &ir::Program {
        self.program
    }
    fn program_mut(&mut self) -> &mut ir::Program {
        self.program
    }
    fn this_func_handle(&self) -> ir::Function {
        unimplemented!("Global context contains no function")
    }
}

impl<'a> FetchVal<'a> for GlobalContext<'a> {
    fn fetch_val(&self, name: &str) -> Option<ir::Value> {
        self.global.get(name).cloned()
    }

    fn fetch_val_kind(&self, val: ir::Value) -> ir::entities::ValueKind {
        self.program.borrow_value(val).kind().clone()
    }

    fn fetch_val_type(&self, val: ir::Value) -> ir::TypeKind {
        self.program.borrow_value(val).ty().kind().clone()
    }
}

impl<'a> GlobalContext<'a> {
    pub fn new(
        program: &'a mut ir::Program,
        global_val_tab: &'a mut ValTab,
    ) -> GlobalContext<'a> {
        GlobalContext {
            program,
            global: global_val_tab,
        }
    }

    pub fn add_global_value<F>(&mut self, builder_fn: F, name: Option<String>) -> ir::Value
    where
        F: FnOnce(ir::builder::GlobalBuilder) -> ir::Value,
    {
        let val = builder_fn(self.program_mut().new_value());
        if name.is_some() {
            self.program_mut().set_value_name(val, name);
        }
        val
    }

    pub fn register_global_value(&mut self, name: &str, value: ir::Value) {
        self.global
            .insert(name.to_string(), value);
    }
}

pub trait AddPlainValue {
    fn add_plain_value_integer(&mut self, val: i32) -> ir::Value;
    fn add_plain_value_aggregate(&mut self, elems: Vec<ir::Value>) -> ir::Value;
    fn add_plain_value_zeroinit(&mut self, ty: ir::Type) -> ir::Value;
}

impl<'a> AddPlainValue for Context<'a> {
    fn add_plain_value_integer(&mut self, val: i32) -> ir::Value {
        self.add_value(val!(integer(val)), None)
    }

    fn add_plain_value_aggregate(&mut self, elems: Vec<ir::Value>) -> ir::Value {
        self.add_value(val!(aggregate(elems)), None)
    }

    fn add_plain_value_zeroinit(&mut self, ty: ir::Type) -> ir::Value {
        self.add_value(val!(zero_init(ty)), None)
    }
}

impl<'a> AddPlainValue for GlobalContext<'a> {
    fn add_plain_value_integer(&mut self, val: i32) -> ir::Value {
        self.add_global_value(val!(integer(val)), None)
    }

    fn add_plain_value_aggregate(&mut self, elems: Vec<ir::Value>) -> ir::Value {
        self.add_global_value(val!(aggregate(elems)), None)
    }

    fn add_plain_value_zeroinit(&mut self, ty: ir::Type) -> ir::Value {
        self.add_global_value(val!(zero_init(ty)), None)
    }
}
