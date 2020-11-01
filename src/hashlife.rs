use std::collections::HashMap;
use std::rc::Rc;

use getrandom::getrandom;
use crate::{GameOfLife, EdgeRule};

// trait QuatTreeNode {
//     fn level(&self) -> usize;
// }

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuadNode {
    Cell(Cell),
    Tree(Box<Tree>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tree {
    ne: QuadNode,
    nw: QuadNode,
    se: QuadNode,
    sw: QuadNode,
    level: usize,
    population: usize,
}

impl Tree {
    /// Create a new Tree node from four smaller QuadNodes.
    /// Invarient: quadrants must all be cells or all be Trees. Cannot mix.
    fn from(ne: QuadNode, nw: QuadNode, se: QuadNode, sw: QuadNode) -> QuadNode {
        use QuadNode::Cell as QCell;
        use QuadNode::Tree as QTree;
        let nodes = (&ne, &nw, &se, &sw);
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
        QuadNode::Tree(Box::new(Tree {
            ne, nw, se, sw, level, population
        }))
    }

    fn empty_tree(level: usize) -> QuadNode {
        if level == 0 {
            return QuadNode::Cell(Cell::Dead);
        }

        let node = Tree {
            ne: Tree::empty_tree(level - 1),
            nw: Tree::empty_tree(level - 1),
            se: Tree::empty_tree(level - 1),
            sw: Tree::empty_tree(level - 1),
            level,
            population: 0,
        };

        QuadNode::Tree(Box::new(node))
    }

    fn random_tree(level: usize) -> QuadNode {
        if level == 0 {
            let mut buf = [0u8; 1];
            getrandom(&mut buf).expect("random number generator failed");
            if buf[0] % 2 == 0 {
                return QuadNode::Cell(Cell::Dead);
            } else {
                return QuadNode::Cell(Cell::Alive);
            }
        }
        let ne = Tree::random_tree(level - 1);
        let nw = Tree::random_tree(level - 1);
        let se = Tree::random_tree(level - 1);
        let sw = Tree::random_tree(level - 1);

        let population: usize = [&ne, &nw, &se, &sw].iter()
            .map(|q| {
                match q {
                    QuadNode::Cell(c) => match c { Cell::Alive => 1, Cell::Dead => 0},
                    QuadNode::Tree(tree) => tree.population,
                }
            })
            .sum();

        let node = Tree {
            ne,
            nw,
            se,
            sw,
            level,
            population,
        };

        QuadNode::Tree(Box::new(node))
    }

    fn c_ne(&self) -> QuadNode {
        self.ne.clone()
    }

    fn c_nw(&self) -> QuadNode {
        self.nw.clone()
    }

    fn c_se(&self) -> QuadNode {
        self.se.clone()
    }

    fn c_sw(&self) -> QuadNode {
        self.sw.clone()
    }
}

impl QuadNode {
    fn set_cell(&self, x: isize, y: isize, cell: Cell) -> Self {
        match self {
            QuadNode::Cell(_) => QuadNode::Cell(cell),
            QuadNode::Tree(tree) => {
                let current_level = tree.level;
                let half: isize = 1 << (current_level - 1);
                if x < 0 { // west
                    if y < 0 { // nw
                        let (x, y) = (x + half, y + half);
                        let nw = tree.c_nw().set_cell(x, y, cell);
                        Tree::from(tree.c_ne(), nw, tree.c_se(), tree.c_sw())
                    } else { // sw
                        let (x, y) = (x + half, y - half);
                        let sw = tree.c_sw().set_cell(x, y, cell);
                        Tree::from(tree.c_ne(), tree.c_nw(), tree.c_se(), sw)
                    } // sw
                } else { // east
                    if y < 0 { // ne
                        let (x, y) = (x - half, y + half);
                        let ne = tree.c_ne().set_cell(x, y, cell);
                        Tree::from(ne, tree.c_nw(), tree.c_se(), tree.c_sw())
                    } else {
                        let (x, y) = (x - half, y - half);
                        let se = tree.c_se().set_cell(x, y, cell);
                        Tree::from(tree.c_ne(), tree.c_nw(), se, tree.c_se())
                    } // se
                }
            }
        }
    }

    pub fn get_cell(&self, x: isize, y: isize) -> Cell {
        match self {
            QuadNode::Cell(cell) => *cell,
            QuadNode::Tree(tree) => {
                let current_level = tree.level;
                let half: isize = 1 << (current_level - 1);
                if x < 0 { // west
                    if y < 0 { // nw
                        let (x, y) = (x + half, y + half);
                        tree.nw.get_cell(x, y)
                    } else { // sw
                        let (x, y) = (x + half, y - half);
                        tree.sw.get_cell(x, y)
                    } // sw
                } else { // east
                    if y < 0 { // ne
                        let (x, y) = (x - half, y + half);
                        tree.ne.get_cell(x, y)
                    } else { // se
                        let (x, y) = (x - half, y - half);
                        tree.se.get_cell(x, y)
                    }
                }
            }
        }
    }
}


pub struct HashLife {
    results_cache: HashMap<usize, QuadNode>,
    position_cache: HashMap<(usize, usize), QuadNode>,
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
}

impl GameOfLife for HashLife {
    fn new(edge: EdgeRule) -> Self {
        match edge {
            EdgeRule::Wrap(width, height) => {
                let max_size = if width >= height { width } else { height } as f64;
                let level = max_size.sqrt().ceil()  as usize;
                let top = Tree::random_tree(level);
                return Self {
                    results_cache: HashMap::new(),
                    position_cache: HashMap::new(),
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

