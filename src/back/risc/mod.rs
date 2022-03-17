mod inst;
mod reg;

use std::fmt::Display;

pub use inst::*;
pub use reg::*;

pub struct Risc(Vec<RiscItem>);

pub enum RiscItem {
    Text,
    Data,
    Global(String),
    Label(String),
    Inst(RiscInst),
    Blank,
}

impl Display for RiscItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::RiscItem::*;
        match self {
            Global(l) => write!(f, "  .globl {l}"),
            Text => write!(f, "  .text"),
            Data => write!(f, "  .data"),
            Label(l) => write!(f, "{l}:"),
            Inst(i) => write!(f, "  {i}"),
            Blank => write!(f, ""),
        }
    }
}