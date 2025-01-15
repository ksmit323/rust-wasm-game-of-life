mod utils;

extern crate fixedbitset;
extern crate js_sys;
extern crate web_sys;

use fixedbitset::FixedBitSet;
use js_sys::Math::random;
use std::fmt;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    current: FixedBitSet,
    next: FixedBitSet,
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        utils::set_panic_hook();

        let width = 64;
        let height = 64;
        let size = (width * height) as usize;
        
        let mut current = FixedBitSet::with_capacity(size);
        for i in 0..size {
            current.set(i, i % 2 == 0 || i % 7 == 0);
        }
        
        let next = FixedBitSet::with_capacity(size);

        Universe {
            width,
            height,
            current,
            next,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        let size = (width * self.height) as usize;

        // Allocate new bit set of the right size
        let mut new_cells = FixedBitSet::with_capacity(size);

        // Fill everything with dead (false)
        for i in 0..size {
            new_cells.set(i, false);
        }

        self.current = new_cells;
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        let size = (height * self.width) as usize;
        let mut new_cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            new_cells.set(i, false);
        }

        self.current = new_cells;
    }

    pub fn cells(&self) -> *const usize {
        self.current.as_slice().as_ptr()
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn tick(&mut self) {  
        let _timer = Timer::new("Universe::tick");
        
        for row in 0..self.height {
            for column in 0..self.width {
                let index = self.get_index(row, column);
                let cell = self.current[index];
                let live_neighbors = self.live_neighbor_count(row, column);
                
                let next_val = match (cell, live_neighbors) {
                        (true, x) if x < 2 => false,
                        (true, 2) | (true, 3) => true,
                        (true, x) if x > 3 => false,
                        (false, 3) => true,
                        (otherwise, _) => otherwise,
                };
                
                self.next.set(index, next_val);
            }
        }
        std::mem::swap(&mut self.current, &mut self.next);
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
    
        let north = if row == 0 {
            self.height - 1
        } else {
            row - 1
        };
    
        let south = if row == self.height - 1 {
            0
        } else {
            row + 1
        };
    
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
        count += self.current[nw] as u8;
    
        let n = self.get_index(north, column);
        count += self.current[n] as u8;
    
        let ne = self.get_index(north, east);
        count += self.current[ne] as u8;
    
        let w = self.get_index(row, west);
        count += self.current[w] as u8;
    
        let e = self.get_index(row, east);
        count += self.current[e] as u8;
    
        let sw = self.get_index(south, west);
        count += self.current[sw] as u8;
    
        let s = self.get_index(south, column);
        count += self.current[s] as u8;
    
        let se = self.get_index(south, east);
        count += self.current[se] as u8;
    
        count
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let index = self.get_index(row, column);
        let current = self.current[index];
        self.current.set(index, !current);
    }

    pub fn randomize(&mut self) {
        let size = (self.width * self.height) as usize;
        for i in 0..size {
            let alive = random() < 0.5;
            self.current.set(i, alive);
        }
    }

    pub fn clear(&mut self) {
        let size = (self.width * self.height) as usize;
        for i in 0..size {
            self.current.set(i, false);
        }
    }

    pub fn insert_glider_at(&mut self, row: u32, column: u32) {
        let glider_offsets = [
            (-1i32, 0i32),
            (0i32, 1i32),
            (1i32, -1i32),
            (1i32, 0i32),
            (1i32, 1i32),
        ];

        let mut positions = Vec::new();
        for (delta_row, delta_column) in glider_offsets {
            let new_r = ((row as i32 + delta_row).rem_euclid(self.height as i32)) as u32;
            let new_c = ((column as i32 + delta_column).rem_euclid(self.width as i32)) as u32;
            positions.push((new_r, new_c));
        }

        self.set_cells(&positions);
    }

    pub fn insert_pulsar_at(&mut self, row: u32, column: u32) {
        let pulsar_offsets = [
            // Row -4
            (-4, -2),
            (-4, -1),
            (-4, 1),
            (-4, 2),
            // Row -2
            (-2, -4),
            (-2, -3),
            (-2, -2),
            (-2, -1),
            (-2, 1),
            (-2, 2),
            (-2, 3),
            (-2, 4),
            // Row  0
            (0, -4),
            (0, -3),
            (0, 3),
            (0, 4),
            // Row  2
            (2, -4),
            (2, -3),
            (2, -2),
            (2, -1),
            (2, 1),
            (2, 2),
            (2, 3),
            (2, 4),
            // Row  4
            (4, -2),
            (4, -1),
            (4, 1),
            (4, 2),
        ];

        let mut positions = Vec::new();
        for (delta_row, delta_column) in pulsar_offsets {
            let new_r = ((row as i32 + delta_row).rem_euclid(self.height as i32)) as u32;
            let new_c = ((column as i32 + delta_column).rem_euclid(self.width as i32)) as u32;
            positions.push((new_r, new_c));
        }

        self.set_cells(&positions);
    }
}

impl fmt::Display for Universe {
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

impl Universe {
    /// Get the dead and alive values for the whole universe
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.current
    }

    /// Set cells to be alive by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().copied() {
            let idx = self.get_index(row, col);
            self.current.set(idx, true);
        }
    }
}

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
