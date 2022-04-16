use super::ir_type::to_array_ty;
use koopa::ir;
use std::{iter::zip, num::TryFromIntError, ops::Deref};

#[derive(Debug, Clone)]
pub struct Shape(Vec<i32>, Vec<i32>, i32);

impl Shape {
    pub fn new(v: Vec<i32>) -> Shape {
        let mut p = vec![1];
        let mut t = 1;
        v.iter().rev().for_each(|d| {
            t *= d;
            p.push(t);
        });
        let size = p.pop().unwrap();
        p.reverse();
        Shape(v, p, size)
    }

    pub fn index(&self, v: &Vec<i32>) -> i32 {
        assert_eq!(v.len(), self.0.len());
        zip(v, &self.1).fold(0, |acc, (i, s)| acc + i * s)
    }

    pub fn total(&self) -> i32 {
        self.2
    }

    pub fn ty(&self, dim_r: usize) -> ir::Type {
        let x = &mut self.0[dim_r..]
            .iter()
            .rev()
            .map(|i| (*i).try_into().unwrap());
        to_array_ty(x)
    }

    pub fn tys(&self) -> Vec<ir::Type> {
        let mut v = vec![ty!(i32)];
        let mut p = v.last().unwrap().clone();
        for d in self.iter().rev() {
            p = ir::Type::get_array(p, (*d) as usize);
            v.push(p);
            p = v.last().unwrap().clone();
        }
        v.reverse();
        v
    }
}

impl From<Vec<i32>> for Shape {
    fn from(v: Vec<i32>) -> Self {
        Shape::new(v)
    }
}

impl From<&Vec<i32>> for Shape {
    fn from(v: &Vec<i32>) -> Self {
        v.clone().into()
    }
}

pub struct TypeKindIter<'a>(&'a ir::TypeKind);

impl<'a> Iterator for TypeKindIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        use ir::TypeKind::*;
        if let Array(t, i) = self.0 {
            self.0 = t.kind();
            Some(*i)
        } else {
            None
        }
    }
}

impl TryFrom<&ir::Type> for Shape {
    type Error = TryFromIntError;

    fn try_from(ty: &ir::Type) -> Result<Self, Self::Error> {
        Ok(ty.kind().try_into()?)
    }
}

impl TryFrom<&ir::TypeKind> for Shape {
    type Error = TryFromIntError;

    fn try_from(kind: &ir::TypeKind) -> Result<Self, Self::Error> {
        Ok(Shape::new(
            TypeKindIter(kind)
                .into_iter()
                .map(|u| u.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl From<Shape> for ir::Type {
    fn from(s: Shape) -> Self {
        to_array_ty(s.0.iter().rev().map(|i| (*i).try_into().unwrap()))
    }
}

impl From<Shape> for Vec<i32> {
    fn from(s: Shape) -> Self {
        s.0
    }
}

impl Deref for Shape {
    type Target = Vec<i32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/*
pub fn fold<'a, 'b: 'a, C>(shape: &'a Shape, flat: Vec<Option<i32>>, ctx: &'b mut C) -> ir::Value
where C: WrapProgram + AddPlainValue {
    let flat: Vec<_> = flat.into_iter().map(|o| o.map(|i: i32| ctx.add_plain_value_integer(i))).collect();
    let folded = shape
    .iter()
    .map(|d| (*d).try_into().unwrap())
    .enumerate()
    .rev()
    .fold(flat, |acc, (dim_r, dim_size)| {
        let ty = shape.ty(dim_r);
        acc
        .as_slice()
        .chunks(dim_size)
        .map(|c| {
            if !c.iter().all(Option::is_none) {
                Some(
                    ctx.add_plain_value_aggregate(
                        c.iter().map(|o| {
                            o.unwrap_or_else(|| ctx.add_plain_value_zeroinit(ty.clone()))
                        }).collect()
                    )
                )
            } else { None }
        })
        .collect()
    });
    dbg!(&folded);
    todo!()
}*/
