mod utils;

use rand::random;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

#[wasm_bindgen]
pub struct Universe {
    cells: Vec<Cell>,
    pub width: u32,
    pub height: u32,
    pub generation: u32,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }
    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for x in [self.height - 1, 0, 1] {
            for y in [self.width - 1, 0, 1] {
                if (x, y) != (0, 0) {
                    let neighbor_row = (row + x) % self.height;
                    let neighbor_col = (column + y) % self.width;
                    let idx = self.get_index(neighbor_row, neighbor_col);
                    count += self.cells[idx] as u8;
                }
            }
        }
        count
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);
                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
        self.generation += 1;
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 64;
        let height = 64;

        Universe {
            width,
            height,
            cells: Self::random(width, height),
            generation: 1,
        }
    }

    pub fn randomize(&mut self) {
        self.cells = Self::random(self.width, self.height);
        self.generation = 1;
    }

    pub fn clear(&mut self) {
        for i in 0..self.width * self.height {
            self.cells[i as usize] = Cell::Dead;
        }
        self.generation = 1;
    }

    /// Generate random cells pattern
    fn random(w: u32, h: u32) -> Vec<Cell> {
        (0..(w * h))
            .map(|_| if random() { Cell::Dead } else { Cell::Alive })
            .collect()
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    /// Get number of living cells
    pub fn living(&self) -> u32 {
        self.cells.iter().filter(|x| **x == Cell::Alive).count() as u32
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height).map(|_i| Cell::Dead).collect();
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height).map(|_i| Cell::Dead).collect();
    }
}
