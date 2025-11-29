use log::debug;

use crate::types::HexTile;

pub struct GridState {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<HexTile>,

    // TODO: for performance reasons these lists are more or less hardcoded 
    // to ensure priority (e.g. mines running before slime); ideally I'd like 
    // some more ideomatic rust way to handle this
    pub slime_tiles: Vec<usize>,
    pub mine_tiles: Vec<usize>,
    pub turret_tiles: Vec<usize>
}

impl GridState {
    pub fn new(width: usize, height: usize, starter_tile: HexTile) -> Self {
        let tiles = vec![starter_tile; width * height];

        debug!("Creating tile map of size {width}x{height}");

        GridState {
            width,
            height,
            tiles,
            slime_tiles: Vec::new(),
            mine_tiles: Vec::new(),
            turret_tiles: Vec::new(),
        }
    }

    pub fn get_index(&self, x: u32, y: u32) -> usize {
        (y as usize * self.width) + x as usize
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<&HexTile> {
        self.tiles.get(self.get_index(x, y))
    }

    pub fn set_tile(&mut self, x: u32, y: u32, new_tile: HexTile) {
        let index = self.get_index(x, y);
        debug!("updating <{x}, {y}> to {new_tile}, giving it an index of <{index}>");

        let tile = self.tiles[index].clone();

        // for priority, we keep a list for each "active" tile type 
        // this requires unregistering/registering to keep it synchronized 
        // with the main tile array
        self.unregister_tile(index, &tile);
        self.tiles[index] = new_tile.clone();
        self.register_tile(index, &new_tile);

        
        debug!("mines: {:?}, slimes: {:?}", self.mine_tiles, self.slime_tiles);
    }

    fn unregister_tile(&mut self, i: usize, t: &HexTile) {
        match t {
            HexTile::Mine(_) => {
                self.mine_tiles.retain(|j| *j != i);
            },
            HexTile::Slime => { 
                self.slime_tiles.retain(|j| *j != i);
            }
            _ => {}
        }
    }

    fn register_tile(&mut self, i: usize, t: &HexTile) {
        match t {
            HexTile::Mine(_) => {
                self.mine_tiles.push(i);
            },
            HexTile::Slime => { 
                self.slime_tiles.push(i);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::MineData;

    use super::*;

    #[test]
    fn it_initializes_empty_arrays() {
        let start_tile = HexTile::Slime;

        let grid_state = GridState::new(5, 5, start_tile);

        assert_eq!(grid_state.width, 5);
        assert_eq!(grid_state.height, 5);
        assert_eq!(grid_state.slime_tiles.len(), 0);
        assert_eq!(grid_state.mine_tiles.len(), 0);
        assert_eq!(grid_state.turret_tiles.len(), 0);
    }

    #[test]
    fn it_gets_correct_index() {
        let start_tile = HexTile::Slime;

        let grid_state = GridState::new(72, 30, start_tile);

        assert_eq!(grid_state.width, 72);
        assert_eq!(grid_state.height, 30);

        // 12x8, so:
        // (1, 0) should be 1
        // (0, 1) should be width
        // (13, 18) should be 18 * width + 13 (18th row, each row has width pixels)
        assert_eq!(grid_state.get_index(1, 0), 1);
        assert_eq!(grid_state.get_index(0, 1), grid_state.width);
        assert_eq!(grid_state.get_index(13, 18), 18 * grid_state.width + 13);
    }

    #[test]
    fn it_sets_the_default_tile() {
        let start_tile = HexTile::Slime;

        let grid_state = GridState::new(72, 30, start_tile.clone());

        assert_eq!(grid_state.width, 72);
        assert_eq!(grid_state.height, 30);

        // 12x8, so:
        // (1, 0) should be 1
        // (0, 1) should be width
        // (13, 18) should be 18 * width + 13 (18th row, each row has width pixels)
        let tile = grid_state.get_tile(1, 0);

        assert!(tile.is_some());
        assert_eq!(tile.unwrap().clone(), start_tile);
    }

    #[test]
    fn it_can_set_slime_tile() {
        let start_tile = HexTile::Slime;

        let mut grid_state = GridState::new(72, 30, start_tile.clone());

        assert_eq!(grid_state.width, 72);
        assert_eq!(grid_state.height, 30);

        // 12x8, so:
        // (1, 0) should be 1
        // (0, 1) should be width
        // (13, 18) should be 18 * width + 13 (18th row, each row has width pixels)
        let tile = grid_state.get_tile(1, 0);

        assert!(tile.is_some());
        assert_eq!(tile.unwrap().clone(), start_tile);

        // now we change it 
        grid_state.set_tile(1, 0, HexTile::Slime);

        assert_eq!(grid_state.get_tile(1, 0).unwrap().clone(), HexTile::Slime);

        // should've also been registered
        assert!(grid_state.slime_tiles.contains(&grid_state.get_index(1, 0)));

        // now we change it back to wild 
        grid_state.set_tile(1, 0, HexTile::Wild);

        // should be unregistered
        assert!(!grid_state.slime_tiles.contains(&grid_state.get_index(1, 0)));
    }

    #[test]
    fn it_can_set_mine_tile() {
        let start_tile = HexTile::Slime;

        let mut grid_state = GridState::new(72, 30, start_tile.clone());

        assert_eq!(grid_state.width, 72);
        assert_eq!(grid_state.height, 30);

        // 12x8, so:
        // (1, 0) should be 1
        // (0, 1) should be width
        // (13, 18) should be 18 * width + 13 (18th row, each row has width pixels)
        let tile = grid_state.get_tile(1, 0);

        assert!(tile.is_some());
        assert_eq!(tile.unwrap().clone(), start_tile);

        let new_tile = HexTile::Mine(MineData {
            level: 1,
            count: 1,
            capacity: 3,
            state: "state".to_string(),
            trade_value: 0
        });

        // now we change it 
        grid_state.set_tile(1, 0, new_tile.clone());

        assert_eq!(grid_state.get_tile(1, 0).unwrap().clone(), new_tile);

        // should've also been registered
        assert!(grid_state.mine_tiles.contains(&grid_state.get_index(1, 0)));

        // now we change it back to wild 
        grid_state.set_tile(1, 0, HexTile::Wild);

        // should be unregistered
        assert!(!grid_state.mine_tiles.contains(&grid_state.get_index(1, 0)));
    }

    // test for invalid range tile

}
