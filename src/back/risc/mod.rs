mod inst;
mod reg;

use std::fmt::Display;

pub use inst::*;
pub use reg::*;

#[derive(Debug, Clone)]
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
    Dirc(RiscDirc),
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
            Label(l) => write!(f, "{l}:\n"),
            Dirc(d) => write!(f, "  .{d}\n"),
            Inst(i) => write!(f, "  {i}\n"),
            Comment(c) => write!(f, "# {c}\n"),
            Blank => write!(f, "\n"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum RiscDirc {
    Text,
    Data,
    Global(RiscLabel),
    Zero(i32),
    Word(i32),
}

impl Display for RiscDirc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::RiscDirc::*;
        match self {
            Text => write!(f, "text"),
            Data => write!(f, "data"),
            Global(l) => write!(f, "globl {l}"),
            Zero(z) => write!(f, "zero {z}"),
            Word(w) => write!(f, "word {w}"),
        }
    }
}