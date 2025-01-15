/******************************************************
 *                     1) Modules
 *****************************************************/
mod utils;

// External crates we need
extern crate fixedbitset;
extern crate js_sys;
extern crate web_sys;

use fixedbitset::FixedBitSet;
use js_sys::Math::random;
use std::fmt;
use wasm_bindgen::prelude::*;
use web_sys::console;

/******************************************************
 *            2) Timer / Debugging Helpers
 *****************************************************/
/// A simple RAII timer that uses `web_sys::console.time`/`time_end`.
/// Instantiated at the start of `tick()` to measure how long each `Universe::tick` takes.
pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}

/******************************************************
 *                 3) Cell and Universe
 *****************************************************/
/// Represents a single cell in Conway's Game of Life.
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

/// The primary struct for Conway’s Game of Life universe.
/// - Double-buffering with `current` + `next`
/// - `changed_cells` for delta-based rendering
#[wasm_bindgen]
pub struct Universe {
    width:  u32,
    height: u32,

    /// The "current" generation.
    current: FixedBitSet,

    /// The "next" generation (reused each tick).
    next:    FixedBitSet,

    /// Indices of cells that flipped state this tick.
    changed_cells: Vec<u32>,
}

/******************************************************
 *                 4) Universe - Public API
 *****************************************************/
#[wasm_bindgen]
impl Universe {
    /// Create a new Universe with a default size (64×64),
    /// partially randomized initial pattern.
    pub fn new() -> Universe {
        utils::set_panic_hook();

        let width  = 64;
        let height = 64;
        let size   = (width * height) as usize;

        // current generation
        let mut current = FixedBitSet::with_capacity(size);
        for i in 0..size {
            current.set(i, i % 2 == 0 || i % 7 == 0);
        }

        // next generation buffer
        let next = FixedBitSet::with_capacity(size);

        Universe {
            width,
            height,
            current,
            next,
            changed_cells: Vec::new(),
        }
    }

    /// Universe width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Adjust universe width, reallocate `current` as all-dead.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        let size = (width * self.height) as usize;

        let mut new_cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            new_cells.set(i, false);
        }

        self.current = new_cells;
    }

    /// Universe height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Adjust universe height, reallocate `current` as all-dead.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        let size    = (height * self.width) as usize;

        let mut new_cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            new_cells.set(i, false);
        }

        self.current = new_cells;
    }

    /// Return a pointer to the current generation's bits,
    /// used by JS to build a `Uint8Array`.
    pub fn cells(&self) -> *const usize {
        self.current.as_slice().as_ptr()
    }

    /// Return a string rendering of the current universe state,
    /// using Unicode squares.
    pub fn render(&self) -> String {
        self.to_string()
    }

    /// The main update step for the Game of Life:
    ///  - Start a `Timer` (for console profiling).
    ///  - Clear `changed_cells`.
    ///  - For each cell, calculate next gen in `self.next`.
    ///  - If a cell flips, push its index into `changed_cells`.
    ///  - Swap `current` and `next`.
    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");
        self.changed_cells.clear();

        for row in 0..self.height {
            for column in 0..self.width {
                let index        = self.get_index(row, column);
                let old_value    = self.current[index];
                let live_neighbors = self.live_neighbor_count(row, column);

                let new_value = match (old_value, live_neighbors) {
                    (true, x) if x < 2 => false,          // Underpopulation
                    (true, 2) | (true, 3) => true,        // Survive
                    (true, x) if x > 3 => false,          // Overpopulation
                    (false, 3) => true,                   // Reproduction
                    (otherwise, _) => otherwise,
                };

                self.next.set(index, new_value);

                // Track changed cell
                if new_value != old_value {
                    self.changed_cells.push(index as u32);
                }
            }
        }

        // Swap the buffers
        std::mem::swap(&mut self.current, &mut self.next);
    }

    /// Toggle a single cell from alive <-> dead.
    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let index = self.get_index(row, column);
        let current_val = self.current[index];
        self.current.set(index, !current_val);
    }

    /// Randomly set each cell to alive ~50% of the time.
    pub fn randomize(&mut self) {
        let size = (self.width * self.height) as usize;
        for i in 0..size {
            let alive = random() < 0.5;
            self.current.set(i, alive);
        }
    }

    /// Clear entire universe: set all cells dead,
    /// also record them in `changed_cells` so JS can redraw them.
    pub fn clear(&mut self) {
        self.changed_cells.clear();
        let size = (self.width * self.height) as usize;

        for i in 0..size {
            self.current.set(i, false);
            self.changed_cells.push(i as u32);
        }
    }

    /// Insert a glider pattern around (row, col).
    pub fn insert_glider_at(&mut self, row: u32, column: u32) {
        let glider_offsets = [
            (-1,  0),
            ( 0,  1),
            ( 1, -1),
            ( 1,  0),
            ( 1,  1),
        ];

        let mut positions = Vec::new();
        for (dr, dc) in glider_offsets {
            let new_r = ((row as i32 + dr).rem_euclid(self.height as i32)) as u32;
            let new_c = ((column as i32 + dc).rem_euclid(self.width  as i32)) as u32;
            positions.push((new_r, new_c));
        }

        self.set_cells(&positions);
    }

    /// Insert a pulsar pattern around (row, col).
    pub fn insert_pulsar_at(&mut self, row: u32, column: u32) {
        let pulsar_offsets = [
            // Row -4
            (-4, -2), (-4, -1), (-4, 1),  (-4, 2),
            // Row -2
            (-2, -4), (-2, -3), (-2, -2), (-2, -1),
            (-2,  1), (-2,  2), (-2,  3), (-2,  4),
            // Row  0
            ( 0, -4), ( 0, -3), ( 0,  3), ( 0,  4),
            // Row  2
            ( 2, -4), ( 2, -3), ( 2, -2), ( 2, -1),
            ( 2,  1), ( 2,  2), ( 2,  3), ( 2,  4),
            // Row  4
            ( 4, -2), ( 4, -1), ( 4,  1), ( 4,  2),
        ];

        let mut positions = Vec::new();
        for (dr, dc) in pulsar_offsets {
            let new_r = ((row as i32 + dr).rem_euclid(self.height as i32)) as u32;
            let new_c = ((column as i32 + dc).rem_euclid(self.width  as i32)) as u32;
            positions.push((new_r, new_c));
        }

        self.set_cells(&positions);
    }

    /// Return the pointer to `changed_cells` for JS to read.
    pub fn changed_cells_ptr(&self) -> *const u32 {
        self.changed_cells.as_ptr()
    }

    /// Return how many changed cells are in `changed_cells`.
    pub fn changed_cells_length(&self) -> usize {
        self.changed_cells.len()
    }
}

/******************************************************
 *         5) Universe - Private / Internal
 *****************************************************/
impl Universe {
    /// Get the fixed-bit index for (row, col).
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    /// Count how many of the 8 neighbors are alive.
    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 { self.height - 1 } else { row - 1 };
        let south = if row == self.height - 1 { 0 } else { row + 1 };
        let west  = if column == 0 { self.width - 1 } else { column - 1 };
        let east  = if column == self.width - 1 { 0 } else { column + 1 };

        // NW
        let nw = self.get_index(north, west);
        count += self.current[nw] as u8;
        // N
        let n = self.get_index(north, column);
        count += self.current[n] as u8;
        // NE
        let ne = self.get_index(north, east);
        count += self.current[ne] as u8;
        // W
        let w = self.get_index(row, west);
        count += self.current[w] as u8;
        // E
        let e = self.get_index(row, east);
        count += self.current[e] as u8;
        // SW
        let sw = self.get_index(south, west);
        count += self.current[sw] as u8;
        // S
        let s = self.get_index(south, column);
        count += self.current[s] as u8;
        // SE
        let se = self.get_index(south, east);
        count += self.current[se] as u8;

        count
    }

    /// Mark these `(row, col)` coordinates as alive in `current`.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for &(row, col) in cells {
            let idx = self.get_index(row, col);
            self.current.set(idx, true);
        }
    }

    /// Return the entire `current` bitset for debugging or advanced usage.
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.current
    }
}

/******************************************************
 *            6) fmt::Display Implementation
 *****************************************************/
impl fmt::Display for Universe {
    /// Renders the Universe as rows of Unicode squares:
    ///  ◼ for alive, ◻ for dead
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                let index = self.get_index(row, col);
                let alive = self.current[index];
                let symbol = if alive { '◼' } else { '◻' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
