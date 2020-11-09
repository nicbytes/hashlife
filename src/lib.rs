mod automata;

use automata::Automata;

use std::rc::Rc;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Node {
    level: usize,
    population: usize,
    hash: u64,
    children: Option<Children>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.level == other.level && self.population == other.population && self.hash == other.hash
    }
}

impl Eq for Node {}

#[derive(Hash, Debug)]
struct Children {
    nw: Rc<Node>,
    ne: Rc<Node>,
    sw: Rc<Node>,
    se: Rc<Node>,
}

struct GrandChildren {
    nwnw: Rc<Node>,
    nwne: Rc<Node>,
    nwsw: Rc<Node>,
    nwse: Rc<Node>,
    nenw: Rc<Node>,
    nene: Rc<Node>,
    nesw: Rc<Node>,
    nese: Rc<Node>,
    swnw: Rc<Node>,
    swne: Rc<Node>,
    swsw: Rc<Node>,
    swse: Rc<Node>,
    senw: Rc<Node>,
    sene: Rc<Node>,
    sesw: Rc<Node>,
    sese: Rc<Node>,
}

struct GrandAutomata {
    nwnw: Automata,
    nwne: Automata,
    nwsw: Automata,
    nwse: Automata,
    nenw: Automata,
    nene: Automata,
    nesw: Automata,
    nese: Automata,
    swnw: Automata,
    swne: Automata,
    swsw: Automata,
    swse: Automata,
    senw: Automata,
    sene: Automata,
    sesw: Automata,
    sese: Automata,
}


pub struct Hashlife {
    cache: Cache,
    top: Option<Rc<Node>>,
}

struct Cache {
    join: HashMap<u64, Rc<Node>>,
    step: HashMap<Rc<Node>, Rc<Node>>,
    dead: Option<Rc<Node>>,
    alive: Option<Rc<Node>>,
}

impl Cache {
    fn new() -> Self {
        Self {
            join: HashMap::new(),
            step: HashMap::new(),
            dead: None,
            alive: None,
        }
    }
}

struct ConstructionParameters<'a> {
    level: usize,
    vector: &'a Vec<u8>,
    width: usize,
    height: usize,
    bound: BoundingBox,
}

struct BoundingBox {
    top: isize,
    bottom: isize,
    left: isize,
    right: isize,
}

impl BoundingBox {
    fn new(x: isize, y: isize, level: usize, max_level: usize) -> Self{
        let level_delta = level;
        let pow2ld = 2isize.pow(level_delta as u32);
        let top = y * pow2ld;
        let bottom = y * pow2ld + pow2ld - 1;
        let left = x * pow2ld;
        let right = x * pow2ld + pow2ld - 1;

        Self { top, bottom, left, right }
    }

    fn from(top: isize, bottom: isize, left: isize, right: isize) -> Self {
        Self { top, bottom, left, right }
    }

    fn collides(&self, other: &BoundingBox) -> bool {
        // up is -y, down is +y
        let other_below_self = other.top > self.bottom;
        let other_above_self = other.bottom < self.top;
        // left is -x, right is +x
        let other_right_of_self = other.left > self.right;
        let other_left_of_self = other.right < self.left;
        return !(other_below_self || other_above_self || other_right_of_self || other_left_of_self);
        // !(other.top < self.bottom || other.bottom > self.top || other.left > self.right || other.right < self.left)
    }

}

impl Hashlife {
    fn new() -> Self {
        Self {
            cache: Cache::new(),
            top: None,
        }
    }

    fn join(&mut self, nw: Rc<Node>, ne: Rc<Node>, sw: Rc<Node>, se: Rc<Node>) -> Rc<Node> {
        let children = Children::from(&nw, &ne, &sw, &se);
        let level = nw.level + 1;
        assert_eq!(nw.level, ne.level);
        assert_eq!(nw.level, sw.level);
        assert_eq!(nw.level, se.level);
        let population = nw.population + ne.population + sw.population + se.population;
        let hash = calculate_hash(&children);
        if let Some(ref_to_node) = self.cache.join.get(&hash) {
            return Rc::clone(ref_to_node);
        }
        let children = Some(children);
        let node = Node {
            level,
            population,
            hash,
            children
        };
        let node = Rc::new(node);
        self.cache.join.insert(hash, Rc::clone(&node));
        return node;
    }

    fn step(&mut self, node: Rc<Node>) -> Rc<Node> {
        if let Some(ref_to_node) = self.cache.step.get(&node) {
            return Rc::clone(ref_to_node);
        }
        let step = match &node.level {
            0 => panic!("attempted to step a node with level 0"),
            1 => panic!("attempted to step a node with level 1"),
            2 => {
                let g = node.get_grand_automata();
                let nw = automata::simb3s23(g.nwse, g.nwnw, g.nwne, g.nenw, g.nesw, g.senw, g.swne, g.swnw, g.nwsw);
                let ne = automata::simb3s23(g.nesw, g.nwne, g.nenw, g.nene, g.nese, g.sene, g.senw, g.swne, g.nwse);
                let sw = automata::simb3s23(g.swne, g.nwsw, g.nwse, g.nesw, g.senw, g.sesw, g.swse, g.swsw, g.swnw);
                let se = automata::simb3s23(g.senw, g.nwse, g.nesw, g.nese, g.sene, g.sese, g.sesw, g.swse, g.swne);
                let nw = self.make_automata(nw);
                let ne = self.make_automata(ne);
                let sw = self.make_automata(sw);
                let se = self.make_automata(se);
                self.join(nw, ne, sw, se)
            },
            _ => {
                let c = node.get_children();
                let g = node.get_grand_children();

                let nw = Rc::clone(&c.nw);
                let ne = Rc::clone(&c.ne);
                let sw = Rc::clone(&c.sw);
                let se = Rc::clone(&c.se);
                let n_ = self.join(g.nwne, g.nenw, Rc::clone(&g.nwse), Rc::clone(&g.nesw));
                let e_ = self.join(Rc::clone(&g.nesw), g.nese, Rc::clone(&g.senw), g.sene);
                let s_ = self.join(Rc::clone(&g.swne), Rc::clone(&g.senw), g.swse, g.sesw);
                let w_ = self.join(g.nwsw, Rc::clone(&g.nwse), g.swnw, Rc::clone(&g.swne));
                let c_ = self.join(g.nwse, g.nesw, g.swne, g.senw);

                let nw_step = self.step(nw);
                let ne_step = self.step(ne);
                let sw_step = self.step(sw);
                let se_step = self.step(se);
                let n_step = self.step(n_);
                let e_step = self.step(e_);
                let s_step = self.step(s_);
                let w_step = self.step(w_);
                let c_step = self.step(c_);

                let nw_res = self.join(
                    Rc::clone(&nw_step.get_children().se),
                    Rc::clone(&n_step.get_children().sw),
                    Rc::clone(&w_step.get_children().ne),
                    Rc::clone(&c_step.get_children().nw)
                );
                let ne_res = self.join(
                    Rc::clone(&n_step.get_children().se),
                    Rc::clone(&ne_step.get_children().sw),
                    Rc::clone(&c_step.get_children().ne),
                    Rc::clone(&e_step.get_children().nw)
                );
                let sw_res = self.join(
                    Rc::clone(&w_step.get_children().se),
                    Rc::clone(&c_step.get_children().sw),
                    Rc::clone(&sw_step.get_children().ne),
                    Rc::clone(&s_step.get_children().nw)
                );
                let se_res = self.join(
                    Rc::clone(&c_step.get_children().se),
                    Rc::clone(&e_step.get_children().sw),
                    Rc::clone(&s_step.get_children().ne),
                    Rc::clone(&se_step.get_children().nw),
                );
                self.join(nw_res, ne_res, sw_res, se_res)
            },
        };
        self.cache.step.insert(node, Rc::clone(&step));
        step
    }

    fn make_automata(&mut self, a: Automata) -> Rc<Node> {
        match a {
            Automata::Dead => {
                if let Some(ref_to_node) = &self.cache.dead {
                    Rc::clone(ref_to_node)
                } else {
                    let mut state = DefaultHasher::new();
                    a.hash(&mut state);
                    let node = Rc::new(Node {
                        level: 0,
                        population: a as usize,
                        children: None,
                        hash: state.finish(),
                    });
                    self.cache.dead = Some(Rc::clone(&node));
                    node
                }
            },
            Automata::Alive => {
                if let Some(ref_to_node) = &self.cache.alive {
                    Rc::clone(ref_to_node)
                } else {
                    let mut state = DefaultHasher::new();
                    a.hash(&mut state);
                    let node = Rc::new(Node {
                        level: 0,
                        population: a as usize,
                        children: None,
                        hash: state.finish(),
                    });
                    self.cache.alive = Some(Rc::clone(&node));
                    node
                }
            },
        }
    }

    /// Construct a Hashlife program given an array of states.
    pub fn from_array(size: usize, array: Vec<u8>, width: usize, height: usize) -> Self {
        //
        let mut hashlife = Hashlife::new();

        // Calculat
        let min_width = -(width as isize / 2);
        let max_width = width as isize - min_width;
        let min_height = -(height as isize / 2);
        let max_height = height as isize - min_height;
        let bound = BoundingBox::from(min_height, max_height, min_width, max_width);
        // Pack some configuration parameters to build the first generation.
        let params = ConstructionParameters {
            level: size,
            vector: &array,
            width,
            height,
            bound,
        };

        let top = hashlife.construct(0, 0, size, &params);
        hashlife.top = Some(top);
        hashlife
    }

    /// Recursively build a Quad tree.
    fn construct(&mut self, x: isize, y: isize, level: usize, params: &ConstructionParameters) -> Rc<Node> {
        // Base case: retrieve value from cell
        if level == 0 {
            let xx = ((params.width / 2) as isize + x) as usize;
            let yx = ((params.height / 2) as isize + y) as usize;
            let idx = params.width * yx + xx;
            let v = params.vector[idx];
            let a = if v % 2 == 0 { Automata::Dead } else { Automata::Alive };
            return self.make_automata(a);
        }

        // Small helper function, speed up construction if building an empty region.
        let mut assemble = |dx, dy| {
            let bound = BoundingBox::new(x, y, level,  params.level);
            if bound.collides(&params.bound) {
                self.construct(x * 2 + dx, y * 2 + dy, level - 1, &params)
            } else {
                self.empty(level - 1)
            }
        };

        let nw = assemble(0, 0);
        let ne = assemble(1, 0);
        let sw = assemble(0, 1);
        let se = assemble(1, 1);

        self.join(nw, ne, sw, se)
    }

    /// Construct an empty Quad Node at the specified level.
    fn empty(&mut self, level: usize) -> Rc<Node> {
        // Base case
        if level == 0 {
            return self.make_automata(Automata::Dead);
        }
        // Construct children.
        let child = self.empty(level - 1);
        let children = Children {
            nw: Rc::clone(&child),
            ne: Rc::clone(&child),
            sw: Rc::clone(&child),
            se: Rc::clone(&child),
        };
        let hash = calculate_hash(&children);
        // Check if node already exists in the cache.
        if let Some(ref_to_node) = self.cache.join.get(&hash) {
            return Rc::clone(ref_to_node);
        };
        let empty = Rc::new(Node {
            level,
            population: 0,
            children: Some(children),
            hash,
        });
        // Add node to cache.
        self.cache.join.insert(hash, Rc::clone(&empty));
        empty
    }

    fn get_node_with(&self, x: isize, y: isize, positions: &Vec<(isize, isize)>, node: Rc<Node>) -> Automata {
        if node.level == 0 {
            return node.as_automata();
        }
        let position = positions[node.level - 2];
        println!("Position: {:?}, level: {},  NW {:?}, NE {:?}, SW {:?}, SE {:?}",
            position, node.level,
            (x*2, y*2),(x*2+1, y*2),(x*2, y*2+1),(x*2+1, y*2+1)
        );
        let nw = (x*2, y*2);
        let ne = (x*2+1, y*2);
        let sw = (x*2, y*2+1);
        let se = (x*2+1, y*2+1);
        let children = node.get_children();
        if position == nw {
            return self.get_node_with(nw.0, nw.1, positions, Rc::clone(&children.nw));
        } else if position == ne {
            return self.get_node_with(ne.0, ne.1, positions, Rc::clone(&children.ne));
        } else if position == sw {
            return self.get_node_with(sw.0, sw.1, positions, Rc::clone(&children.sw));
        } else if position == se {
            return self.get_node_with(se.0, se.1, positions, Rc::clone(&children.se));
        } else {
            panic!("invalid coordinate calculated");
        }
    }

    pub fn get(&self, x: isize, y: isize) -> Option<Automata> {
        let top = if let Some(top) = self.top.as_ref() {
            Rc::clone(top)
        } else {
            return None;
        };

        let mut positions = Vec::with_capacity(top.level);
        let mut xx = x;
        let mut yy = y;
        for _ in 0..top.level-2 {
            positions.push((xx,yy));
            xx /= 2;
            yy /= 2;
        }
        println!("positions: {:?}", positions);
        println!("levels: {:?}", top.level);

        // TODO: what if level == 0?
        let children = top.get_children();

        if y < 0 {
            if x < 0 { // NW
                Some(self.get_node_with(-1, -1, &positions, Rc::clone(&children.nw)))
            } else { // NE
                Some(self.get_node_with(0, -1, &positions, Rc::clone(&children.ne)))
            }
        } else {
            if x < 0 { // SW
                Some(self.get_node_with(-1, 0, &positions, Rc::clone(&children.sw)))
            } else { // SE
                Some(self.get_node_with(0, 0, &positions, Rc::clone(&children.se)))
            }
        }
    }
}


impl Node {

    fn get_children(&self) ->&Children {
        &self.children.as_ref().unwrap()
    }

    fn get_grand_children(&self) -> GrandChildren {
        let err1 = "unable to unwrap child (and expecting grand-children)";
        let err2 = "unable to unwrap grand-children";
        GrandChildren {
            nwnw: Rc::clone(&self.children.as_ref().expect(err1).nw.children.as_ref().expect(err2).nw),
            nwne: Rc::clone(&self.children.as_ref().expect(err1).nw.children.as_ref().expect(err2).ne),
            nwsw: Rc::clone(&self.children.as_ref().expect(err1).nw.children.as_ref().expect(err2).sw),
            nwse: Rc::clone(&self.children.as_ref().expect(err1).nw.children.as_ref().expect(err2).se),
            nenw: Rc::clone(&self.children.as_ref().expect(err1).ne.children.as_ref().expect(err2).nw),
            nene: Rc::clone(&self.children.as_ref().expect(err1).ne.children.as_ref().expect(err2).ne),
            nesw: Rc::clone(&self.children.as_ref().expect(err1).ne.children.as_ref().expect(err2).sw),
            nese: Rc::clone(&self.children.as_ref().expect(err1).ne.children.as_ref().expect(err2).se),
            swnw: Rc::clone(&self.children.as_ref().expect(err1).sw.children.as_ref().expect(err2).nw),
            swne: Rc::clone(&self.children.as_ref().expect(err1).sw.children.as_ref().expect(err2).ne),
            swsw: Rc::clone(&self.children.as_ref().expect(err1).sw.children.as_ref().expect(err2).sw),
            swse: Rc::clone(&self.children.as_ref().expect(err1).sw.children.as_ref().expect(err2).se),
            senw: Rc::clone(&self.children.as_ref().expect(err1).se.children.as_ref().expect(err2).nw),
            sene: Rc::clone(&self.children.as_ref().expect(err1).se.children.as_ref().expect(err2).ne),
            sesw: Rc::clone(&self.children.as_ref().expect(err1).se.children.as_ref().expect(err2).sw),
            sese: Rc::clone(&self.children.as_ref().expect(err1).se.children.as_ref().expect(err2).se),
        }
    }

    fn get_grand_automata(&self) -> GrandAutomata {
        if self.level != 2 {
            panic!("node must be at level 2 to get automatas");
        }
        let grand_children = self.get_grand_children();
        GrandAutomata {
            nwnw: grand_children.nwnw.as_automata(),
            nwne: grand_children.nwne.as_automata(),
            nwsw: grand_children.nwsw.as_automata(),
            nwse: grand_children.nwse.as_automata(),
            nenw: grand_children.nenw.as_automata(),
            nene: grand_children.nene.as_automata(),
            nesw: grand_children.nesw.as_automata(),
            nese: grand_children.nese.as_automata(),
            swnw: grand_children.swnw.as_automata(),
            swne: grand_children.swne.as_automata(),
            swsw: grand_children.swsw.as_automata(),
            swse: grand_children.swse.as_automata(),
            senw: grand_children.senw.as_automata(),
            sene: grand_children.sene.as_automata(),
            sesw: grand_children.sesw.as_automata(),
            sese: grand_children.sese.as_automata(),
        }
    }

    fn as_automata(&self) -> Automata {
        Automata::from(self.population)
    }

    fn from_automata(cell: Automata) -> Node {
        let mut state = DefaultHasher::new();
        cell.hash(&mut state);
        Node {
            level: 0,
            population: cell as usize,
            children: None,
            hash: state.finish(),
        }

    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Children {
    fn from(nw: &Rc<Node>, ne: &Rc<Node>, sw: &Rc<Node>, se: &Rc<Node>) -> Self {
        Self {
            nw: Rc::clone(nw),
            ne: Rc::clone(ne),
            sw: Rc::clone(sw),
            se: Rc::clone(se),
        }
    }
}

fn calculate_hash(children: &Children) -> u64 {
    let mut state = DefaultHasher::new();
    children.hash(&mut state);
    state.finish()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get() {
        let cell_width = 4;
        let cell_height = 4;
        let cells = vec![
            1,1,1,1,
            1,1,0,1,
            1,1,1,1,
            1,1,1,1,
        ];
        let hashlife = Hashlife::from_array(3, cells, cell_width, cell_height);
        for x in -2..2 {
            for y in -2..2 {
                if x == 0 && y == -1 { continue; }
                assert_eq!(hashlife.get(x, y), Some(Automata::Alive));
            }
        }
        assert_eq!(hashlife.get(0, -1), Some(Automata::Dead));
    }

    #[test]
    fn get8() {
        let cell_width = 8;
        let cell_height = 8;
        let cells = vec![
            1,1,1,1, 1,1,0,1,
            1,1,1,1, 1,1,1,1,
            1,1,1,1, 1,1,1,1,
            1,1,0,1, 1,1,1,1,
            
            1,1,1,1, 1,1,1,1,
            1,1,1,1, 1,1,1,1,
            1,0,1,1, 1,1,1,1,
            1,1,1,1, 1,1,1,1,
        ];
        let hashlife = Hashlife::from_array(3, cells, cell_width, cell_height);
        println!("here");
        // Path is meant to be (0,-1) -> (1,-2) -> (2,-4)
        let res = hashlife.get(2, -4);
        assert_eq!(res, Some(Automata::Dead));
        assert_eq!(hashlife.get(-2, -1), Some(Automata::Dead));
        assert_eq!(hashlife.get(-3, 2), Some(Automata::Dead));
        for x in -4..4 {
            for y in -4..4 {
                if x == 2 && y == -4 { continue; }
                if x == -2 && y == -1 { continue; }
                if x == -3 && y == 2 { continue; }
                assert_eq!(hashlife.get(x, y), Some(Automata::Alive));
            }
        }
    }

    #[test]
    fn array_construct_even() {
        let cell_width = 4;
        let cell_height = 6;
        let cells = vec![
            1,0,0,1,
            1,1,0,1,
            1,0,1,1,
            1,0,0,1,
            0,0,1,0,
            0,1,0,1
        ];
        let mut hashlife = Hashlife::from_array(3, cells, cell_width, cell_height);

        // two left most columns
        for x in -4..-2 {
            for y in -4..4 {
                assert_eq!(hashlife.get(x, y), Some(Automata::Dead));
            }
        }

        // two right most columns
        for x in 2..4 {
            for y in -4..4 {
                assert_eq!(hashlife.get(x, y), Some(Automata::Dead));
            }
        }

        // top and bottom rows
        for x in -2..2 {
            for y in [-4, 3].iter() {
                assert_eq!(hashlife.get(x, *y), Some(Automata::Dead));
            }
        }

        // row 1
        assert_eq!(hashlife.get(-2,-3), Some(Automata::Alive));
        assert_eq!(hashlife.get(-1,-3), Some(Automata::Dead));
        assert_eq!(hashlife.get( 0,-3), Some(Automata::Dead));
        assert_eq!(hashlife.get( 1,-3), Some(Automata::Alive));
        // row 2
        assert_eq!(hashlife.get(-2,-2), Some(Automata::Alive));
        assert_eq!(hashlife.get(-1,-2), Some(Automata::Alive));
        assert_eq!(hashlife.get( 0,-2), Some(Automata::Dead));
        assert_eq!(hashlife.get( 1,-2), Some(Automata::Alive));
        // row 3
        assert_eq!(hashlife.get(-2,-1), Some(Automata::Alive));
        assert_eq!(hashlife.get(-1,-1), Some(Automata::Dead));
        assert_eq!(hashlife.get( 0,-1), Some(Automata::Alive));
        assert_eq!(hashlife.get( 1,-1), Some(Automata::Alive));
        // row 4
        assert_eq!(hashlife.get(-2, 0), Some(Automata::Alive));
        assert_eq!(hashlife.get(-1, 0), Some(Automata::Dead));
        assert_eq!(hashlife.get( 0, 0), Some(Automata::Dead));
        assert_eq!(hashlife.get( 1, 0), Some(Automata::Alive));
        // row 5
        assert_eq!(hashlife.get(-2, 1), Some(Automata::Dead));
        assert_eq!(hashlife.get(-1, 1), Some(Automata::Dead));
        assert_eq!(hashlife.get( 0, 1), Some(Automata::Alive));
        assert_eq!(hashlife.get( 1, 1), Some(Automata::Dead));
        // row 6
        assert_eq!(hashlife.get(-2, 2), Some(Automata::Dead));
        assert_eq!(hashlife.get(-1, 2), Some(Automata::Alive));
        assert_eq!(hashlife.get( 0, 2), Some(Automata::Dead));
        assert_eq!(hashlife.get( 1, 2), Some(Automata::Alive));


        // top and bottom rows
        // assert_eq!(hashlife.get(-2,-4), Some(Automata::Dead));
        // assert_eq!(hashlife.get(-1,-4), Some(Automata::Dead));
        // assert_eq!(hashlife.get( 0,-4), Some(Automata::Dead));
        // assert_eq!(hashlife.get( 1,-4), Some(Automata::Dead));

        // // bottom
        // assert_eq!(hashlife.get(-2, 3), Some(Automata::Dead));
        // assert_eq!(hashlife.get(-1, 3), Some(Automata::Dead));
        // assert_eq!(hashlife.get( 0, 3), Some(Automata::Dead));
        // assert_eq!(hashlife.get( 1, 3), Some(Automata::Dead));
    }
}