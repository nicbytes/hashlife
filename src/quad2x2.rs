use crate::automata::Automata;
use crate::quad4x4::RcQuad4x4;
use crate::quad::Quadrant;
use crate::ReferenceCounter;
use crate::factory::Factory;

use std::hash::{Hash, Hasher};

pub type RcQuad2x2 = ReferenceCounter<Quad2x2>;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum CellOf2x2 {
    Automata(Automata),
    Reference(Direction),
    None,
}

impl CellOf2x2 {
    pub fn is_none(&self) -> bool {
        match self {
            CellOf2x2::None => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.is_none()
    }

    pub fn is_automata(&self) -> bool {
        match self {
            CellOf2x2::Automata(_) => true,
            _ => false,
        }
    }

    pub fn is_dead(&self) -> bool {
        match self {
            CellOf2x2::Automata(a) => a.is_dead(),
            _ => false,
        }
    }

    pub fn is_alive(&self) -> bool {
        match self {
            CellOf2x2::Automata(a) => a.is_alive(),
            _ => false,
        }
    }

    pub fn is_reference(&self) -> bool {
        match self {
            CellOf2x2::Reference(_) => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Direction {
    NW, N, NE, E, SE, S, SW, W,
}

pub struct Quad2x2 {
    pub quadrant: Quadrant,
    pub parent: Option<RcQuad4x4>,
    pub nw: CellOf2x2,
    pub ne: CellOf2x2,
    pub sw: CellOf2x2,
    pub se: CellOf2x2,
    pub hash: u64,
}

impl Hash for Quad2x2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_factory() -> Factory {
        Factory::new()
    }

    #[test]
    fn make_empty_top_2x2() {
        let e = make_factory().empty2x2(None, Quadrant::TOP);
        assert_eq!(e.quadrant, Quadrant::TOP);
        assert!(e.parent.is_none());
    }

    #[test]
    fn make_empty_2x2_with_empty_children() {
        let e = make_factory().empty2x2(None, Quadrant::TOP);
        assert!(e.nw.is_empty());
        assert!(e.ne.is_empty());
        assert!(e.sw.is_empty());
        assert!(e.se.is_empty());
    }

    #[test]
    fn make_dead_top() {
        let x = make_factory().dead2x2(None, Quadrant::TOP);
        assert_eq!(x.quadrant, Quadrant::TOP);
        assert!(x.parent.is_none());
    }

    #[test]
    fn make_dead_2x2_with_empty_children() {
        let x = make_factory().dead2x2(None, Quadrant::TOP);
        assert!(x.nw.is_dead());
        assert!(x.ne.is_dead());
        assert!(x.sw.is_dead());
        assert!(x.se.is_dead());
    }

    #[test]
    fn make_rand_top() {
        let x = make_factory().rand2x2(None, Quadrant::TOP);
        assert_eq!(x.quadrant, Quadrant::TOP);
        assert!(x.parent.is_none());
    }

    #[test]
    fn make_rand_2x2_with_empty_children() {
        let x = make_factory().rand2x2(None, Quadrant::TOP);
        assert!(x.nw.is_dead() || x.nw.is_alive());
        assert!(x.ne.is_dead() || x.ne.is_alive());
        assert!(x.sw.is_dead() || x.sw.is_alive());
        assert!(x.se.is_dead() || x.se.is_alive());
    }

    #[test]
    fn make_2x2_join() {
        let x1 = CellOf2x2::Automata(Automata::Dead);
        let x2 = CellOf2x2::Reference(Direction::E);
        let x3 = CellOf2x2::Automata(Automata::Alive);
        let x4 = CellOf2x2::Reference(Direction::E);
        let x = make_factory().join2x2(None, Quadrant::TOP, x1, x2, x3, x4);
        assert!(x.nw.is_automata());
        assert!(x.nw.is_dead());
        assert!(x.sw.is_alive());
        assert!(x.ne.is_reference());
        assert!(x.se.is_reference());

    }
}

