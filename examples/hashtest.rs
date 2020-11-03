use hashlife::hashlife::{Cell, HashLife, Tree, QuadNode};
use hashlife::{EdgeRule, GameOfLife};

use std::rc::Rc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}


fn main() {
    let mut hl = HashLife::new(hashlife::EdgeRule::Wrap(100, 100));
    println!("start");
    let top = hl.random_tree(10);
    println!("finish");
    let next = hl.next_generation(top);
    println!("next: {:?}", next.as_ref().force_tree());

}