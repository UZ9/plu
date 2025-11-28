use std::collections::HashMap;

use crate::types::{HexData, TerrainType};

pub struct GridState {
    pub width: u32,
    pub height: u32,
    pub tiles: HashMap<(i32, i32), HexData>
}

impl GridState {
    pub fn new(width: u32, height: u32, starter_tile: TerrainType) -> Self {
        let mut grid = HashMap::new();

        for row in 0..width {
            for col in 0..height {
                grid.insert((col as i32, row as i32), HexData {
                    terrain: starter_tile.clone(),
                });
            }
        }

        GridState {
            width,
            height,
            tiles: grid
        }
    }
}
