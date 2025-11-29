use std::{collections::HashMap, fmt::Display};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
pub struct TurretData {
    pub level: u32,
    // in the future it could be neat to require ammo
    pub state: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
pub struct MineData {
    pub level: u32,
    pub count: u32, // how much gold i currently have
    pub capacity: u32, // how much gold i can have at max (might be dynamic in future)
    pub state: String, // we'll have a string that you can arbitarily put data into for "persistence"
    pub trade_value: u32 // bad name for this, but >0 is me offering gold to the networking, <0 is me
                     // requesting gold from the network
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
pub enum HexTile {
    Wild,
    Mine(MineData),
    Turret(TurretData),
    Slime
}

impl Display for HexTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wild => write!(f, "Wild"),
            Self::Mine(_) => write!(f, "Mine"),
            Self::Turret(_) => write!(f, "Turret"),
            Self::Slime => write!(f, "Slime"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
pub struct TileState {
    pub col: i32,
    pub row: i32,
    pub data: HexTile
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "request_grid_state")]
    RequestGridState,
    #[serde(rename = "tile_update")]
    TileUpdate { col: i32, row: i32, data: HexTile }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "grid_state")]
    GridState { width: usize, height: usize, tiles: Vec<TileState> },
    #[serde(rename = "tile_update")]
    TileUpdate { col: i32, row: i32, data: HexTile }
}
