#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

/// Generic simulator for life of any rule set.
pub fn sim(center: Cell, neighbors: Vec<Cell>, birth: Vec<usize>, servival: Vec<usize>) -> Cell {
    let living_neighbors: usize = neighbors.iter().map(|cell| *cell as usize).sum();
    match center {
        Cell::Alive => {
            if servival.contains(&living_neighbors) {
                Cell::Alive
            } else {
                Cell::Dead
            }
        },
        Cell::Dead => {
            if birth.contains(&living_neighbors) {
                Cell::Alive
            } else {
                Cell::Dead
            }
        },
    }
}

pub fn simb3s23(center: Cell, n1: Cell, n2: Cell, n3: Cell, n4: Cell, n5: Cell, n6: Cell, n7: Cell, n8: Cell) -> Cell {
    let cells = vec![n1, n2, n3, n4, n5, n6, n7, n8];
    let servival = vec![2, 3];
    let birth = vec![3];
    sim(center, cells, birth, servival)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn alive_count() {
        let cell = Cell::Alive;
        assert_eq!(1, cell as usize);
    }

    #[test]
    fn alive_count_as_rc() {
        let cell = Rc::new(Cell::Alive);
        assert_eq!(1, *cell as usize);
    }

    #[test]
    /// Only similar cells have the same hash.
    fn cells_hashed() {
        let c1 = Cell::Alive;
        let c2 = Cell::Alive;
        let c3 = Cell::Dead;
        assert_eq!(calculate_hash(&c1), calculate_hash(&c2));
        assert_ne!(calculate_hash(&c1), calculate_hash(&c3));
    }

    #[test]
    /// Ensure the Rc returns the same hash as Cell.
    fn cell_rcs_hashed() {
        let c1 = Rc::new(Cell::Alive);
        let c2 = Cell::Alive;
        let c3 = Rc::new(Cell::Dead);
        assert_eq!(calculate_hash(&c1), calculate_hash(&c2));
        assert_ne!(calculate_hash(&c1), calculate_hash(&c3));
    }

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}