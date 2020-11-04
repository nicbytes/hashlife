use crate::automata::Automata;
use crate::quad2x2::{Quad2x2, RcQuad2x2, CellOf2x2};
use crate::quad4x4::RcQuad4x4;
use crate::quad::{RcQuad, Quadrant};
use crate::ReferenceCounter;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Whenever creating or crafting any quad in any way, use the factory as a
/// single point of reference to ensure the result is made as fast as
/// possible.
pub struct Factory {
    cache2x2: HashMap<u64, RcQuad2x2>,
    cache4x4: HashMap<u64, RcQuad4x4>,
    cache_x_: HashMap<u64, RcQuad>,
}

impl Factory {
    /// Create a new Quad cache factory.
    pub fn new() -> Self {
        Self {
            cache2x2: HashMap::new(),
            cache4x4: HashMap::new(),
            cache_x_: HashMap::new(),
        }
    }

    pub fn join2x2(&mut self, parent: Option<RcQuad4x4>, quadrant: Quadrant, nw: CellOf2x2, ne: CellOf2x2, sw: CellOf2x2, se: CellOf2x2) -> RcQuad2x2 {
        // pre-compute hash
        let hash = calculate_hash_for_2x2(&quadrant, &nw, &ne, &sw, &se);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache2x2.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let quad = ReferenceCounter::new(Quad2x2 {
            quadrant,
            parent,
            nw, ne, sw, se,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache2x2.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }

    /// Create a quad with empty children.
    pub fn empty2x2(&mut self, parent: Option<RcQuad4x4>, quadrant: Quadrant) -> RcQuad2x2 {
        // make empty automatas
        let e = CellOf2x2::None;
        self.join2x2(parent, quadrant, e, e, e, e)
    }

    /// Create a quad with dead automata.
    pub fn dead2x2(&mut self, parent: Option<RcQuad4x4>, quadrant: Quadrant) -> RcQuad2x2 {
        // make dead automatas
        let d = CellOf2x2::Automata(Automata::Dead);
        self.join2x2(parent, quadrant, d, d, d, d)
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

    pub fn rand2x2(&mut self, parent: Option<RcQuad4x4>, quadrant: Quadrant) -> RcQuad2x2 {
        // collect random automata
        let nw = self.random_alive_dead_cell();
        let ne = self.random_alive_dead_cell();
        let sw = self.random_alive_dead_cell();
        let se = self.random_alive_dead_cell();
        self.join2x2(parent, quadrant, nw, ne, sw, se)
    }

}

fn calculate_hash_for_2x2(quadrant: &Quadrant, nw: &CellOf2x2, ne: &CellOf2x2, sw: &CellOf2x2, se: &CellOf2x2) -> u64 {
    let mut state = DefaultHasher::new();
    quadrant.hash(&mut state);
    nw.hash(&mut state);
    ne.hash(&mut state);
    sw.hash(&mut state);
    se.hash(&mut state);
    state.finish()
}