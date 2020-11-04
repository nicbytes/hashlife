use crate::automata::Automata;
use crate::quad4x4::Quad4x4;
use crate::quad::Quadrant;

use std::rc::Rc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum CellOf2x2 {
    Automata(Automata),
    Reference(Direction),
    None,
}

impl CellOf2x2 {
    fn is_none(&self) -> bool {
        match self {
            CellOf2x2::None => true,
            _ => false,
        }
    }

    fn is_automata(&self) -> bool {
        match self {
            CellOf2x2::Automata(_) => true,
            _ => false,
        }
    }

    fn is_dead(&self) -> bool {
        match self {
            CellOf2x2::Automata(a) => a.is_dead(),
            _ => false,
        }
    }

    fn is_alive(&self) -> bool {
        match self {
            CellOf2x2::Automata(a) => a.is_alive(),
            _ => false,
        }
    }

    fn is_reference(&self) -> bool {
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

struct Quad2x2 {
    quadrant: Quadrant,
    parent: Option<Rc<Quad4x4>>,
    nw: Rc<CellOf2x2>,
    ne: Rc<CellOf2x2>,
    sw: Rc<CellOf2x2>,
    se: Rc<CellOf2x2>,
    hash: u64,
}

impl Hash for Quad2x2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

fn calculate_hash(quadrant: &Quadrant, nw: &Rc<CellOf2x2>, ne: &Rc<CellOf2x2>, sw: &Rc<CellOf2x2>, se: &Rc<CellOf2x2>) -> u64 {
    let mut state = DefaultHasher::new();
    quadrant.hash(&mut state);
    nw.hash(&mut state);
    ne.hash(&mut state);
    sw.hash(&mut state);
    se.hash(&mut state);
    state.finish();
}

use cached::SizedCache;


impl Quad2x2 {
    cached_key! {
        JOIN: SizedCache<u64, Rc<Self>> = SizedCache::with_size(1024);
        Key = { calculate_hash(&quadrant, &nw, &ne, &sw, &se) };

        fn join(parent: Option<Rc<Quad4x4>>, quadrant: Quadrant, nw: Rc<CellOf2x2>, ne: Rc<CellOf2x2>, sw: Rc<CellOf2x2>, se: Rc<CellOf2x2>) -> Rc<Self> {
            // pre-compute hash
            let hash = calculate_hash(&quadrant, &nw, &ne, &sw, &se);

            Rc::new(Self {
                quadrant,
                parent,
                nw, ne, sw, se,
                hash,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_empty_top_2x2() {
        let e = Quad2x2::empty(None, Quadrant::TOP);
        assert_eq!(e.quadrant, Quadrant::TOP);
        assert!(e.parent.is_none());
    }

    #[test]
    fn make_empty_2x2_with_empty_children() {
        let e = Quad2x2::empty(None, Quadrant::TOP);
        assert!(e.nw.is_empty());
        assert!(e.ne.is_empty());
        assert!(e.sw.is_empty());
        assert!(e.se.is_empty());
    }

    #[test]
    fn make_dead_top() {
        let x = Quad2x2::dead(None, Quadrant::TOP);
        assert_eq!(x.quadrant, Quadrant::TOP);
        assert!(x.parent.is_none());
    }

    #[test]
    fn make_dead_2x2_with_empty_children() {
        let x = Quad2x2::dead(None, Quadrant::TOP);
        assert!(x.nw.is_dead());
        assert!(x.ne.is_dead());
        assert!(x.sw.is_dead());
        assert!(x.se.is_dead());
    }

    #[test]
    fn make_rand_top() {
        let x = Quad2x2::rand(None, Quadrant::TOP);
        assert_eq!(x.quadrant, Quadrant::TOP);
        assert!(x.parent.is_none());
    }

    #[test]
    fn make_rand_2x2_with_empty_children() {
        let x = Quad2x2::rand(None, Quadrant::TOP);
        assert!(x.nw.is_dead() || x.nw.is_alive());
        assert!(x.ne.is_dead() || x.ne.is_alive());
        assert!(x.sw.is_dead() || x.sw.is_alive());
        assert!(x.se.is_dead() || x.se.is_alive());
    }

    #[test]
    fn make_2x2_from() {
        let x1 = CellOf2x2::Automata(Automata::Dead);
        let x2 = CellOf2x2::Reference(Direction::E);
        let x3 = CellOf2x2::Automata(Automata::Alive);
        let x4 = CellOf2x2::Reference(Direction::E);
        let x = Quad2x2::join(None, Quadrant::TOP, x1, x2, x3, x4);
        assert!(x.nw.is_automata());
        assert!(x.nw.is_dead());
        assert!(x.sw.is_dead());
        assert!(x.ne.is_reference());
        assert!(x.se.is_reference());

    }
}

