#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Automata {
    Dead = 0,
    Alive = 1,
}

impl Automata {
    pub fn is_dead(&self) -> bool {
        match self {
            Automata::Dead => true,
            _ => false,
        }
    }

    pub fn is_alive(&self) -> bool {
        match self {
            Automata::Alive => true,
            _ => false,
        }
    }
}

/// Generic simulator for life of any rule set.
pub fn sim(center: Automata, neighbors: Vec<Automata>, birth: Vec<usize>, servival: Vec<usize>) -> Automata {
    let living_neighbors: usize = neighbors.iter().map(|a| *a as usize).sum();
    match center {
        Automata::Alive => {
            if servival.contains(&living_neighbors) {
                Automata::Alive
            } else {
                Automata::Dead
            }
        },
        Automata::Dead => {
            if birth.contains(&living_neighbors) {
                Automata::Alive
            } else {
                Automata::Dead
            }
        },
    }
}

pub fn simb3s23(center: Automata, n1: Automata, n2: Automata, n3: Automata, n4: Automata, n5: Automata, n6: Automata, n7: Automata, n8: Automata) -> Automata {
    let automatas = vec![n1, n2, n3, n4, n5, n6, n7, n8];
    let servival = vec![2, 3];
    let birth = vec![3];
    sim(center, automatas, birth, servival)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn alive_count() {
        let a = Automata::Alive;
        assert_eq!(1, a as usize);
    }

    #[test]
    fn alive_count_as_rc() {
        let a = Rc::new(Automata::Alive);
        assert_eq!(1, *a as usize);
    }

    #[test]
    /// Only similar cells have the same hash.
    fn cells_hashed() {
        let c1 = Automata::Alive;
        let c2 = Automata::Alive;
        let c3 = Automata::Dead;
        assert_eq!(calculate_hash(&c1), calculate_hash(&c2));
        assert_ne!(calculate_hash(&c1), calculate_hash(&c3));
    }

    #[test]
    /// Ensure the Rc returns the same hash as Automata.
    fn cell_rcs_hashed() {
        let c1 = Rc::new(Automata::Alive);
        let c2 = Automata::Alive;
        let c3 = Rc::new(Automata::Dead);
        assert_eq!(calculate_hash(&c1), calculate_hash(&c2));
        assert_ne!(calculate_hash(&c1), calculate_hash(&c3));
    }

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}