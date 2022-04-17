use std::fmt::Display;

use crate::back::risc::MAX_IMM;

use super::{reg::RiscReg as Reg, RiscLabel};

#[allow(dead_code)]
/// RISC-V 指令
///
/// https://pku-minic.github.io/online-doc/#/misc-app-ref/riscv-insts
pub enum RiscInst {
    /// 按位与 `and rd, rs1, rs2`
    And(Reg, Reg, Reg),
    /// 按位或 `and rd, rs1, rs2`
    Or(Reg, Reg, Reg),
    /// 按位异或 `xor rd, rs1, rs2`
    Xor(Reg, Reg, Reg),
    /// 按位与立即数 `andi rd, rs, imm12`
    Andi(Reg, Reg, i32),
    /// 按位或立即数 `ori rd, rs, imm12`
    Ori(Reg, Reg, i32),
    /// 按位异或立即数 `xori rd, rs, imm12`
    Xori(Reg, Reg, i32),
    /// 小于 `slt rd, rs1, rs2`
    Slt(Reg, Reg, Reg),
    /// 大于 `sgt rd, rs1, rs2`
    Sgt(Reg, Reg, Reg),
    /// 加 `add rd, rs1, rs2`
    Add(Reg, Reg, Reg),
    /// 加立即数 `addi rd, rs, imm12`
    Addi(Reg, Reg, i32),
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
    /// 判零转移 `beqz rs, label`
    Beqz(Reg, RiscLabel),
    /// 非零转移 `bnez rs, label`
    Bnez(Reg, RiscLabel),
    /// 无条件转移 `j label`
    J(RiscLabel),
    /// 返回 `ret`
    Ret,
    /// 调用函数 `call label`
    Call(RiscLabel),
    /// 加载立即数 `li rd, imm`
    Li(Reg, i32),
    /// 寄存器复制 `mv rd, rs`
    Mv(Reg, Reg),
    /// 标签地址 data `la rd, label`
    La(Reg, RiscLabel),
    /// 取 `lw rs, imm12(rd)`（取 `rd+imm12` 存入 `rs`）
    Lw(Reg, i32, Reg),
    /// 存 `sw rs2, imm12(rs1)`（存 `rs2` 入 `rs1+imm2`）
    Sw(Reg, i32, Reg),

    /// 注释
    Com(String),
}

impl Display for RiscInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::RiscInst::*;
        match self {
            And(rd, rs1, rs2) => write!(f, "and {rd}, {rs1}, {rs2}"),
            Or(rd, rs1, rs2) => write!(f, "or {rd}, {rs1}, {rs2}"),
            Xor(rd, rs1, rs2) => write!(f, "xor {rd}, {rs1}, {rs2}"),
            Andi(rd, rs, i) => write!(f, "andi {rd}, {rs}, {i}"),
            Ori(rd, rs, i) => write!(f, "ori {rd}, {rs}, {i}"),
            Xori(rd, rs, i) => write!(f, "xori {rd}, {rs}, {i}"),
            Slt(rd, rs1, rs2) => write!(f, "slt {rd}, {rs1}, {rs2}"),
            Sgt(rd, rs1, rs2) => write!(f, "sgt {rd}, {rs1}, {rs2}"),
            Add(rd, rs1, rs2) => write!(f, "add {rd}, {rs1}, {rs2}"),
            Addi(rd, rs, i) => write!(f, "addi {rd}, {rs}, {i}"),
            Sub(rd, rs1, rs2) => write!(f, "sub {rd}, {rs1}, {rs2}"),
            Mul(rd, rs1, rs2) => write!(f, "mul {rd}, {rs1}, {rs2}"),
            Div(rd, rs1, rs2) => write!(f, "div {rd}, {rs1}, {rs2}"),
            Rem(rd, rs1, rs2) => write!(f, "rem {rd}, {rs1}, {rs2}"),
            Seqz(rd, rs) => write!(f, "seqz {rd}, {rs}"),
            Snez(rd, rs) => write!(f, "snez {rd}, {rs}"),
            Beqz(rs, label) => write!(f, "beqz {rs}, {label}"),
            Bnez(rs, label) => write!(f, "bnez {rs}, {label}"),
            Ret => write!(f, "ret"),
            Call(label) => write!(f, "call {label}"),
            J(label) => write!(f, "j {label}"),
            Li(r, i) => write!(f, "li {r}, {i}"),
            Mv(rd, rs) => write!(f, "mv {rd}, {rs}"),
            La(rd, l) => write!(f, "la {rd}, {l}"),
            Lw(rs, of, rd) => write!(f, "lw {rs}, {of}({rd})"),
            Sw(rs2, of, rs1) => write!(f, "sw {rs2}, {of}({rs1})"),

            Com(c) => write!(f, "# {c}"),
        }
    }
}

impl RiscInst {
    pub fn expand_imm(self) -> Vec<RiscInst> {
        use RiscInst::*;
        match self {
            Lw(rs, of, rd) => {
                if of > MAX_IMM {
                    vec![
                        Li(Reg::T(0), of),
                        Add(Reg::T(0), rd, Reg::T(0)),
                        Lw(rs, 0, Reg::T(0)),
                    ]
                } else {
                    vec![self]
                }
            }
            Sw(rs2, of, rs1) => {
                if of > MAX_IMM {
                    vec![
                        Li(Reg::T(0), of),
                        Add(Reg::T(0), rs1, Reg::T(0)),
                        Sw(rs2, 0, Reg::T(0)),
                    ]
                } else {
                    vec![self]
                }
            }
            Addi(rd, rs, imm) => {
                if imm > MAX_IMM {
                    vec![Li(Reg::T(0), imm), Add(rd, rs, Reg::T(0))]
                } else {
                    vec![self]
                }
            }
            _ => unimplemented!(),
        }
    }
}
