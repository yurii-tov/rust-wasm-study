mod utils;

use std::{mem::swap, str::FromStr};

use rand::random;
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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

#[derive(Debug)]
struct Pattern {
    alive_cells: Vec<(u32, u32)>,
    width: u32,
    height: u32,
}

impl FromStr for Pattern {
    fn from_str(schema: &str) -> Result<Self, Self::Err> {
        let alive_cells: Vec<(u32, u32)> = schema
            .lines()
            .filter(|l| !l.starts_with("!"))
            .enumerate()
            .flat_map(|(i, l)| {
                l.char_indices()
                    .filter(|(_, c)| *c == 'O')
                    .map(move |(j, _)| (i as u32, j as u32))
            })
            .collect();
        if alive_cells.is_empty() {
            return Err(format!("No alive cells in given pattern: [{}]", schema));
        }
        let (width, height) = alive_cells.iter().fold((1, 1), |(xmax, ymax), (x, y)| {
            (xmax.max(*x + 1), ymax.max(*y + 1))
        });
        Ok(Self {
            alive_cells,
            width,
            height,
        })
    }

    type Err = String;
}

#[wasm_bindgen]
pub struct Universe {
    cells: Vec<Cell>,
    buffer: Vec<Cell>,
    diff: Vec<i32>,
    pub width: u32,
    pub height: u32,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 { self.height - 1 } else { row - 1 };

        let south = if row == self.height - 1 { 0 } else { row + 1 };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.cells[nw] as u8;

        let n = self.get_index(north, column);
        count += self.cells[n] as u8;

        let ne = self.get_index(north, east);
        count += self.cells[ne] as u8;

        let w = self.get_index(row, west);
        count += self.cells[w] as u8;

        let e = self.get_index(row, east);
        count += self.cells[e] as u8;

        let sw = self.get_index(south, west);
        count += self.cells[sw] as u8;

        let s = self.get_index(south, column);
        count += self.cells[s] as u8;

        let se = self.get_index(south, east);
        count += self.cells[se] as u8;

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
        self.diff.fill(-1);
        let mut diff_index: usize = 0;
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

                match (cell, next_cell) {
                    (a, b) if a != b => {
                        self.diff[diff_index] = idx as i32;
                        diff_index += 1;
                    }
                    _ => {}
                }

                self.buffer[idx] = next_cell;
            }
        }

        swap(&mut self.cells, &mut self.buffer);
    }

    pub fn diff(&self) -> *const i32 {
        self.diff.as_ptr()
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 120;
        let height = 120;

        Universe {
            width,
            height,
            cells: Self::random(width, height),
            buffer: vec![Cell::Dead; (width * height) as usize],
            diff: vec![-1; (width * height) as usize],
        }
    }

    pub fn randomize(&mut self) {
        self.cells = Self::random(self.width, self.height);
    }

    pub fn clear(&mut self) {
        for i in 0..self.width * self.height {
            self.cells[i as usize] = Cell::Dead;
        }
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

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    fn insert_pattern(&mut self, pattern: &Pattern, row: u32, column: u32) {
        let center = (pattern.width / 2, pattern.height / 2);
        let row = ((row - center.0) + self.width) % self.width;
        let column = ((column - center.1) + self.height) % self.height;
        for x in 0..pattern.width {
            for y in 0..pattern.height {
                let x = (row + x) % self.width;
                let y = (column + y) % self.height;
                let i = self.get_index(x, y);
                self.cells[i] = Cell::Dead;
            }
        }
        for (x, y) in &pattern.alive_cells {
            let x = (row + x) % self.width;
            let y = (column + y) % self.height;
            let i = self.get_index(x, y);
            self.cells[i] = Cell::Alive;
        }
    }

    pub fn insert_glider(&mut self, row: u32, column: u32) {
        let glider = "!Name: Glider
!Author: Richard K. Guy
!The smallest, most common, and first discovered spaceship.
!www.conwaylife.com/wiki/index.php?title=Glider
.O
..O
OOO"
        .parse()
        .unwrap();
        self.insert_pattern(&glider, row, column);
    }

    pub fn insert_pulsar(&mut self, row: u32, column: u32) {
        let pulsar: Pattern = "!Name: Pulsar
!Author: John Conway
!Despite its size, this is the fourth most common oscillator (and by far the most common of period greater than 2).
!www.conwaylife.com/wiki/index.php?title=Pulsar
..OOO...OOO

O....O.O....O
O....O.O....O
O....O.O....O
..OOO...OOO

..OOO...OOO
O....O.O....O
O....O.O....O
O....O.O....O

..OOO...OOO".parse().unwrap();
        self.insert_pattern(&pulsar, row, column);
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
