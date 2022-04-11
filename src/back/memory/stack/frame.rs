use std::collections::HashMap;

use koopa::ir;

use crate::back::{risc::RiscReg as Reg, allocate::Allocate};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum FrameObj {
    Val(ir::Value),
    Reg(Reg),
    Slot(i32),
}

impl Into<FrameObj> for ir::Value {
    fn into(self) -> FrameObj {
        FrameObj::Val(self)
    }
}

impl Into<FrameObj> for Reg {
    fn into(self) -> FrameObj {
        FrameObj::Reg(self)
    }
}

#[derive(Debug)]
/// 在栈帧上的位置，向低地址对齐（参考 README.md）
pub enum FrameAddress {
    /// 从当前栈帧高地址端向低地址计算
    High(i32),
    /// 从当前栈帧低地址端向高地址计算
    Low(i32),
    /// 从前一栈帧低地址端向高地址计算，不在本栈帧中
    Prev(i32),
}

#[derive(Debug)]
pub struct FrameMap {
    map: HashMap<FrameObj, FrameAddress>,
    size_high: i32,
    size_low: i32,
    size_prev: i32,
    size_aligned: Option<i32>,
}

impl FrameMap {
    pub fn new() -> FrameMap {
        FrameMap {
            map: HashMap::new(),
            size_high: 0,
            size_low: 0,
            size_prev: 0,
            size_aligned: None,
        }
    }

    pub fn size_aligned(&self) -> i32 {
        self.size_aligned.unwrap()
    }

    pub fn align(&mut self) {
        let size = self.size_high + self.size_low;
        self.size_aligned = Some(size + (16 - size % 16) % 16);
    }

    /// 在当前栈帧高地址端插入
    pub fn insert_high<K, V>(&mut self, k: K, v: &V)
    where
        K: Into<FrameObj>,
        V: Allocate
    {
        let size = v.allocate();
        if size > 0 {
            self.size_aligned = None;
            self.size_high += size;
            self.map.insert(k.into(), FrameAddress::High(self.size_high));
        }
    }

    /// 在当前栈帧低地址端插入，为函数调用预留的位置
    pub fn insert_low<K, V>(&mut self, k: K, v: &V)
    where
        K: Into<FrameObj>,
        V: Allocate
    {
        let size = v.allocate();
        if size > 0 {
            self.size_aligned = None;
            self.map.insert(k.into(), FrameAddress::Low(self.size_low));
            self.size_low += size;
        }
    }

    /// 指向前一栈帧低地址端，使用调用者预留的位置
    pub fn insert_prev<K, V>(&mut self, k: K, v: &V)
    where
        K: Into<FrameObj>,
        V: Allocate
    {
        let size = v.allocate();
        if size > 0 {
            self.map.insert(k.into(), FrameAddress::Prev(self.size_prev));
            self.size_prev += size;
        }
    }

    pub fn get<K>(&self, k: K) -> i32
    where K: Into<FrameObj> + std::fmt::Debug
    {
        use FrameAddress::*;
        match self.map.get(&k.into()).unwrap() {
            High(i) => self.size_aligned.unwrap() - i,
            Low(i) => *i,
            Prev(i) => self.size_aligned.unwrap() + i,
        }
    }

    // pub fn contains_key(&self, val: ir::Value) -> bool {
    //     self.stack_allo.contains_key(&val)
    // }
}

impl Default for FrameMap {
    fn default() -> FrameMap {
        FrameMap::new()
    }
}