pub mod frame;

use std::collections::HashMap;

pub use frame::*;
use koopa::ir;

pub struct StackMap {
    stack: HashMap<ir::Function, FrameMap>
}

impl StackMap {
    pub fn new() -> StackMap {
        StackMap {
            stack: HashMap::new()
        }
    }

    /// 获得栈帧的引用
    pub fn frame(&self, func: ir::Function) -> &FrameMap {
        self.stack.get(&func).unwrap()
    }

    /// 获得栈帧的可变引用
    pub fn frame_mut(&mut self, func: ir::Function) -> &mut FrameMap {
        self.stack.get_mut(&func).unwrap()
    }

    /// 插入一个新的栈帧
    pub fn new_frame(&mut self, func: ir::Function) {
        self.stack.insert(func, FrameMap::default());
    }
}

impl Default for StackMap {
    fn default() -> Self {
        StackMap::new()
    }
}