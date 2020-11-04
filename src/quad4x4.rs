use crate::quad2x2::RcQuad2x2;
use crate::quad::{Quadrant, RcQuad};
use crate::ReferenceCounter;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};


pub type RcQuad4x4 = ReferenceCounter<Quad4x4>;

pub struct Quad4x4 {
    quadrant: Quadrant,
    parent: Option<RcQuad>,
    nw: RcQuad2x2,
    ne: RcQuad2x2,
    sw: RcQuad2x2,
    se: RcQuad2x2,
    hash: u64,
}
