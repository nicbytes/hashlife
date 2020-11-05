use crate::automata::Automata;
use crate::automata;
use crate::quad2x2::{Quad2x2, RcQuad2x2};
use crate::quad4x4::{Quad4x4, RcQuad4x4};
use crate::quad::{Quad, RcQuad, Children, Children8x8, ChildrenJxJ};
use crate::ReferenceCounter;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

type RcStep = ReferenceCounter<Step>;

pub enum Step {
    X4(RcQuad4x4),
    XJ(RcQuad),
}

/// Whenever creating or crafting any quad in any way, use the factory as a
/// single point of reference to ensure the result is made as fast as
/// possible.
pub struct Factory {
    cache2x2: HashMap<u64, RcQuad2x2>,
    cache4x4: HashMap<u64, RcQuad4x4>,
    cache4x4_sim: HashMap<RcQuad4x4, RcQuad2x2>,
    cache_x_: HashMap<u64, RcQuad>,
    cache_sim: HashMap<RcQuad, RcStep>,
}

impl Factory {
    /// Create a new Quad cache factory.
    pub fn new() -> Self {
        Self {
            cache2x2: HashMap::new(),
            cache4x4: HashMap::new(),
            cache4x4_sim: HashMap::new(),
            cache_x_: HashMap::new(),
            cache_sim: HashMap::new(),
        }
    }

    pub fn join2x2(&mut self, nw: Automata, ne: Automata, sw: Automata, se: Automata) -> RcQuad2x2 {
        // pre-compute hash
        let hash = calculate_hash_for_2x2(&nw, &ne, &sw, &se);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache2x2.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let population = ne as usize + nw as usize + se as usize + sw as usize;

        let quad = ReferenceCounter::new(Quad2x2 {
            nw, ne, sw, se,
            population,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache2x2.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }

    /// Create a quad with dead automata.
    pub fn dead2x2(&mut self) -> RcQuad2x2 {
        // make dead automatas
        let d = Automata::Dead;
        self.join2x2(d, d, d, d)
    }

    fn random_alive_dead_cell(&self) -> Automata {
        let mut automata = [0u8; 1];
        getrandom::getrandom(&mut automata).expect("failed to gen rand automata");
        if automata[0] % 2 == 0 {
            Automata::Dead
        } else {
            Automata::Alive
        }
    }

    pub fn rand2x2(&mut self) -> RcQuad2x2 {
        // collect random automata
        let nw = self.random_alive_dead_cell();
        let ne = self.random_alive_dead_cell();
        let sw = self.random_alive_dead_cell();
        let se = self.random_alive_dead_cell();
        self.join2x2(nw, ne, sw, se)
    }

    pub fn join4x4(&mut self, nw: RcQuad2x2, ne: RcQuad2x2, sw: RcQuad2x2, se: RcQuad2x2) -> RcQuad4x4 {
        // pre-compute hash
        let hash = calculate_hash_for_4x4(&nw, &ne, &sw, &se);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache4x4.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let population = ne.pop() + nw.pop() + se.pop() + sw.pop();

        let quad = ReferenceCounter::new(Quad4x4 {
            nw, ne, sw, se,
            population,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache4x4.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }

    /// Create a quad with dead automata.
    pub fn dead4x4(&mut self) -> RcQuad4x4 {
        // make dead automatas
        let d = self.dead2x2();
        self.join4x4(
            ReferenceCounter::clone(&d), 
        ReferenceCounter::clone(&d), 
        ReferenceCounter::clone(&d), d)
    }

    pub fn rand4x4(&mut self) -> RcQuad4x4 {
        // collect random automata
        let nw = self.rand2x2();
        let ne = self.rand2x2();
        let sw = self.rand2x2();
        let se = self.rand2x2();
        self.join4x4(nw, ne, sw, se)
    }

    /// Steps a 4x4 quad one generation into the future.
    pub fn step4x4(&mut self, quad: RcQuad4x4) -> RcQuad2x2 {
        if let Some(ref_to_quad) = self.cache4x4_sim.get(&quad) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let nw = automata::simb3s23(
            quad.nw.se,
            quad.nw.nw,
            quad.nw.ne,
            quad.ne.nw,
            quad.ne.sw,
            quad.se.nw,
            quad.sw.ne,
            quad.sw.nw,
            quad.nw.sw
        );
        let ne = automata::simb3s23(
            quad.ne.sw,
            quad.nw.ne,
            quad.ne.nw,
            quad.ne.ne,
            quad.ne.se,
            quad.se.ne,
            quad.se.nw,
            quad.sw.ne,
            quad.nw.se,
        );
        let sw = automata::simb3s23(
            quad.sw.ne,
            quad.nw.sw,
            quad.nw.se,
            quad.ne.sw,
            quad.se.nw,
            quad.se.sw,
            quad.sw.se,
            quad.sw.sw,
            quad.sw.nw,
        );
        let se = automata::simb3s23(
            quad.se.nw,
            quad.nw.se,
            quad.ne.sw,
            quad.ne.se,
            quad.se.ne,
            quad.se.se,
            quad.se.sw,
            quad.sw.se,
            quad.sw.ne,
        );

        let result = self.join2x2(nw, ne, sw, se);

        // Save the newly created quad in the cache.
        self.cache4x4_sim.insert(quad, ReferenceCounter::clone(&result));
        result

    }

    pub fn join_quad_from_4x4(&mut self, nw: RcQuad4x4, ne: RcQuad4x4, sw: RcQuad4x4, se: RcQuad4x4) -> RcQuad {
        // pre-compute hash
        let hash = calculate_hash_for_quad_of_4x4(&nw, &ne, &sw, &se);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache_x_.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let population = ne.pop() + nw.pop() + se.pop() + sw.pop();

        let quad = ReferenceCounter::new(Quad {
            children: Children::X8(Children8x8 {nw, ne, sw, se}),
            population,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache_x_.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }


    pub fn join_x_(&mut self, nw: RcQuad, ne: RcQuad, sw: RcQuad, se: RcQuad) -> RcQuad {
        // pre-compute hash
        let hash = calculate_hash_for_x_(&nw, &ne, &sw, &se);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache_x_.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let children = Children::XJ(ChildrenJxJ { nw, ne, sw, se });
        let population = ne.pop() + nw.pop() + se.pop() + sw.pop();

        let quad = ReferenceCounter::new(Quad {
            children,
            population,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache_x_.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }


    pub fn join_from_8x8(&mut self, nw: Children8x8, ne: Children8x8, sw: Children8x8, se: Children8x8) -> RcQuad {
        // reconstruct nodes
        let nw8x8 = self.join_quad_from_4x4(nw.nw, nw.ne, nw.sw, nw.se);
        let ne8x8 = self.join_quad_from_4x4(ne.nw, ne.ne, ne.sw, ne.se);
        let sw8x8 = self.join_quad_from_4x4(sw.nw, sw.ne, sw.sw, sw.se);
        let se8x8 = self.join_quad_from_4x4(se.nw, se.ne, se.sw, se.se);
        let nw = nw8x8;
        let ne = ne8x8;
        let sw = sw8x8;
        let se = se8x8;

        // pre-compute hash
        let hash = calculate_hash_for_x_(&nw, &ne, &sw, &se);

        // Check if already exists.
        if let Some(ref_to_quad) = self.cache_x_.get(&hash) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        let children = Children::XJ(ChildrenJxJ { nw, ne, sw, se });
        let population = ne.pop() + nw.pop() + se.pop() + sw.pop();

        let quad = ReferenceCounter::new(Quad {
            children,
            population,
            hash,
        });

        // Save the newly created quad in the cache.
        self.cache_x_.insert(hash, ReferenceCounter::clone(&quad));
        quad
    }

    pub fn step_children_8x8(&self, child: &Children8x8) -> RcQuad4x4 {
        
    }

    pub fn step_x_(&mut self, quad: RcQuad) -> RcStep {
        if let Some(ref_to_quad) = self.cache_sim.get(&quad) {
            return ReferenceCounter::clone(ref_to_quad);
        }

        match quad.children {
            // Steps a 8x8 quad one generation into the future.
            // Will need to access 2x2 components to produce resulting 4x4.
            Children::X8(quad_child) => {
                let nw4x4 = ReferenceCounter::clone(&quad_child.nw);
                let ne4x4 = ReferenceCounter::clone(&quad_child.ne);
                let sw4x4 = ReferenceCounter::clone(&quad_child.sw);
                let se4x4 = ReferenceCounter::clone(&quad_child.se);
                let n_4x4 = self.join4x4(
                    nw4x4.ne,
                    ne4x4.nw,
                    nw4x4.se,
                    ne4x4.sw
                );
                let e_4x4 = self.join4x4(
                    ne4x4.sw,
                    ne4x4.se,
                    se4x4.nw,
                    se4x4.ne
                );
                let s_4x4 = self.join4x4(
                    sw4x4.ne,
                    se4x4.nw,
                    sw4x4.se,
                    se4x4.sw
                );
                let w_4x4 = self.join4x4(
                    nw4x4.sw,
                    nw4x4.se,
                    sw4x4.nw,
                    sw4x4.ne
                );
                let c_4x4 = self.join4x4(
                    nw4x4.se,
                    ne4x4.sw,
                    sw4x4.ne,
                    se4x4.nw
                );
                let nw4x4_step = self.step4x4(nw4x4);
                let ne4x4_step = self.step4x4(ne4x4);
                let sw4x4_step = self.step4x4(sw4x4);
                let se4x4_step = self.step4x4(se4x4);
                let n_4x4_step = self.step4x4(n_4x4);
                let e_4x4_step = self.step4x4(e_4x4);
                let s_4x4_step = self.step4x4(s_4x4);
                let w_4x4_step = self.step4x4(w_4x4);
                let c_4x4_step = self.step4x4(c_4x4);
                let nw2x2 = self.join2x2(
                    nw4x4_step.se,
                    n_4x4_step.sw,
                    w_4x4_step.ne,
                    c_4x4_step.nw
                );
                let ne2x2 = self.join2x2(
                    n_4x4_step.se,
                    ne4x4_step.sw,
                    c_4x4_step.ne,
                    w_4x4_step.nw
                );
                let sw2x2 = self.join2x2(
                    w_4x4_step.se,
                    c_4x4_step.sw,
                    sw4x4_step.ne,
                    s_4x4_step.nw
                );
                let se2x2 = self.join2x2(
                    c_4x4_step.se,
                    w_4x4_step.sw,
                    s_4x4_step.ne,
                    se4x4_step.nw,
                );
                let step = self.join4x4(nw2x2, ne2x2, sw2x2, se2x2);
                let step = ReferenceCounter::new(Step::X4(step));
                self.cache_sim.insert(quad, ReferenceCounter::clone(&step));
                return step;
            }
            // Children of children (which needs access) may contain X8s?.
            Children::XJ(x) => {
                let nodes = (&x.nw.children, &x.ne.children, &x.sw.children, &x.se.children);
                match nodes {
                    // Steps a 16x16 quad one generation into the future.
                    // Will need to access 4x4 components to produce resulting 8x8.
                    (Children::X8(nw8x8), Children::X8(ne8x8), Children::X8(sw8x8), Children::X8(se8x8)) => {
                        let n_8x8 = self.join_quad_from_4x4(
                            nw8x8.ne,
                            ne8x8.nw,
                            nw8x8.se,
                            ne8x8.sw
                        );
                        let e_8x8 = self.join_quad_from_4x4(
                            ne8x8.sw,
                            ne8x8.se,
                            se8x8.nw,
                            se8x8.ne
                        );
                        let s_8x8 = self.join_quad_from_4x4(
                            sw8x8.ne,
                            se8x8.nw,
                            sw8x8.se,
                            se8x8.sw
                        );
                        let w_8x8 = self.join_quad_from_4x4(
                            nw8x8.sw,
                            nw8x8.se,
                            sw8x8.nw,
                            sw8x8.ne
                        );
                        let c_8x8 = self.join_quad_from_4x4(
                            nw8x8.se,
                            ne8x8.sw,
                            sw8x8.ne,
                            se8x8.nw
                        );
                        let nw8x8 = self.join_quad_from_4x4(nw8x8.nw, nw8x8.ne, nw8x8.sw, nw8x8.se);
                        let ne8x8 = self.join_quad_from_4x4(ne8x8.nw, ne8x8.ne, ne8x8.sw, ne8x8.se);
                        let sw8x8 = self.join_quad_from_4x4(sw8x8.nw, sw8x8.ne, sw8x8.sw, sw8x8.se);
                        let se8x8 = self.join_quad_from_4x4(se8x8.nw, se8x8.ne, se8x8.sw, se8x8.se);
                        let nw8x8_step = self.step_x_(nw8x8);
                        let ne8x8_step = self.step_x_(ne8x8);
                        let sw8x8_step = self.step_x_(sw8x8);
                        let se8x8_step = self.step_x_(se8x8);
                        let n_8x8_step = self.step_x_(n_8x8);
                        let e_8x8_step = self.step_x_(e_8x8);
                        let s_8x8_step = self.step_x_(s_8x8);
                        let w_8x8_step = self.step_x_(w_8x8);
                        let c_8x8_step = self.step_x_(c_8x8);

                        let nodes_stepped = (
                            nw8x8_step.as_ref(),
                            ne8x8_step.as_ref(),
                            sw8x8_step.as_ref(),
                            se8x8_step.as_ref(),
                            n_8x8_step.as_ref(),
                            e_8x8_step.as_ref(),
                            s_8x8_step.as_ref(),
                            w_8x8_step.as_ref(),
                            c_8x8_step.as_ref()
                        );

                        let 
                        (
                            nw8x8_step,
                            ne8x8_step,
                            sw8x8_step,
                            se8x8_step,
                            n_8x8_step,
                            e_8x8_step,
                            s_8x8_step,
                            w_8x8_step,
                            c_8x8_step
                        )
                        =
                        match nodes_stepped {
                            (
                                Step::X4(n1),
                                Step::X4(n2),
                                Step::X4(n3),
                                Step::X4(n4),
                                Step::X4(n5),
                                Step::X4(n6),
                                Step::X4(n7),
                                Step::X4(n8),
                                Step::X4(n9)
                            ) => (n1, n2, n3, n4, n5, n6, n7, n8, n9),
                            _ => panic!("8x8 must only contain 4x4s")
                        };

                        let nw4x4 = self.join4x4(
                            nw8x8_step.se,
                            n_8x8_step.sw,
                            w_8x8_step.ne,
                            c_8x8_step.nw
                        );
                        let ne4x4 = self.join4x4(
                            n_8x8_step.se,
                            ne8x8_step.sw,
                            c_8x8_step.ne,
                            w_8x8_step.nw
                        );
                        let sw4x4 = self.join4x4(
                            w_8x8_step.se,
                            c_8x8_step.sw,
                            sw8x8_step.ne,
                            s_8x8_step.nw
                        );
                        let se4x4 = self.join4x4(
                            c_8x8_step.se,
                            w_8x8_step.sw,
                            s_8x8_step.ne,
                            se8x8_step.nw,
                        );
                        let step = self.join_quad_from_4x4(nw4x4, ne4x4, sw4x4, se4x4);
                        let step =  ReferenceCounter::new(Step::XJ(step));
                        self.cache_sim.insert(quad, ReferenceCounter::clone(&step));
                        return step;
                    },
                    // Steps a (32+ x 32+) (IxI) quad one generation into the future.
                    // Will need to access (8+ x 8+) (LxL) components to produce resulting (16+ x 16+) (JxJ).
                    (Children::XJ(nwJxJ), Children::XJ(neJxJ), Children::XJ(swJxJ), Children::XJ(seJxJ)) => {
                        let n_JxJ = self.join_x_(
                            nwJxJ.ne,
                            neJxJ.nw,
                            nwJxJ.se,
                            neJxJ.sw
                        );
                        let e_JxJ = self.join_x_(
                            neJxJ.sw,
                            neJxJ.se,
                            seJxJ.nw,
                            seJxJ.ne
                        );
                        let s_JxJ = self.join_x_(
                            swJxJ.ne,
                            seJxJ.nw,
                            swJxJ.se,
                            seJxJ.sw
                        );
                        let w_JxJ = self.join_x_(
                            nwJxJ.sw,
                            nwJxJ.se,
                            swJxJ.nw,
                            swJxJ.ne
                        );
                        let c_JxJ = self.join_x_(
                            nwJxJ.se,
                            neJxJ.sw,
                            swJxJ.ne,
                            seJxJ.nw
                        );
                        // TODO: replace lines below with `let nwJxJ = x.nw;`
                        let nwJxJ = self.join_x_(nwJxJ.nw, nwJxJ.ne, nwJxJ.sw, nwJxJ.se);
                        let neJxJ = self.join_x_(neJxJ.nw, neJxJ.ne, neJxJ.sw, neJxJ.se);
                        let swJxJ = self.join_x_(swJxJ.nw, swJxJ.ne, swJxJ.sw, swJxJ.se);
                        let seJxJ = self.join_x_(seJxJ.nw, seJxJ.ne, seJxJ.sw, seJxJ.se);
                        let nwLxL_step = self.step_x_(nwJxJ);
                        let neLxL_step = self.step_x_(neJxJ);
                        let swLxL_step = self.step_x_(swJxJ);
                        let seLxL_step = self.step_x_(seJxJ);
                        let n_LxL_step = self.step_x_(n_JxJ);
                        let e_LxL_step = self.step_x_(e_JxJ);
                        let s_LxL_step = self.step_x_(s_JxJ);
                        let w_LxL_step = self.step_x_(w_JxJ);
                        let c_LxL_step = self.step_x_(c_JxJ);
                        let nodes_stepped = (
                            nwLxL_step.as_ref(),
                            neLxL_step.as_ref(),
                            swLxL_step.as_ref(),
                            seLxL_step.as_ref(),
                            n_LxL_step.as_ref(),
                            e_LxL_step.as_ref(),
                            s_LxL_step.as_ref(),
                            w_LxL_step.as_ref(),
                            c_LxL_step.as_ref()
                        );
                        let 
                        (
                            nwLxL_step,
                            neLxL_step,
                            swLxL_step,
                            seLxL_step,
                            n_LxL_step,
                            e_LxL_step,
                            s_LxL_step,
                            w_LxL_step,
                            c_LxL_step
                        )
                        =
                        match nodes_stepped {
                            (
                                Step::XJ(n1),
                                Step::XJ(n2),
                                Step::XJ(n3),
                                Step::XJ(n4),
                                Step::XJ(n5),
                                Step::XJ(n6),
                                Step::XJ(n7),
                                Step::XJ(n8),
                                Step::XJ(n9)
                            ) => (n1, n2, n3, n4, n5, n6, n7, n8, n9),
                            _ => panic!("8x8 must only contain 4x4s")
                        };

                        let nwLxL = self.join_x_(
                            nwLxL_step.children.se,
                            n_LxL_step.sw,
                            w_LxL_step.ne,
                            c_LxL_step.nw
                        );
                        let neLxL = self.join_x_(
                            n_LxL_step.se,
                            neLxL_step.sw,
                            c_LxL_step.ne,
                            w_LxL_step.nw
                        );
                        let swLxL = self.join_x_(
                            w_LxL_step.se,
                            c_LxL_step.sw,
                            swLxL_step.ne,
                            s_LxL_step.nw
                        );
                        let seLxL = self.join_x_(
                            c_LxL_step.se,
                            w_LxL_step.sw,
                            s_LxL_step.ne,
                            seLxL_step.nw,
                        );

                        ()
                    },
                    _ => panic!("children of a quad are not of the same level"),
                }
            }
        }
    }
}

fn calculate_hash_for_2x2(nw: &Automata, ne: &Automata, sw: &Automata, se: &Automata) -> u64 {
    let mut state = DefaultHasher::new();
    nw.hash(&mut state);
    ne.hash(&mut state);
    sw.hash(&mut state);
    se.hash(&mut state);
    state.finish()
}

fn calculate_hash_for_4x4(nw: &RcQuad2x2, ne: &RcQuad2x2, sw: &RcQuad2x2, se: &RcQuad2x2) -> u64 {
    let mut state = DefaultHasher::new();
    nw.hash(&mut state);
    ne.hash(&mut state);
    sw.hash(&mut state);
    se.hash(&mut state);
    state.finish()
}

fn calculate_hash_for_quad_of_4x4(nw: &RcQuad4x4, ne: &RcQuad4x4, sw: &RcQuad4x4, se: &RcQuad4x4) -> u64 {
    let mut state = DefaultHasher::new();
    nw.hash(&mut state);
    ne.hash(&mut state);
    sw.hash(&mut state);
    se.hash(&mut state);
    state.finish()
}

fn calculate_hash_for_x_(nw: &RcQuad, ne: &RcQuad, sw: &RcQuad, se: &RcQuad) -> u64 {
    let mut state = DefaultHasher::new();
    nw.hash(&mut state);
    ne.hash(&mut state);
    sw.hash(&mut state);
    se.hash(&mut state);
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn make_4x4_step() {
        let dead = Automata::Dead;
        let alive = Automata::Alive;
        let mut factory = Factory::new();
        let x1 = factory.join2x2(alive, dead, dead, dead);
        let x2 = factory.join2x2(alive, dead, dead, dead);
        let x3 = factory.join2x2(dead, alive, dead, dead);
        let x4 = factory.join2x2(dead, dead, alive, alive);
        let expected = factory.join2x2(alive, dead, dead, alive);
        let x = factory.join4x4(x1, x2, x3, x4);
        let result = factory.step4x4(x);
        assert_eq!(expected, result);
    }
}