use std::collections::HashMap;
use std::rc::Rc;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use getrandom::getrandom;
use crate::{GameOfLife, EdgeRule};

// trait QuatTreeNode {
//     fn level(&self) -> usize;
// }

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}
// impl Hash for Cell {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         match self {
//             Cell::Dead => state.write_i8(2),
//             Cell::Alive => state.write_i8(3),
//         }
//     }
// }

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum QuadNode {
    Cell(Cell),
    Tree(Tree),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tree {
    ne: Rc<QuadNode>,
    nw: Rc<QuadNode>,
    se: Rc<QuadNode>,
    sw: Rc<QuadNode>,
    level: usize,
    population: usize,
}

impl Hash for Tree {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ne.hash(state);
        self.nw.hash(state);
        self.se.hash(state);
        self.sw.hash(state);
        self.level.hash(state);
    }
}

impl Tree {

    fn ne(&self) -> Rc<QuadNode> {
        Rc::clone(&self.ne)
    }

    fn nw(&self) -> Rc<QuadNode> {
        Rc::clone(&self.nw)
    }

    fn se(&self) -> Rc<QuadNode> {
        Rc::clone(&self.se)
    }

    fn sw(&self) -> Rc<QuadNode> {
        Rc::clone(&self.sw)
    }
}

impl QuadNode {
    // fn set_cell(&self, x: isize, y: isize, cell: Cell) -> Self {
    //     match self {
    //         QuadNode::Cell(_) => QuadNode::Cell(cell),
    //         QuadNode::Tree(tree) => {
    //             let current_level = tree.level;
    //             let half: isize = 1 << (current_level - 1);
    //             if x < 0 { // west
    //                 if y < 0 { // nw
    //                     let (x, y) = (x + half, y + half);
    //                     let nw = tree.c_nw().set_cell(x, y, cell);
    //                     Tree::from(tree.c_ne(), nw, tree.c_se(), tree.c_sw())
    //                 } else { // sw
    //                     let (x, y) = (x + half, y - half);
    //                     let sw = tree.c_sw().set_cell(x, y, cell);
    //                     Tree::from(tree.c_ne(), tree.c_nw(), tree.c_se(), sw)
    //                 } // sw
    //             } else { // east
    //                 if y < 0 { // ne
    //                     let (x, y) = (x - half, y + half);
    //                     let ne = tree.c_ne().set_cell(x, y, cell);
    //                     Tree::from(ne, tree.c_nw(), tree.c_se(), tree.c_sw())
    //                 } else {
    //                     let (x, y) = (x - half, y - half);
    //                     let se = tree.c_se().set_cell(x, y, cell);
    //                     Tree::from(tree.c_ne(), tree.c_nw(), se, tree.c_se())
    //                 } // se
    //             }
    //         }
    //     }
    // }

    // pub fn get_cell(&self, x: isize, y: isize) -> Cell {
    //     match self {
    //         QuadNode::Cell(cell) => *cell,
    //         QuadNode::Tree(tree) => {
    //             let current_level = tree.level;
    //             let half: isize = 1 << (current_level - 1);
    //             if x < 0 { // west
    //                 if y < 0 { // nw
    //                     let (x, y) = (x + half, y + half);
    //                     tree.nw.get_cell(x, y)
    //                 } else { // sw
    //                     let (x, y) = (x + half, y - half);
    //                     tree.sw.get_cell(x, y)
    //                 } // sw
    //             } else { // east
    //                 if y < 0 { // ne
    //                     let (x, y) = (x - half, y + half);
    //                     tree.ne.get_cell(x, y)
    //                 } else { // se
    //                     let (x, y) = (x - half, y - half);
    //                     tree.se.get_cell(x, y)
    //                 }
    //             }
    //         }
    //     }
    // }

    fn next_generation(&self) {
        
    }

    fn force_cell(&self) -> Cell {
        match self {
            QuadNode::Cell(cell) => *cell,
            _ => panic!("forced to cell but not a cell")
        }
    }

    fn force_tree(&self) -> &Tree {
        match self {
            QuadNode::Tree(tree) => tree,
            _ => panic!("forced to Tree but not a tree")
        }
    }


    /// Create a new Tree node from four smaller QuadNodes.
    /// Invarient: quadrants must all be cells or all be Trees. Cannot mix.
    fn from(ne: Rc<QuadNode>, nw: Rc<QuadNode>, se: Rc<QuadNode>, sw: Rc<QuadNode>) -> QuadNode {
        use QuadNode::Cell as QCell;
        use QuadNode::Tree as QTree;
        let nodes = (ne.as_ref(), nw.as_ref(), se.as_ref(), sw.as_ref());
        let (level, population) = match nodes {
            (QCell(ne), QCell(nw), QCell(se), QCell(sw)) => {
                let level = 0;
                let population = *ne as usize + *nw as usize + *se as usize + *sw as usize;
                (level, population)
            },
            (QTree(ne), QTree(nw), QTree(se), QTree(sw)) => {
                let level = ne.level;
                assert_eq!(nw.level, level, "Quad levels do not match");
                assert_eq!(se.level, level, "Quad levels do not match");
                assert_eq!(sw.level, level, "Quad levels do not match");
                let population = ne.population + nw.population + se.population + sw.population;
                (level + 1, population)
            },
            _ => panic!("quadrants have different types")
        };
        
        let ne = Rc::clone(&ne);
        let nw = Rc::clone(&nw);
        let se = Rc::clone(&se);
        let sw= Rc::clone(&sw);
        QuadNode::Tree(Tree {
            ne, nw, se, sw, level, population
        })
    }
}

impl HashLife {
    fn join(&mut self, ne: Rc<QuadNode>, nw: Rc<QuadNode>, se: Rc<QuadNode>, sw: Rc<QuadNode>) -> Rc<QuadNode> {
        let mut hasher = DefaultHasher::new();
        ne.hash(&mut hasher);
        nw.hash(&mut hasher);
        se.hash(&mut hasher);
        sw.hash(&mut hasher);
        let hash = hasher.finish();
        if let Some(ref_to_node) = self.cache.join.get(&hash) {
            // Node already exist, return a reference to it.
            Rc::clone(ref_to_node)
        } else {
            // Create node and store in cache.
            let node = QuadNode::from(ne, nw, se, sw);
            let node = Rc::new(node);
            self.cache.join.insert(hash, Rc::clone(&node));
            self.cache.register_node(&node);
            node
        }
    }

    fn empty_tree(&mut self, level: usize) -> Rc<QuadNode> {
        if let Some(ref_to_node) = self.cache.empty.get(&level) {
            return Rc::clone(ref_to_node);
        }

        if level == 0 {
            let node = Rc::new(QuadNode::Cell(Cell::Dead));
            self.cache.register_empty(level, Rc::clone(&node));
            return node;
        }

        let sub_empty = self.empty_tree(level - 1);

        let tree = Tree {
            ne: Rc::clone(&sub_empty),
            nw: Rc::clone(&sub_empty),
            se: Rc::clone(&sub_empty),
            sw: sub_empty,
            level,
            population: 0,
        };

        let node = Rc::new(QuadNode::Tree(tree));
        self.cache.register_empty(level, Rc::clone(&node));
        node
    }

    /// Generates a quad tree with `level` levels of random cells. Similar sub
    /// trees at the same level refer to the same memory.
    /// Not efficient.
    fn random_tree(&mut self, level: usize) -> Rc<QuadNode> {
        // Base case.
        if level == 0 {
            let mut buf = [0u8; 1];
            getrandom(&mut buf).expect("random number generator failed");
            let leaf = if buf[0] % 2 == 0 {
                QuadNode::Cell(Cell::Dead)
            } else {
                QuadNode::Cell(Cell::Alive)
            };
            let node = self.cache.singlise_node(Rc::new(leaf));
            return node;
        }

        // Inductive case.
        let ne = self.random_tree(level - 1);
        let nw = self.random_tree(level - 1);
        let se = self.random_tree(level - 1);
        let sw = self.random_tree(level - 1);
        let node = self.join(ne, nw, se, sw);
        node
    }

    fn center(&mut self, node: Rc<QuadNode>) -> Rc<QuadNode> {
        let tree =  match node.as_ref() {
            QuadNode::Cell(_) => panic!("cannot center cell"),
            QuadNode::Tree(tree) => tree,
        };
        let e =  self.empty_tree(tree.level);
        let ne = self.join(Rc::clone(&e), Rc::clone(&e), Rc::clone(&e), tree.sw());
        let nw = self.join(Rc::clone(&e), Rc::clone(&e), tree.se(), Rc::clone(&e));
        let se = self.join(Rc::clone(&e), tree.nw(), Rc::clone(&e), Rc::clone(&e));
        let sw = self.join(tree.ne(), Rc::clone(&e), Rc::clone(&e), Rc::clone(&e));
        self.join(ne, nw, se, sw)
    }

    /// Calculate the next generation for the center (c) cell
    fn life(&mut self, nw: Rc<QuadNode>, n: Rc<QuadNode>, ne: Rc<QuadNode>,
                       w: Rc<QuadNode>, c: Rc<QuadNode>, e: Rc<QuadNode>,
                       sw: Rc<QuadNode>, s: Rc<QuadNode>, se: Rc<QuadNode>
    ) -> Rc<QuadNode> {
        let v = vec![nw, n, ne, w, e, sw, s, se];
        let living: usize = v.iter()
            .map(|q| q.as_ref().force_cell())
            .map(|cell| cell as usize)
            .sum();
        let c = match c.as_ref() {
            QuadNode::Cell(cell) => cell,
            _ => panic!("must calculate 3x3s on cells only")
        };
        let cell = match (c, living) {
            (Cell::Alive, 2) | (_, 3) => Cell::Alive,
            _ => Cell::Dead,
        };
        let node = Rc::new(QuadNode::Cell(cell));
        let node = self.cache.singlise_node(node);
        node
    }

    fn life4x4(&mut self, node: Rc<QuadNode>) -> Rc<QuadNode> {
        let tree = node.force_tree();
        let tl_ = tree.nw(); let tl = tl_.force_tree();
        let tr_ = tree.ne(); let tr = tr_.force_tree();
        let bl_ = tree.sw(); let bl = bl_.force_tree();
        let br_ = tree.se(); let br = br_.force_tree();
        let nw = self.life(tl.nw(), tl.ne(), tr.nw(), tl.sw(), tl.se(), tr.sw(), bl.nw(), bl.ne(), br.nw());
        let ne = self.life(tl.ne(), tr.nw(), tr.ne(), tl.se(), tr.sw(), tr.se(), bl.ne(), br.nw(), br.ne());
        let sw = self.life(tl.sw(), tl.se(), tr.sw(), bl.nw(), bl.ne(), br.nw(), bl.sw(), bl.se(), br.sw());
        let se = self.life(tl.se(), tr.sw(), tr.se(), bl.ne(), br.nw(), br.ne(), bl.se(), br.sw(), br.se());
        self.join(ne, nw, se, sw)
    }

    fn next_generation(&mut self, node: Rc<QuadNode>) -> Rc<QuadNode> {
        let mut state = DefaultHasher::new();
        node.hash(&mut state);
        let hash = state.finish();
        if let Some(ref_to_node) = self.cache.generation.get(&hash) {
            return Rc::clone(ref_to_node);
        }

        let tree = node.force_tree();
        let next = if tree.population == 0 {
            tree.nw()
        } else if tree.level == 2 {
            self.life4x4(node)
        } else {
            let tl_ = tree.nw(); let tl = tl_.force_tree();
            let tr_ = tree.ne(); let tr = tr_.force_tree();
            let bl_ = tree.sw(); let bl = bl_.force_tree();
            let br_ = tree.se(); let br = br_.force_tree();

            let c1 = self.join(tl.ne(), tl.nw(), tl.se(), tl.sw());
            let c1 = self.next_generation(c1);
            let c1 = c1.force_tree();
            let c2 = self.join(tr.nw(), tl.ne(), tr.sw(), tl.se());
            let c2 = self.next_generation(c2);
            let c2 = c2.force_tree();
            let c3 = self.join(tr.ne(), tr.nw(), tr.se(), tr.sw());
            let c3 = self.next_generation(c3);
            let c3 = c3.force_tree();
            let c4 = self.join(tl.se(), tl.sw(), bl.ne(), bl.nw());
            let c4 = self.next_generation(c4);
            let c4 = c4.force_tree();
            let c5 = self.join(tr.sw(), tl.se(), br.nw(), bl.ne());
            let c5 = self.next_generation(c5);
            let c5 = c5.force_tree();
            let c6 = self.join(tr.se(), tr.sw(), br.ne(), br.nw());
            let c6 = self.next_generation(c6);
            let c6 = c6.force_tree();
            let c7 = self.join(bl.ne(), bl.nw(), bl.se(), bl.sw());
            let c7 = self.next_generation(c7);
            let c7 = c7.force_tree();
            let c8 = self.join(br.nw(), bl.ne(), br.sw(), bl.se());
            let c8 = self.next_generation(c8);
            let c8 = c8.force_tree();
            let c9 = self.join(br.ne(), br.nw(), br.se(), br.sw());
            let c9 = self.next_generation(c9);
            let c9 = c9.force_tree();

            let nw = self.join(c2.sw(), c1.se(), c5.nw(), c4.ne());
            let ne = self.join(c3.sw(), c2.se(), c6.nw(), c5.ne());
            let sw = self.join(c5.sw(), c4.se(), c8.nw(), c7.ne());
            let se = self.join(c6.sw(), c5.se(), c9.nw(), c8.ne());
            self.join(ne, nw, se, sw)
        };

        self.cache.generation.insert(hash, Rc::clone(&next));
        next
    }
}

struct Cache {
    join: HashMap<u64, Rc<QuadNode>>,
    empty: HashMap<usize, Rc<QuadNode>>,
    nodes: HashMap<u64, Rc<QuadNode>>,
    generation: HashMap<u64, Rc<QuadNode>>,
}

impl Cache {
    fn new() -> Self {
        Self {
            join: HashMap::new(),
            empty: HashMap::new(),
            nodes: HashMap::new(),
            generation: HashMap::new(),
        }
    }

    fn register_node(&mut self, node: &Rc<QuadNode>) {
        let mut hasher = DefaultHasher::new();
        node.hash(&mut hasher);
        let hash = hasher.finish();
        if let None = self.nodes.get(&hash) {
            self.nodes.insert(hash, Rc::clone(&node));
        }
    }

    fn register_empty(&mut self, level: usize, node: Rc<QuadNode>) {
        self.register_node(&node);
        self.empty.insert(level, node);
    }

    fn singlise_node(&mut self, node: Rc<QuadNode>) -> Rc<QuadNode> {
        let mut hasher = DefaultHasher::new();
        node.hash(&mut hasher);
        let hash = hasher.finish();
        if let Some (ref_to_node) = self.nodes.get(&hash) {
            Rc::clone(ref_to_node)
        } else {
            self.register_node(&node);
            node
        }
    }
}

pub struct HashLife {
    cache: Cache,
    top: QuadNode,
    generation: usize,
    width: usize,
    height: usize,
}

impl HashLife {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn top(&self) -> QuadNode {
        self.top.clone()
    }

    // fn expandUniverse(&mut self) {
    //     if let QuadNode::Tree(tree) = self.top {
    //         let border = Tree::empty_tree(tree.level)
    //     }
        
    // }

    fn quad_to_list(node: Rc<QuadNode>, width: usize, height: usize) -> Vec<Vec<Cell>> {
        let mut list = Vec::new();
        match node.as_ref() {
            QuadNode::Cell(cell) => vec![ vec![ *cell ] ],
            QuadNode::Tree(tree) => {
                
            },
        }
        
    }
}

impl GameOfLife for HashLife {
    fn new(edge: EdgeRule) -> Self {
        match edge {
            EdgeRule::Wrap(width, height) => {
                let max_size = if width >= height { width } else { height } as f64;
                let level = max_size.sqrt().ceil()  as usize;
                let top = QuadNode::Cell(Cell::Dead);
                return Self {
                    cache: Cache::new(),
                    top,
                    generation: 0,
                    width,
                    height,
                }
            },
            EdgeRule::Truncate(width, height) => todo!(),
            EdgeRule::Infinite => todo!(),

        }

        
        todo!()
    }

    fn from_rle(edge: EdgeRule, content: String) -> Self {
        todo!()
    }

    fn tick(&mut self) {
        // todo!()
    }

    fn get_generation(&self) -> usize {
        self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_nodes_inc() {
        let mut hl = HashLife::new(EdgeRule::Wrap(2,2));
        hl.empty_tree(0);
        assert_eq!(1, hl.cache.nodes.len());
    }

    #[test]
    fn cache_empty_inc_once() {
        let mut hl = HashLife::new(EdgeRule::Wrap(2,2));
        hl.empty_tree(0);
        assert_eq!(1, hl.cache.empty.len());
    }

    #[test]
    fn cache_empty_and_nodes_inc() {
        let mut hl = HashLife::new(EdgeRule::Wrap(2,2));
        hl.empty_tree(0);
        assert_eq!(1, hl.cache.nodes.len());
        assert_eq!(1, hl.cache.empty.len());
    }

    #[test]
    fn cache_empty_and_nodes_inc_twice() {
        let mut hl = HashLife::new(EdgeRule::Wrap(2,2));
        hl.empty_tree(1);
        assert_eq!(2, hl.cache.nodes.len());
        assert_eq!(2, hl.cache.empty.len());
    }

    #[test]
    fn cache_empty_and_nodes_inc_thrice() {
        let mut hl = HashLife::new(EdgeRule::Wrap(2,2));
        hl.empty_tree(2);
        assert_eq!(3, hl.cache.nodes.len());
        assert_eq!(3, hl.cache.empty.len());
    }
}

