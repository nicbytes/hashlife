use std::collections::HashMap;
use std::rc::Rc;

use getrandom::getrandom;
use crate::{GameOfLife, EdgeRule};

// trait QuatTreeNode {
//     fn level(&self) -> usize;
// }

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cell {
    Dead = 0,
    Alive = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QuadNode {
    Cell(Cell),
    QuadTree(Rc<QuadTree>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct QuadTree {
    ne: QuadNode,
    nw: QuadNode,
    se: QuadNode,
    sw: QuadNode,
    level: usize,
    population: usize,
}

impl QuadTree {
    fn empty_tree(level: usize) -> QuadNode {
        if level == 0 {
            return QuadNode::Cell(Cell::Dead);
        }

        let node = QuadTree {
            ne: QuadTree::empty_tree(level - 1),
            nw: QuadTree::empty_tree(level - 1),
            se: QuadTree::empty_tree(level - 1),
            sw: QuadTree::empty_tree(level - 1),
            level,
            population: 0,
        };

        QuadNode::QuadTree(Rc::new(node))
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

impl GameOfLife for HashLife {
    fn new(edge: EdgeRule) -> Self {

        let mut randoms = vec![0u8; (width * height) as usize];
        getrandom(&mut randoms[..]).expect("random num generation failed.");
        let cells = randoms.iter().map(|r| if r%2==0 {Cell::Dead} else {Cell::Alive}).collect();
        todo!()
    }

    fn from_rle(edge: EdgeRule, content: String) -> Self {
        todo!()
    }

    fn tick(&mut self) {
        todo!()
    }

    fn get_generation(&self) -> usize {
        todo!()
    }
}

