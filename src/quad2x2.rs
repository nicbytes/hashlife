use crate::automata::Automata;
use crate::ReferenceCounter;

use std::hash::{Hash, Hasher};

pub type RcQuad2x2 = ReferenceCounter<Quad2x2>;

#[derive(Debug)]
pub struct Quad2x2 {
    pub nw: Automata,
    pub ne: Automata,
    pub sw: Automata,
    pub se: Automata,
    pub population: usize,
    pub hash: u64,
}

impl Quad2x2 {
    pub fn pop(&self) -> usize {
        self.population
    }
}

impl Hash for Quad2x2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl PartialEq for Quad2x2 {
    fn eq(&self, other: &Quad2x2) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Quad2x2 {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::Factory;

    #[test]
    fn make_dead_2x2() {
        let x = Factory::new().dead2x2();
        assert!(x.nw.is_dead());
        assert!(x.ne.is_dead());
        assert!(x.sw.is_dead());
        assert!(x.se.is_dead());
    }

    #[test]
    fn make_rand_2x2() {
        let x = Factory::new().rand2x2();
        assert!(x.nw.is_dead() || x.nw.is_alive());
        assert!(x.ne.is_dead() || x.ne.is_alive());
        assert!(x.sw.is_dead() || x.sw.is_alive());
        assert!(x.se.is_dead() || x.se.is_alive());
    }

    #[test]
    fn make_2x2_join() {
        let x1 = Automata::Dead;
        let x2 = Automata::Alive;
        let x3 = Automata::Dead;
        let x4 = Automata::Alive;
        let x = Factory::new().join2x2(x1, x2, x3, x4);
        assert!(x.nw.is_dead());
        assert!(x.ne.is_alive());
        assert!(x.sw.is_dead());
        assert!(x.se.is_alive());

    }

    #[test]
    fn make_2x2_population1() {
        let x1 = Automata::Dead;
        let x2 = Automata::Dead;
        let x3 = Automata::Dead;
        let x4 = Automata::Alive;
        let x = Factory::new().join2x2(x1, x2, x3, x4);
        assert_eq!(x.pop(), 1);
    }

    #[test]
    fn make_2x2_population3() {
        let x1 = Automata::Dead;
        let x2 = Automata::Alive;
        let x3 = Automata::Alive;
        let x4 = Automata::Alive;
        let x = Factory::new().join2x2(x1, x2, x3, x4);
        assert_eq!(x.pop(), 3);
    }
}

