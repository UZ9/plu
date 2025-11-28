use std::{sync::Arc, time::Duration};

use tokio::sync::{broadcast, RwLock};

use crate::{api::grid_api::GridState, network::ws::WebSocketServer, types::{HexData, TerrainType, TileState}};

const MAP_WIDTH: u32 = 20;
const MAP_HEIGHT: u32 = 40;
const STARTER_TILE: TerrainType = TerrainType::Wild;

const SERVER_URL: &str = "0.0.0.0:9001";

pub mod types;
pub mod api;
pub mod network;

pub type UpdateBroadcast = broadcast::Sender<Vec<TileState>>;

async fn game_loop(state: Arc<RwLock<GridState>>, tx: UpdateBroadcast) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        let mut updates = Vec::new();

        {
            let mut grid = state.write().await;

            let positions: Vec<_> = grid.tiles.keys().copied().collect();

            println!("sending update...");

            // 3 updates to happen (maybe not in this order)
            // 1. miner update 
            // 2. logistics update
            // 3. defense update 
            // this will be handled via wasm, but for now it'll be implementations of the 
            // rust traits 
        }

        if !updates.is_empty() {
            let _ = tx.send(updates);
        }
    }

}

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(GridState::new(
                MAP_WIDTH,
                MAP_HEIGHT,
                STARTER_TILE
    )));

    let (tx, _) = broadcast::channel::<Vec<TileState>>(100);

    let state_clone = state.clone();

    let tx_clone = tx.clone();

    tokio::spawn(async move {
        game_loop(state_clone, tx_clone).await;
    });

    let ws_handler = WebSocketServer::new("/ws".to_string());
    ws_handler.start_server(SERVER_URL.parse().unwrap(), state, tx).await;
}
