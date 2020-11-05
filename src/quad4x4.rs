use crate::quad2x2::RcQuad2x2;
use crate::ReferenceCounter;

use std::hash::{Hash, Hasher};


pub type RcQuad4x4 = ReferenceCounter<Quad4x4>;

#[derive(Debug)]
pub struct Quad4x4 {
    pub nw: RcQuad2x2,
    pub ne: RcQuad2x2,
    pub sw: RcQuad2x2,
    pub se: RcQuad2x2,
    pub population: usize,
    pub hash: u64,
}

impl Quad4x4 {
    pub fn pop(&self) -> usize {
        self.population
    }
}

impl Hash for Quad4x4 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl PartialEq for Quad4x4 {
    fn eq(&self, other: &Quad4x4) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Quad4x4 {}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::Factory;
    use crate::automata::Automata;

    #[test]
    fn make_dead4x4() {
        let _e = Factory::new().dead4x4();
    }

    #[test]
    fn make_4x4_join() {
        let dead = Automata::Dead;
        let alive = Automata::Alive;
        let mut factory = Factory::new();
        let x1 = factory.join2x2(alive, dead, dead, dead);
        let x2 = factory.join2x2(alive, alive, dead, dead);
        let x3 = factory.join2x2(alive, alive, alive, dead);
        let x4 = factory.join2x2(alive, alive, alive, alive);
        let x = factory.join4x4(x1, x2, x3, x4);
        assert_eq!(x.nw.pop(), 1);
        assert_eq!(x.ne.pop(), 2);
        assert_eq!(x.sw.pop(), 3);
        assert_eq!(x.se.pop(), 4);
    }

    #[test]
    fn make_4x4_pop() {
        let dead = Automata::Dead;
        let alive = Automata::Alive;
        let mut factory = Factory::new();
        let x1 = factory.join2x2(alive, dead, dead, dead);
        let x2 = factory.join2x2(alive, alive, dead, dead);
        let x3 = factory.join2x2(alive, alive, alive, dead);
        let x4 = factory.join2x2(alive, alive, alive, alive);
        let x = factory.join4x4(x1, x2, x3, x4);
        assert_eq!(x.pop(), 1 + 2 + 3 + 4);
    }
}
