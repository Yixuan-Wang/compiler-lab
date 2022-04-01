use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiscReg {
    A(u8),
    T(u8),
    Sp,
    X0,
}

impl Display for RiscReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::RiscReg::*;
        match self {
            A(i) => write!(f, "a{i}"),
            T(i) => write!(f, "t{i}"),
            Sp => write!(f, "sp"),
            X0 => write!(f, "x0"),
        }
    }
}