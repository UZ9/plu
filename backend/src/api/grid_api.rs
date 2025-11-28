use crate::types::{HexData, TerrainType};

pub struct GridState {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<HexData>
}

impl GridState {
    pub fn new(width: usize, height: usize, starter_tile: TerrainType) -> Self {
        let tiles = vec![HexData { terrain: starter_tile }; width * height];

        GridState {
            width,
            height,
            tiles
        }
    }

    pub fn get_index(&self, x: u32, y: u32) -> usize {
        (y as usize * self.width) + x as usize
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<&HexData> {
        self.tiles.get(self.get_index(x, y))
    }

    pub fn set_tile(&mut self, x: u32, y: u32, new_tile: HexData) {
        let index = self.get_index(x, y);

        let tile = self.tiles.get_mut(index);

        if let Some(tile) = tile {
            *tile = new_tile;
        }
    }
}
