use std::fmt::Display;

pub enum RiscReg {
    A0,
    A1,
}

impl Display for RiscReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::RiscReg::*;
        match self {
            A0 => write!(f, "a0"),
            A1 => write!(f, "a1"),
        }
    }
}