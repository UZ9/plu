use std::fmt::Display;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/types.ts")]
pub enum TerrainType {
    Wild,
    Mine,
    Turret,
    Slime
}

impl Display for TerrainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wild => write!(f, "Wild"),
            Self::Mine => write!(f, "Mine"),
            Self::Turret => write!(f, "Turret"),
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
