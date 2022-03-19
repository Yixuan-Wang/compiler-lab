use std::fmt::Display;

use super::reg::RiscReg as Reg;

/// RISC-V 指令
/// 
/// https://pku-minic.github.io/online-doc/#/misc-app-ref/riscv-insts
pub enum RiscInst {
    /// 异或 `xor rd, rs1, rs2`
    Xor(Reg, Reg, Reg),
    /// 加 `add rd, rs1, rs2`
    Add(Reg, Reg, Reg),
    /// 减 `sub rd, rs1, rs2`
    Sub(Reg, Reg, Reg),
    /// 乘 `mul rd, rs1, rs2`
    Mul(Reg, Reg, Reg),
    /// 除 `div rd, rs1, rs2`
    Div(Reg, Reg, Reg),
    /// 模 `rem rd, rs1, rs2`
    Rem(Reg, Reg, Reg),
    /// 判零 `seqz rd, rs`
    Seqz(Reg, Reg),
    /// 非零 `snez rd, rs`
    Snez(Reg, Reg),
    /// 返回 `ret`
    Ret,
    /// 加载立即数 `li rd, imm`
    Li(Reg, i32),
    /// 寄存器复制 `mv rd, rs` 
    Mv(Reg, Reg),
}

impl Display for RiscInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::RiscInst::*;
        match self {
            Xor(rd, rs1, rs2) => write!(f, "xor {rd}, {rs1}, {rs2}"),
            Add(rd, rs1, rs2) => write!(f, "add {rd}, {rs1}, {rs2}"),
            Sub(rd, rs1, rs2) => write!(f, "sub {rd}, {rs1}, {rs2}"),
            Mul(rd, rs1, rs2) => write!(f, "mul {rd}, {rs1}, {rs2}"),
            Div(rd, rs1, rs2) => write!(f, "div {rd}, {rs1}, {rs2}"),
            Rem(rd, rs1, rs2) => write!(f, "rem {rd}, {rs1}, {rs2}"),
            Seqz(rd, rs) => write!(f, "seqz {rd}, {rs}"),
            Snez(rd, rs) => write!(f, "snez {rd}, {rs}"),
            Ret => write!(f, "ret"),
            Li(r, i) => write!(f, "li {r}, {i}"),
            Mv(rd, rs) => write!(f, "mv {rd}, {rs}"),
        }
    }
}