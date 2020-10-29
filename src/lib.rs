use getrandom;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

pub struct HashLife {
    generation: u64,
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl HashLife {
    pub fn new(width: usize, height: usize) -> Self {
        let mut randoms = vec![0u8; (width * height) as usize];
        getrandom::getrandom(&mut randoms[..]).expect("random num generation failed.");
        let cells = randoms.iter().map(|r| if r%2==0 {Cell::Dead} else {Cell::Alive}).collect();
        Self {
            generation: 0,
            width,
            height,
            cells,
        }
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.living_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }
        self.cells = next;
        self.generation += 1;
    }

    fn get_index(&self, row: usize, column: usize) -> usize {
        (row * self.width + column) as usize
    }

    fn living_neighbor_count(&self, row: usize, col: usize) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_col == 0 && delta_row == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (col + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cells(&self) -> Vec<Cell> {
        self.cells.clone()
    }
}