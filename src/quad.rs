use crate::quad2x2::RcQuad2x2;
use crate::quad4x4::RcQuad4x4;
use crate::ReferenceCounter;

use std::hash::{Hash, Hasher};

pub type RcQuad = ReferenceCounter<Quad>;
pub type RcLevel = ReferenceCounter<Children>;

#[derive(Debug)]
pub struct Children8x8 {
    pub nw: RcQuad4x4,
    pub ne: RcQuad4x4,
    pub sw: RcQuad4x4,
    pub se: RcQuad4x4,
}

#[derive(Debug)]
pub struct ChildrenJxJ {
    pub nw: RcQuad,
    pub ne: RcQuad,
    pub sw: RcQuad,
    pub se: RcQuad,
}

#[derive(Debug)]
pub enum Children {
    X8(Children8x8),
    XJ(ChildrenJxJ),
}

impl Hash for Children {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Children::X8(x) => state.write_u64(x.hash),
            Children::XJ(x) => state.write_u64(x.hash),
        }
    }
}

impl PartialEq for Children {
    fn eq(&self, other: &Children) -> bool {
        match self {
            Children::X8(s) => match other {
                Children::X8(o) => s.hash == o.hash,
                Children::XJ(_) => false,
            },
            Children::XJ(s) => match other {
                Children::X8(_) => false,
                Children::XJ(o) => s.hash == o.hash,
            },
        }
    }
}

impl Eq for Children {}


#[derive(Debug)]
pub struct Quad {
    pub children: Children,
    pub population: usize,
    pub hash: u64,
}

impl Quad {
    pub fn pop(&self) -> usize {
        self.population
    }
}

impl Hash for Quad {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl PartialEq for Quad {
    fn eq(&self, other: &Quad) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Quad {}

