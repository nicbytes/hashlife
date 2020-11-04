use crate::automata::Automata;
use crate::quad4x4::Quad4x4;
use crate::quad::Quadrant;

use std::rc::Rc;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

type ReferenceCounter<T> = Rc<T>;
type RcQuad2x2 = ReferenceCounter<Quad2x2>;
type RcQuad4x4 = ReferenceCounter<Quad4x4>;

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

struct Quad2x2 {
    quadrant: Quadrant,
    parent: Option<RcQuad4x4>,
    nw: CellOf2x2,
    ne: CellOf2x2,
    sw: CellOf2x2,
    se: CellOf2x2,
    hash: u64,
}

impl Hash for Quad2x2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

fn calculate_hash(quadrant: &Quadrant, nw: &CellOf2x2, ne: &CellOf2x2, sw: &CellOf2x2, se: &CellOf2x2) -> u64 {
    let mut state = DefaultHasher::new();
    quadrant.hash(&mut state);
    nw.hash(&mut state);
    ne.hash(&mut state);
    sw.hash(&mut state);
    se.hash(&mut state);
    state.finish()
}

/// Whenever creating or crafting a Quad2x2 in any way, use the factory as a
/// single point of reference to ensure the result is made as fast as
/// possible.
struct Factory2x2 {
    cache: HashMap<u64, RcQuad2x2>,
}

impl Factory2x2 {
    /// Create a new Quad 2x2 cache factory.
    fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    pub fn join(&mut self, parent: Option<RcQuad4x4>, quadrant: Quadrant, nw: CellOf2x2, ne: CellOf2x2, sw: CellOf2x2, se: CellOf2x2) -> RcQuad2x2 {
        // pre-compute hash
        let hash = calculate_hash(&quadrant, &nw, &ne, &sw, &se);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let quad = ReferenceCounter::new(Quad2x2 {
            quadrant,
            parent,
            nw, ne, sw, se,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }

    /// Create a quad with empty children.
    pub fn empty(&mut self, parent: Option<RcQuad4x4>, quadrant: Quadrant) -> RcQuad2x2 {
        // make empty automatas
        let e = CellOf2x2::None;
        // pre-compute hash
        let hash = calculate_hash(&quadrant, &e, &e, &e, &e);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let quad = ReferenceCounter::new(Quad2x2 {
            quadrant,
            parent,
            nw: e, ne: e, sw: e, se: e,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }

    /// Create a quad with dead automata.
    pub fn dead(&mut self, parent: Option<RcQuad4x4>, quadrant: Quadrant) -> RcQuad2x2 {
        // make dead automatas
        let d = CellOf2x2::Automata(Automata::Dead);
        // pre-compute hash
        let hash = calculate_hash(&quadrant, &d, &d, &d, &d);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let quad = ReferenceCounter::new(Quad2x2 {
            quadrant,
            parent,
            nw: d, ne: d, sw: d, se: d,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }

    fn random_alive_dead_cell(&self) -> CellOf2x2 {
        let mut automata = [0u8; 1];
        getrandom::getrandom(&mut automata).expect("failed to gen rand automata");
        if automata[0] % 2 == 0 {
            CellOf2x2::Automata(Automata::Dead)
        } else {
            CellOf2x2::Automata(Automata::Alive)
        }
    }

    pub fn rand(&mut self, parent: Option<RcQuad4x4>, quadrant: Quadrant) -> RcQuad2x2 {
        // collect random automata
        let nw = self.random_alive_dead_cell();
        let ne = self.random_alive_dead_cell();
        let sw = self.random_alive_dead_cell();
        let se = self.random_alive_dead_cell();

        // pre-compute hash
        let hash = calculate_hash(&quadrant, &nw, &ne, &sw, &se);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let quad = ReferenceCounter::new(Quad2x2 {
            quadrant,
            parent,
            nw, ne, sw, se,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_factory() -> Factory2x2 {
        Factory2x2 { cache: HashMap::new() }
    }

    #[test]
    fn make_empty_top_2x2() {
        let e = make_factory().empty(None, Quadrant::TOP);
        assert_eq!(e.quadrant, Quadrant::TOP);
        assert!(e.parent.is_none());
    }

    #[test]
    fn make_empty_2x2_with_empty_children() {
        let e = make_factory().empty(None, Quadrant::TOP);
        assert!(e.nw.is_empty());
        assert!(e.ne.is_empty());
        assert!(e.sw.is_empty());
        assert!(e.se.is_empty());
    }

    #[test]
    fn make_dead_top() {
        let x = make_factory().dead(None, Quadrant::TOP);
        assert_eq!(x.quadrant, Quadrant::TOP);
        assert!(x.parent.is_none());
    }

    #[test]
    fn make_dead_2x2_with_empty_children() {
        let x = make_factory().dead(None, Quadrant::TOP);
        assert!(x.nw.is_dead());
        assert!(x.ne.is_dead());
        assert!(x.sw.is_dead());
        assert!(x.se.is_dead());
    }

    #[test]
    fn make_rand_top() {
        let x = make_factory().rand(None, Quadrant::TOP);
        assert_eq!(x.quadrant, Quadrant::TOP);
        assert!(x.parent.is_none());
    }

    #[test]
    fn make_rand_2x2_with_empty_children() {
        let x = make_factory().rand(None, Quadrant::TOP);
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
        let x = make_factory().join(None, Quadrant::TOP, x1, x2, x3, x4);
        assert!(x.nw.is_automata());
        assert!(x.nw.is_dead());
        assert!(x.sw.is_alive());
        assert!(x.ne.is_reference());
        assert!(x.se.is_reference());

    }
}

