mod inst;
mod reg;

use std::fmt::Display;

pub use inst::*;
pub use reg::*;

pub struct RiscLabel(String);

impl Display for RiscLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RiscLabel {
    pub fn new<T>(string: T) -> RiscLabel
    where T: ToString
    {
        RiscLabel(string.to_string())
    }

    pub fn strip<T>(string: T) -> RiscLabel
    where T: ToString
    {
        let mut string = string.to_string();
        string.remove(0);
        RiscLabel(string)
    }

    pub fn with_prefix<T>(self, prefix: T) -> RiscLabel
    where T: Display
    {
        RiscLabel(format!("{}_{}", prefix, self.0))
    }
}

#[allow(dead_code)]
pub enum RiscItem {
    Text,
    Data,
    Global(RiscLabel),
    Label(RiscLabel),
    Inst(RiscInst),
    Comment(String),
    Blank,
}

pub const MAX_IMM: i32 = 2047;

impl Display for RiscItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::RiscItem::*;
        match self {
            Global(l) => write!(f, "  .globl {l}\n"),
            Text => write!(f, "  .text\n"),
            Data => write!(f, "  .data\n"),
            Label(l) => write!(f, "{l}:\n"),
            Inst(i) => write!(f, "  {i}\n"),
            Comment(c) => write!(f, "# {c}\n"),
            Blank => write!(f, "\n"),
        }
    }
}
