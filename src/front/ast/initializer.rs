use std::{iter, fmt::{Display, Debug}};

use crate::{front::ast::exp::Exp, util::shape::Shape};

#[derive(Debug)]
pub enum Initializer {
    List(Vec<Initializer>),
    Value(Exp)
}

#[derive(Clone)]
pub enum RawAggregate<'a> {
    Agg(Vec<RawAggregate<'a>>),
    ZeroInitOne(usize),
    ZeroInitWhole(usize),
    Value(&'a Exp),
}

impl Debug for RawAggregate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Agg(arg0) => f.debug_set().entries(arg0.iter()).finish(),
            Self::ZeroInitOne(arg0) => write!(f, "0O{{{arg0}}}"),
            Self::ZeroInitWhole(arg0) => write!(f, "0W{{{arg0}}}"),
            Self::Value(arg0) => write!(f, "{arg0}"),
        }
    }
}

pub enum EvaledAggregate
{
    Agg(Vec<EvaledAggregate>),
    ZeroInit(usize),
    Value(i32),
}

pub enum GeneratedAggregate
{
    ZeroInit(usize),
    Value(i32),
}

impl<'a> Display for RawAggregate<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawAggregate::Agg(v) => {
                write!(f, "{{")?;
                let w: Result<Vec<_>, _> = v.iter().enumerate().map(|(i, a)| {
                    if i == 0 { write!(f, "{a}") }
                    else { write!(f, ", {a}") }
                }).collect();
                w?;
                write!(f, "}}")?;
            }
            RawAggregate::Value(e) => write!(f, "{e}")?,
            RawAggregate::ZeroInitOne(_) => write!(f, "{{}}")?,
            RawAggregate::ZeroInitWhole(_) => write!(f, "{{}}")?,
        }
        Ok(())
    }
}

pub struct ShapedInitializer<'a>(pub &'a Shape, pub &'a Initializer);

impl Initializer {
    pub fn build<'a, 'b>(&'a self, shape: &'b Shape) -> RawAggregate<'a> {
        let mut h: InitializerStack<'a, 'b> = InitializerStack::new(&shape);
        // let mut dest = vec![];
        self.fill(&mut h, );
        h.try_carry(0);
        dbg!(&h);
        assert_eq!(h.stack.len(), 1);
        let x = h.stack.pop().unwrap();
        x
    }

    fn fill<'a: 'c, 'b, 'c>(&'a self, h: &'c mut InitializerStack<'a, 'b>/* , d: &'b mut Vec<Option<&'a Exp>> */) {
        match &self {
            Initializer::Value(e) => {
                if h.progress.len() < h.carrying.len() {
                    h.status = InitializerState::Fill;
                    while h.progress.len() < h.carrying.len() {
                        h.progress.push(0);
                    }
                }

                h.stack.push(RawAggregate::Value(e));
                *h.progress.last_mut().unwrap() += 1;

                if matches!(h.status, InitializerState::Fill) && h.should_carry() {
                    h.status = InitializerState::Pad;
                    h.carry();
                }
            }
            Initializer::List(v) => {
                let prev_status = h.status;
                if matches!(h.status, InitializerState::Pad) {
                    h.progress.push(0);
                }
                let enter_level = h.progress.len() - 1;

                for l in v {
                    l.fill(h);
                }

                println!("108 {:?} @ {:?}", &h.stack, &h.progress);

                if v.is_empty() && (matches!(h.status, InitializerState::Fill) || h.progress.len() > h.carrying.len()) {
                    if !matches!(h.status, InitializerState::Fill) {
                        h.progress.pop();
                    }
                    h.stack.push(RawAggregate::ZeroInitOne(h.progress.len()));

                    *h.progress.last_mut().unwrap() += 1;

                    if matches!(h.status, InitializerState::Fill) && h.should_carry() {
                        h.status = InitializerState::Pad;
                    }
                }
 
                if prev_status != h.status {
                    h.status = InitializerState::Pad;
                }

                if matches!(h.status, InitializerState::Pad) {
                    h.try_carry(enter_level);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitializerState {
    Pad,
    Fill,
}

#[derive(Debug)]
struct InitializerStack<'a, 'b> {
    status: InitializerState,
    pub stack: Vec<RawAggregate<'a>>,
    progress: Vec<i32>,
    carrying: &'b Shape,
    // shape: Shape,
}

// fn carrying(shape: &Shape) -> Vec<i32> {
//     let mut s = vec![];
//     let mut p = 1;
//     shape.iter().rev().for_each(|i| {
//         p *= i;
//         s.push(p);
//     });
//     s.push(*s.last().unwrap());
//     s.reverse();
//     s
// }

impl<'a, 'b> InitializerStack<'a, 'b> {
    fn new(shape: &'b Shape) -> InitializerStack<'a, 'b> {
        InitializerStack { 
            status: InitializerState::Pad,
            stack: vec![],
            progress: vec![],
            carrying: shape,//carrying(shape),
            // shape: shape.clone()
        }
    }

    fn try_carry(&mut self, enter_level: usize) {
        if self.should_pad(enter_level) {
            let pad = self.pad();
            let pad_u: usize = pad.try_into().unwrap();
            if self.progress.last() != Some(&0) {
                self.stack.extend(iter::repeat(RawAggregate::ZeroInitOne(self.progress.len())).take(pad_u));
            } else {
                self.stack.push(RawAggregate::ZeroInitWhole(self.progress.len() - 1))
            }
            *self.progress.last_mut().unwrap() += pad;
        }
        while self.should_carry() {
            self.carry();
        }
    }

    fn carry(&mut self) {
        assert!(matches!(self.status, InitializerState::Pad));
        // let t_s = self;
        // dbg!(&self);
        let level = self.progress.len() - 1;
        let is_zero_init_whole = if let Some(RawAggregate::ZeroInitWhole(u)) = self.stack.last() {
            level == *u
        } else { false };
        let aggregate = if is_zero_init_whole {
            self.stack.pop();
            RawAggregate::ZeroInitWhole(self.progress.len() - 1)
        } else {
            let carry_len: usize = (*self.carrying.get(self.progress.len() - 1).unwrap()).try_into().unwrap();
            RawAggregate::Agg(self.stack.drain(self.stack.len()-carry_len..).collect())
        };
        self.progress.pop();
        self.stack.push(aggregate);
        self.progress.last_mut().and_then(|x| Some(*x += 1));//p));
    }

    fn is_not_full(&self) -> bool {
        !self.progress.is_empty() && self.progress.last() < self.carrying.get(self.progress.len() - 1)
    }

    fn should_pad(&self, enter_level: usize) -> bool {
        self.is_not_full() && enter_level <= self.progress.len() - 1
    }

    fn pad(&self) -> i32 {
        self.carrying.get(self.progress.len() - 1).unwrap() - self.progress.last().unwrap()
    }

    fn should_carry(&self) -> bool {
        !self.progress.is_empty() && self.progress.last() == self.carrying.get(self.progress.len() - 1)
    }
}
