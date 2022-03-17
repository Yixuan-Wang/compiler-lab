use std::fmt::Display;

use super::{reg::*, Risc};

/// RISC-V 指令
/// 
/// https://pku-minic.github.io/online-doc/#/misc-app-ref/riscv-insts
pub enum RiscInst {
    /// 加载立即数
    Li(RiscReg, i32),
    Ret,
}

impl Display for RiscInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::RiscInst::*;
        match self {
            Li(r, i) => write!(f, "li {r}, {i}"),
            Ret => write!(f, "ret"),
        }
    }
}