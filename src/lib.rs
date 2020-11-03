pub mod linear;
pub mod hashlife;
pub mod cell;

pub enum EdgeRule {
    Wrap (usize, usize), // width, height
    Truncate(usize, usize),  // width, height
    Infinite,
    InfiniteWithRandom(usize), // size
}

/// The Game of Life trait specifies
pub trait GameOfLife {
    fn new(edge: EdgeRule) -> Self;
    fn from_rle(edge: EdgeRule, content: String) -> Self;
    fn tick(&mut self);
    fn tickn(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.tick();
        }
    }
    fn get_generation(&self) -> usize;
    fn draw_with(&self, f: &mut dyn FnMut (&Self) -> ()) { f(self); }
}
