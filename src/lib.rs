mod automata;

pub use automata::Automata;

use std::rc::Rc;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A `Node` represents the top of a tree (or subtree) in the Hashlife data
/// structure. The state of Hashlife is stored in a `Node` and its children
/// forming a state tree.
/// 
/// # Level
/// 
/// Each `Node` has a `level` where `0` is a leaf and
/// any positive number is a branch. Given a level 'x', it can be derived
/// that the node is `x` nodes away from the leaf level. All leaves are on
/// the same level.
/// 
/// # Population
/// 
/// Each node has a `population` informing how many living `Automata::Alive`
/// leaves this subtree constains. If the `Node` is a leaf, then the
/// popilation can only be 1 or 0 respectively representing
/// `Automata::Alive` or `Automata::Dead`.
/// 
/// # Children
/// 
/// The `Node` in a hashlife algorithm is known as a QuadTree where the node
/// points to four child nodes. The children field is an optional and is
/// `None` only when it is a leaf node at `level=0`.
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

pub enum Edge {
    Torus,
    Truncate,
    Infinite,
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

/// A nonant is a 1/9 separation of a space. This structure represents a node
/// that is separated into 9 nonants.
/// 
/// Invarient: The node that constructs this nonant collection must have
/// `level>=3`.
struct Nonants {
    nw: Rc<Node>,
    ne: Rc<Node>,
    sw: Rc<Node>,
    se: Rc<Node>,
    n_: Rc<Node>,
    e_: Rc<Node>,
    s_: Rc<Node>,
    w_: Rc<Node>,
    c_: Rc<Node>,
}

pub struct Hashlife {
    cache: Cache,
    edge: Edge,
    top: Option<Rc<Node>>,
    previous: Option<Rc<Node>>,
    gen: usize,
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


pub struct BoundingBox {
    top: isize,
    bottom: isize,
    left: isize,
    right: isize,
}

impl BoundingBox {
    fn new(x: isize, y: isize, level: usize) -> Self {
        let level_delta = level;
        let pow2ld = 2isize.pow(level_delta as u32);
        let top = y * pow2ld + pow2ld - 1;
        let bottom = y * pow2ld;
        let left = x * pow2ld;
        let right = x * pow2ld + pow2ld - 1;

        assert!(top >= bottom, "top: {}, bottom: {}", top, bottom);
        assert!(left <= right, "left: {}, right: {}", left, right);

        Self { top, bottom, left, right }
    }

    pub fn from(top: isize, bottom: isize, left: isize, right: isize) -> Self {
        Self { top, bottom, left, right }
    }

    fn collides(&self, other: &BoundingBox) -> bool {
        // up is -y, down is +y
        let other_below_self = other.top < self.bottom;
        let other_above_self = other.bottom > self.top;
        // left is -x, right is +x
        let other_right_of_self = other.left > self.right;
        let other_left_of_self = other.right < self.left;
        return !(other_below_self || other_above_self || other_right_of_self || other_left_of_self);
        // !(other.top < self.bottom || other.bottom > self.top || other.left > self.right || other.right < self.left)
    }

    fn width(&self) -> usize {
        (self.right - self.left + 1) as usize
    }

    fn height(&self) -> usize {
        (self.top - self.bottom + 1) as usize
    }

    fn index(&self, x: isize, y: isize) -> usize {
        let width = (self.right - self.left) as usize + 1;
        let idx_height = (self.top - self.bottom) as usize;
        let x_adjusted = (x - self.left) as usize;
        let y_adjusted = (y - self.bottom) as usize;
        width * (idx_height - y_adjusted) + x_adjusted
    }

}

impl Hashlife {
    fn new() -> Self {
        Self {
            cache: Cache::new(),
            edge: Edge::Infinite,
            top: None,
            previous: None,
            gen: 0,
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

    /// Separates a node into 9 
    fn into_nonants(&mut self, node: Rc<Node>) -> Nonants {
        match &node.level {
            0 => panic!("attempted to bread node into 9x9 at level 0"),
            1 => panic!("attempted to bread node into 9x9 at level 1"),
            2 => panic!("attempted to bread node into 9x9 at level 2"),
            _ => ()
        };

        let c = node.get_children();
        let g = node.get_grand_children();

        Nonants {
            nw: Rc::clone(&c.nw),
            ne: Rc::clone(&c.ne),
            sw: Rc::clone(&c.sw),
            se: Rc::clone(&c.se),
            n_: self.join(g.nwne, g.nenw, Rc::clone(&g.nwse), Rc::clone(&g.nesw)),
            e_: self.join(Rc::clone(&g.nesw), g.nese, Rc::clone(&g.senw), g.sene),
            s_: self.join(Rc::clone(&g.swne), Rc::clone(&g.senw), g.swse, g.sesw),
            w_: self.join(g.nwsw, Rc::clone(&g.nwse), g.swnw, Rc::clone(&g.swne)),
            c_: self.join(g.nwse, g.nesw, g.swne, g.senw),
        }
    }

    fn join_nonants(&mut self, nodes: Nonants) -> Rc<Node> {
        let nw_res = self.join(
            Rc::clone(&nodes.nw.get_children().se),
            Rc::clone(&nodes.n_.get_children().sw),
            Rc::clone(&nodes.w_.get_children().ne),
            Rc::clone(&nodes.c_.get_children().nw)
        );
        let ne_res = self.join(
            Rc::clone(&nodes.n_.get_children().se),
            Rc::clone(&nodes.ne.get_children().sw),
            Rc::clone(&nodes.c_.get_children().ne),
            Rc::clone(&nodes.e_.get_children().nw)
        );
        let sw_res = self.join(
            Rc::clone(&nodes.w_.get_children().se),
            Rc::clone(&nodes.c_.get_children().sw),
            Rc::clone(&nodes.sw.get_children().ne),
            Rc::clone(&nodes.s_.get_children().nw)
        );
        let se_res = self.join(
            Rc::clone(&nodes.c_.get_children().se),
            Rc::clone(&nodes.e_.get_children().sw),
            Rc::clone(&nodes.s_.get_children().ne),
            Rc::clone(&nodes.se.get_children().nw),
        );
        self.join(nw_res, ne_res, sw_res, se_res)
    }

    /// Invarient: Node.level >= 2
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

                let mut g9x9 = self.into_nonants(Rc::clone(&node));

                g9x9.nw = self.step(g9x9.nw);
                g9x9.ne = self.step(g9x9.ne);
                g9x9.sw = self.step(g9x9.sw);
                g9x9.se = self.step(g9x9.se);
                g9x9.n_ = self.step(g9x9.n_);
                g9x9.e_ = self.step(g9x9.e_);
                g9x9.s_ = self.step(g9x9.s_);
                g9x9.w_ = self.step(g9x9.w_);
                g9x9.c_ = self.step(g9x9.c_);

                self.join_nonants(g9x9)
            },
        };
        self.cache.step.insert(node, Rc::clone(&step));
        step
    }

    fn expand_empty_border(&mut self, node: Rc<Node>) -> Rc<Node> {

        let debug = |n: &Rc<Node>| {
            let v = n.as_array().into_iter().map(|arr| arr.into_iter().map(|a| a as usize).collect::<Vec<usize>>()).collect::<Vec<Vec<usize>>>();
            for row in v.iter() {
                println!("{:?}", row);
            }
        };

        let c = node.get_children();
        let e = self.empty(node.level-1);
        let e = || Rc::clone(&e);
        let nw = self.join(e(), e(), e(), Rc::clone(&c.nw));
        let ne = self.join(e(), e(), Rc::clone(&c.ne), e());
        let sw = self.join(e(), Rc::clone(&c.sw), e(), e());
        let se = self.join(Rc::clone(&c.se), e(), e(), e());
        println!("nw:");debug(&nw);
        println!("ne:");debug(&ne);
        println!("sw:");debug(&sw);
        println!("se:");debug(&se);
        self.join(nw, ne, sw, se)
    }

    pub fn next_generation(&mut self) {
        let top = if let Some(top) = &self.top {
            Rc::clone(top)
        } else {
            return;
        };
        self.previous = Some(Rc::clone(&top));
        let next = match self.edge {
            Edge::Infinite => {
                // Expand
                // given top level is n
                // expanded level is n + 1
                let expanded = self.expand_empty_border(Rc::clone(&top));
                // expanded level is n + 2
                let expanded = self.expand_empty_border(Rc::clone(&expanded));
                // step level is n + 1
                let step = self.step(expanded);
                // Check if there is population in the border
                let g = step.get_grand_children();
                let boarder_population = step.population - g.nwse.population - g.nesw.population - g.swne.population - g.senw.population;
                if boarder_population == 0 {
                    // result level is n
                    self.join(g.nwse, g.nesw, g.swne, g.senw)
                } else {
                    // result level is n + 1
                    step
                }
            },
            Edge::Torus => {
                let c = top.get_children();
                let nw = Rc::clone(&c.nw);
                let ne = Rc::clone(&c.ne);
                let sw = Rc::clone(&c.sw);
                let se = Rc::clone(&c.se);
                let inverted = self.join(se, sw, ne, nw);
                let inverted = || Rc::clone(&inverted);
                let expanded = self.join(inverted(), inverted(), inverted(), inverted());
                self.step(expanded)
            },
            Edge::Truncate => {
                let expanded = self.expand_empty_border(Rc::clone(&top));
                self.step(expanded)
            },
        };
        self.top = Some(next);
        self.gen += 1;
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
    pub fn from_array(buffer: Vec<u8>, width: usize, height: usize, edge: Edge) -> Self {
        assert_eq!(buffer.len(), width * height);
        //
        let mut hashlife = Hashlife::new();

        // center on x-axis and negative on left
        let left = -(width as isize / 2);
        let right = width as isize + left - 1;
        // center on y-axis and nevative is up
        let bottom = -(height as isize / 2);
        let top = height as isize + bottom - 1;
        let bound = BoundingBox::from(top, bottom, left, right);

        let larger_length = *[width, height].iter().max().unwrap_or(&width) as f64;
        let size = larger_length.log2().ceil() as usize;

        // assert_eq!(bound.width(), width);
        // assert_eq!(bound.height(), height);

        if size == 0 {
            hashlife.top = Some(hashlife.make_automata(Automata::from(buffer[0] as usize)));
            return hashlife;
        }

        // Pack some configuration parameters to build the first generation.
        let params = ConstructionParameters {
            level: size,
            vector: &buffer,
            width,
            height,
            bound,
        };

        let nw = hashlife.construct(-1, 0, size - 1, &params);
        let ne= hashlife.construct(0, 0, size - 1, &params);
        let sw= hashlife.construct(-1, -1, size - 1, &params);
        let se= hashlife.construct(0, -1, size - 1, &params);
        let top = hashlife.join(nw, ne, sw, se);

        hashlife.top = Some(top);
        hashlife
    }

    /// Recursively build a Quad tree.
    fn construct(&mut self, x: isize, y: isize, level: usize, params: &ConstructionParameters) -> Rc<Node> {
        // Base case: retrieve value from cell
        if level == 0 {
            let bound = BoundingBox::new(x, y, level);
            if !bound.collides(&params.bound) {
                return self.empty(0);
            }
            let xidx = (x - params.bound.left) as usize;
            let yidx = params.height - 1 - (y - params.bound.bottom) as usize;
            let idx = params.width * yidx + xidx;
            let v = params.vector[idx];
            let a = Automata::from(v as usize);
            return self.make_automata(a);
        }

        // Small helper function, speed up construction if building an empty region.
        let mut assemble = |dx, dy| {
            let bound = BoundingBox::new(x, y, level-1);
            if bound.collides(&params.bound) {
                self.construct(x * 2 + dx, y * 2 + dy, level - 1, &params)
            } else {
                self.empty(level - 1)
            }
        };

        let nw = assemble(0, 1);
        let ne = assemble(1, 1);
        let sw = assemble(0, 0);
        let se = assemble(1, 0);

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
        let position = positions[node.level-1];
        let nw = (x*2, y*2+1);
        let ne = (x*2+1, y*2+1);
        let sw = (x*2, y*2);
        let se = (x*2+1, y*2);
        println!("Position: {:?}, level: {},  NW {:?}, NE {:?}, SW {:?}, SE {:?}",
            position, node.level,
            nw,ne,sw,se
        );
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
        for _ in 0..top.level {
            positions.push((xx,yy));
            xx = xx.div_euclid(2);
            yy = yy.div_euclid(2);
        }
        println!("------------------------------------------------------------------------------------");
        println!("positions: {:?}", positions);
        println!("top level: {:?}", top.level);
        println!("target: {:?}", (x, y));

        // TODO: what if level == 0?
        let children = top.get_children();

        if y < 0 {
            if x < 0 { // SW
                Some(self.get_node_with(-1, -1, &positions, Rc::clone(&children.sw)))
            } else { // SE
                Some(self.get_node_with(0, -1, &positions, Rc::clone(&children.se)))
            }
        } else {
            if x < 0 { // NW
                Some(self.get_node_with(-1, 0, &positions, Rc::clone(&children.nw)))
            } else { // NE
                Some(self.get_node_with(0, 0, &positions, Rc::clone(&children.ne)))
            }
        }
    }

    /// Returns the maximum node level in the tree. Setting n to the result,
    /// the number of levels is n + 1.
    fn max_level(&self) -> usize {
        if let Some(top) = &self.top {
            top.level
        } else {
            0
        }
    }


    /// Draw automata that differes from the previous generation in the given array.
    pub fn draw_diff_to_viewport_array(&mut self, buffer: &mut [u8], viewport: BoundingBox) {
        // case where the cell only contains 1 level.
        if self.max_level() == 0 {
            if let Some(top) = self.top.as_ref() {
                buffer[0] = top.population as u8;
            }
            return;
        }
        let top = if let Some(top) = &self.top {
            Rc::clone(top)
        } else {
            return;
        };
        let previous = if let Some(previous) = &self.previous {
            if previous.level == 0 {
                return;
            }
            Rc::clone(previous)
        } else {
            return;
        };

        let top_children = top.get_children();
        let previous_children = previous.get_children();
        let t_nw = Rc::clone(&top_children.nw);
        let t_ne = Rc::clone(&top_children.ne);
        let t_sw = Rc::clone(&top_children.sw);
        let t_se = Rc::clone(&top_children.se);
        let p_nw = Rc::clone(&previous_children.nw);
        let p_ne = Rc::clone(&previous_children.ne);
        let p_sw = Rc::clone(&previous_children.sw);
        let p_se = Rc::clone(&previous_children.se);
        if t_nw != p_nw {
            self.draw_diff_to_cell(buffer, t_nw, p_nw, &viewport, -1, 0);
        }
        if t_ne != p_ne {
            self.draw_diff_to_cell(buffer, t_ne, p_ne, &viewport, 0, 0);
        }
        if t_sw != p_sw {
            self.draw_diff_to_cell(buffer, t_sw, p_sw, &viewport, -1, -1);
        }
        if t_se != p_se {
            self.draw_diff_to_cell(buffer, t_se, p_se, &viewport, 0, -1);
        }
    }

    /// Helper function for drawing the node to the buffer. Children of the node
    /// will not be drawn if they are equal to the previous respective children.
    fn draw_diff_to_cell(&mut self, buffer: &mut [u8], node: Rc<Node>, previous: Rc<Node>, viewport: &BoundingBox, x: isize, y: isize) {
        let area = BoundingBox::new(x, y, node.level);
        if !area.collides(&viewport) {
            return;
        }

        if node.level == 0 {
            buffer[viewport.index(x, y)] = node.population as u8;
        } else {
            let mut draw_down = |dx: isize, dy: isize, n: Rc<Node>, p: Rc<Node>| {
                if n == p { return; }
                self.draw_diff_to_cell(&mut buffer[..], n, p, &viewport, 2*x+dx, 2*y+dy);
            };
            let c = node.get_children();
            let pc = previous.get_children();
            draw_down(0, 1, Rc::clone(&c.nw), Rc::clone(&pc.nw));
            draw_down(1, 1, Rc::clone(&c.ne), Rc::clone(&pc.ne));
            draw_down(0, 0, Rc::clone(&c.sw), Rc::clone(&pc.sw));
            draw_down(1, 0, Rc::clone(&c.se), Rc::clone(&pc.se));
        }
    }

    pub fn draw_to_viewport_buffer(&mut self, buffer: &mut [u8], viewport: BoundingBox) {
        if self.max_level() == 0 {
            if let Some(top) = self.top.as_ref() {
                buffer[0] = top.population as u8;
            }
            return;
        }
        let top = Rc::clone(self.top.as_ref().unwrap());
        let c = top.get_children();
        let nw = Rc::clone(&c.nw);
        let ne = Rc::clone(&c.ne);
        let sw = Rc::clone(&c.sw);
        let se = Rc::clone(&c.se);
        self.draw_to_cell(buffer, nw, &viewport, -1, 0);
        self.draw_to_cell(buffer, ne, &viewport, 0, 0);
        self.draw_to_cell(buffer, sw, &viewport, -1, -1);
        self.draw_to_cell(buffer, se, &viewport, 0, -1);
    }

    /// Helper function for drawing the entire tree to a buffer
    fn draw_to_cell(&mut self, buffer: &mut [u8], node: Rc<Node>, viewport: &BoundingBox, x: isize, y: isize) {
        let area = BoundingBox::new(x, y, node.level);
        if !area.collides(&viewport) {
            return;
        }

        if node.level == 0 {
            buffer[viewport.index(x, y)] = node.population as u8;
        } else {
            let mut draw_down = |dx: isize, dy: isize, n: Rc<Node>| {
                self.draw_to_cell(&mut buffer[..], n, &viewport, 2*x+dx, 2*y+dy);
            }; 
            let c = node.get_children();
            draw_down(0, 1, Rc::clone(&c.nw));
            draw_down(1, 1, Rc::clone(&c.ne));
            draw_down(0, 0, Rc::clone(&c.sw));
            draw_down(1, 0, Rc::clone(&c.se));
        }
    }

    fn as_vector(&self) -> Vec<Automata> {
        if let Some(top) = &self.top {
            top.as_array().into_iter().flatten().collect()
        } else {
            vec![]
        }
    }

    pub fn get_generation(&self) -> usize {
        self.gen
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

    fn as_array(&self) -> Vec<Vec<Automata>> {
        if self.level == 0 {
            return vec![vec![self.as_automata()]];
        }
        let children = self.get_children();
        let nw = children.nw.as_array();
        let ne = children.ne.as_array();
        let sw = children.sw.as_array();
        let se = children.se.as_array();
        let top = nw.into_iter()
            .zip(ne.into_iter())
            .map(|(left, right)| {
                let mut result = Vec::with_capacity(left.len() + right.len());
                result.extend(left);
                result.extend(right);
                result
            })
            .collect::<Vec<Vec<Automata>>>();
        let bottom = sw.into_iter()
            .zip(se.into_iter())
            .map(|(left, right)| {
                let mut result = Vec::with_capacity(left.len() + right.len());
                result.extend(left);
                result.extend(right);
                result
            })
            .collect::<Vec<Vec<Automata>>>();
        let mut rows = Vec::with_capacity(top.len() + bottom.len());
        rows.extend(top);
        rows.extend(bottom);
        rows
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
    fn get3() {
        let cell_width = 3;
        let cell_height = 3;
        let cells = vec![
            1,1,1,
            1,0,1,
            1,1,1,
        ];
        let hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        assert_eq!(hashlife.max_level(), 2);
        // Odd numbered widths and heights are shifted one to the right or up.
        for x in -2..2 {
            for y in -2..2 {
                if x == 0 && y == 0 { continue; }
                if !(x==-2 || y==-2) {
                    assert_eq!(hashlife.get(x, y), Some(Automata::Alive));
                } else {
                    assert_eq!(hashlife.get(x, y), Some(Automata::Dead));
                }
            }
        }
        assert_eq!(hashlife.get(0, 0), Some(Automata::Dead));
    }

    #[test]
    fn get4() {
        let cell_width = 4;
        let cell_height = 4;
        let cells = vec![
            1,1,1,1,
            1,1,0,1,
            1,1,1,1,
            1,1,1,1,
        ];
        let hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        assert_eq!(hashlife.max_level(), 2);
        for x in -2..2 {
            for y in -2..2 {
                if x == 0 && y == 0 { continue; }
                assert_eq!(hashlife.get(x, y), Some(Automata::Alive));
            }
        }
        assert_eq!(hashlife.get(0, 0), Some(Automata::Dead));
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
        let hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        assert_eq!(hashlife.max_level(), 3);
        println!("here");
        assert_eq!(hashlife.get(2, 3), Some(Automata::Dead));
        assert_eq!(hashlife.get(-2, 0), Some(Automata::Dead));
        assert_eq!(hashlife.get(-3, -3), Some(Automata::Dead));
        for x in -4..4 {
            for y in -4..4 {
                if x == 2 && y == 3 { continue; }
                if x == -2 && y == 0 { continue; }
                if x == -3 && y == -3 { continue; }
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
        let mut hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        assert_eq!(hashlife.max_level(), 3);

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
        assert_eq!(hashlife.get(-2, 2), Some(Automata::Alive));
        assert_eq!(hashlife.get(-1, 2), Some(Automata::Dead));
        assert_eq!(hashlife.get( 0, 2), Some(Automata::Dead));
        assert_eq!(hashlife.get( 1, 2), Some(Automata::Alive));
        // row 2
        assert_eq!(hashlife.get(-2, 1), Some(Automata::Alive));
        assert_eq!(hashlife.get(-1, 1), Some(Automata::Alive));
        assert_eq!(hashlife.get( 0, 1), Some(Automata::Dead));
        assert_eq!(hashlife.get( 1, 1), Some(Automata::Alive));
        // row 3
        assert_eq!(hashlife.get(-2, 0), Some(Automata::Alive));
        assert_eq!(hashlife.get(-1, 0), Some(Automata::Dead));
        assert_eq!(hashlife.get( 0, 0), Some(Automata::Alive));
        assert_eq!(hashlife.get( 1, 0), Some(Automata::Alive));
        // row 4
        assert_eq!(hashlife.get(-2,-1), Some(Automata::Alive));
        assert_eq!(hashlife.get(-1,-1), Some(Automata::Dead));
        assert_eq!(hashlife.get( 0,-1), Some(Automata::Dead));
        assert_eq!(hashlife.get( 1,-1), Some(Automata::Alive));
        // row 5
        assert_eq!(hashlife.get(-2,-2), Some(Automata::Dead));
        assert_eq!(hashlife.get(-1,-2), Some(Automata::Dead));
        assert_eq!(hashlife.get( 0,-2), Some(Automata::Alive));
        assert_eq!(hashlife.get( 1,-2), Some(Automata::Dead));
        // row 6
        assert_eq!(hashlife.get(-2,-3), Some(Automata::Dead));
        assert_eq!(hashlife.get(-1,-3), Some(Automata::Alive));
        assert_eq!(hashlife.get( 0,-3), Some(Automata::Dead));
        assert_eq!(hashlife.get( 1,-3), Some(Automata::Alive));
    }

    #[test]
    fn bounding_box_single_pt() {
        let bound = BoundingBox::from(0,0,0,0);
        assert_eq!(bound.width(), 1);
        assert_eq!(bound.height(), 1);
        assert_eq!(bound.top, 0);
        assert_eq!(bound.bottom, 0);
        assert_eq!(bound.left, 0);
        assert_eq!(bound.right, 0);
    }

    #[test]
    fn new_bounding_box_1() {
        let (x, y) = (0,0);
        let level = 1;
        let b = BoundingBox::new(x, y, level);
        assert_eq!(b.left, 0);
        assert_eq!(b.right, 1);
        assert_eq!(b.bottom, 0);
        assert_eq!(b.top, 1);
    }

    #[test]
    fn max_level_eq_0() {
        let cell_width = 1;
        let cell_height = 1;
        let cells = vec![ 1 ];
        let hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        assert_eq!(hashlife.max_level(), 0);
    }

    #[test]
    fn max_level_eq_1() {
        let cell_width = 2;
        let cell_height = 2;
        let cells = vec![ 1, 0, 1, 0 ];
        let hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        assert_eq!(hashlife.max_level(), 1);
    }

    #[test]
    fn cell3x2() {
        let cell_width = 3;
        let cell_height = 2;
        let cells = vec![ 1, 0, 1, 0, 0, 1 ];
        let hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        assert_eq!(hashlife.max_level(), 2);
    }

    #[test]
    /// Get the full universe as an array.
    /// Warning: This operation is rather slow.
    fn as_array_trunc_next_gen() {
        let cell_width = 4;
        let cell_height = 4;
        let cells = vec![
            0,0,0,0,
            0,1,1,1,
            0,0,0,0,
            0,0,0,0,
        ];
        let cells_next = vec![
            0,0,1,0,
            0,0,1,0,
            0,0,1,0,
            0,0,0,0,
        ];
        let mut hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        hashlife.next_generation();
        let result = hashlife.as_vector().into_iter().map(|a| a as u8).collect::<Vec<u8>>();
        assert_eq!(cells_next, result);
    }

    #[test]
    fn draw_diff_viewport_buffer() {
        let cell_width = 4;
        let cell_height = 6;
        let cells = vec![
            0,0,0,0,
            0,1,1,1,
            0,0,0,0,
            0,0,0,0,
            0,0,1,1,
            0,0,1,1
        ];
        let cells_next_expected = vec![
            0,0,1,0,
            0,0,0,0,
            0,0,1,0,
            0,0,0,0,
            0,0,0,0,
            0,0,0,0,
        ];

        let mut buffer = vec![
            0,0,0,0,
            0,0,0,0,
            0,0,0,0,
            0,0,0,0,
            0,0,0,0,
            0,0,0,0,
        ];
        let mut hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        hashlife.next_generation();
        let bound = BoundingBox::from(2, -3, -2, 1);
        hashlife.draw_diff_to_viewport_array(&mut buffer, bound);
        assert_eq!(cells_next_expected, buffer);
    }

    #[test]
    fn draw_to_viewport_buffer() {
        let cell_width = 4;
        let cell_height = 6;
        let cells = vec![
            0,0,0,0,
            0,1,1,1,
            0,0,0,0,
            0,0,0,0,
            0,0,1,1,
            0,0,1,1
        ];
        let expected = cells.clone();

        let mut buffer = vec![
            0,0,0,0,
            0,0,0,0,
            0,0,0,0,
            0,0,0,0,
            0,0,0,0,
            0,0,0,0,
        ];
        let mut hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        let bound = BoundingBox::from(2, -3, -2, 1);
        hashlife.draw_to_viewport_buffer(&mut buffer, bound);
        assert_eq!(expected, buffer);
    }


    #[test]
    fn empty_border() {
        let cell_width = 2;
        let cell_height = 2;
        let cells = vec![
            0,0,
            1,1,
        ];
        let cells_next = vec![
            0,0,0,0,
            0,0,0,0,
            0,1,1,0,
            0,0,0,0,
        ];
        let mut hashlife = Hashlife::from_array(cells, cell_width, cell_height, Edge::Truncate);
        let expanded = hashlife.expand_empty_border(Rc::clone(hashlife.top.as_ref().unwrap()));
        let viewport = BoundingBox::from(1, -2, -2, 1);
        hashlife.top = Some(expanded);
        for x in -2..2 {
            for y in -2..2 {
                let idx = viewport.index(x, y);
                let expected = cells_next[idx] as usize;
                let result = hashlife.get(x, y).unwrap() as usize;
                assert_eq!(expected, result,
                    "got {result} when expecting {expected} at abs({x},{y}) [index={idx}]",
                    expected=expected,
                    result=result,
                    x=x,
                    y=y,
                    idx=idx
                );
            }
        }
    }
}