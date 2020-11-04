
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Quadrant {
    NW,
    NE,
    SW,
    SE,
    TOP,
}


use crate::ReferenceCounter;


pub type RcQuad = ReferenceCounter<Quad>;

// enum QuadNode {
//     Calculatable(Rc<Quad>),
    
// }

pub struct Quad {}

// struct Quad {
//     position: Quadrant,
//     parent: Rc<Quad>,
//     nw: ,
//     ne: ,
//     sw: ,
//     se: ,
// }

