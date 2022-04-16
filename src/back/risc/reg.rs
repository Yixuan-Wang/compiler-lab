use std::fmt::Display;

#[allow(dead_code)]
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiscReg {
    /// 函数参数/返回值，调用者保存
    ///
    /// - `x10-11` `a0-1` 函数参数/返回值
    /// - `x12-17` `a2-7` 函数参数
    A(u8),
    /// 临时寄存器，调用者保存
    ///
    /// - `x5-7` `t0-2`
    /// - `x28-31` `t3-6`
    T(u8),
    /// `x0`, 恒为 0
    Zero,
    /// `x1`, 返回地址
    Ra,
    /// `x2`，栈指针，调用者保存
    Sp,
}

impl Display for RiscReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::RiscReg::*;
        match self {
            A(i) => write!(f, "a{i}"),
            T(i) => write!(f, "t{i}"),
            Zero => write!(f, "zero"),
            Ra => write!(f, "ra"),
            Sp => write!(f, "sp"),
        }
    }
}
