use std::{collections::HashMap, fmt::Display};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
pub struct TurretData {
    level: u32,
    // in the future it could be neat to require ammo
    state: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
pub struct MineData {
    level: u32,
    count: u32, // how much gold i currently have
    capacity: u32, // how much gold i can have at max (might be dynamic in future)
    state: String, // we'll have a string that you can arbitarily put data into for "persistence"
    trade_value: u32 // bad name for this, but >0 is me offering gold to the networking, <0 is me
                     // requesting gold from the network
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
pub enum TerrainType {
    Wild,
    Mine(MineData),
    Turret(TurretData),
    Slime
}

impl Display for TerrainType {
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
pub struct HexData {
    pub terrain: TerrainType,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
pub struct TileState {
    pub col: i32,
    pub row: i32,
    pub data: HexData
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "request_grid_state")]
    RequestGridState,
    #[serde(rename = "tile_update")]
    TileUpdate { col: i32, row: i32, data: HexData }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "grid_state")]
    GridState { width: u32, height: u32, tiles: Vec<TileState> },
    #[serde(rename = "tile_update")]
    TileUpdate { col: i32, row: i32, data: HexData }
}
