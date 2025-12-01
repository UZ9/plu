use std::{sync::Arc, time::Duration};

use log::debug;
use tokio::sync::{RwLock, broadcast};

use crate::{
    api::grid_api::GridState,
    network::ws::WebSocketServer,
    types::{HexTile, MineData, TileState},
};

const MAP_WIDTH: usize = 20;
const MAP_HEIGHT: usize = 40;
const STARTER_TILE: HexTile = HexTile::Wild;

const SERVER_URL: &str = "0.0.0.0:9001";

pub mod api;
pub mod network;
pub mod types;

pub type UpdateBroadcast = broadcast::Sender<Vec<TileState>>;

async fn game_loop(state: Arc<RwLock<GridState>>, tx: UpdateBroadcast) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        {
            debug!("sending update...");

            let tiles_to_modify = {
                let grid = state.read().await;

                grid.slime_tiles
                    .iter()
                    .flat_map(|t| {
                        let (x, y) = grid.get_coords(*t);

                        grid.get_neighbors(x, y)
                            .map(|n| grid.get_coords(n))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            };

            {
                let mut grid = state.write().await;

                for (nx, ny) in tiles_to_modify {
                    let tile = grid.get_tile(nx, ny).unwrap();

                    if !matches!(tile, HexTile::Mine(_)) {
                        grid.set_tile(
                            nx,
                            ny,
                            HexTile::Mine(MineData {
                                count: 1,
                                level: 1,
                                capacity: 1,
                                trade_value: 1,
                                state: "".to_string(),
                            }),
                        );
                    }
                }
            }

            // 3 updates to happen (maybe not in this order)
            // 1. miner update
            // 2. logistics update
            // 3. defense update
            // this will be handled via wasm, but for now it'll be implementations of the
            // rust traits
        }

        // if !updates.is_empty() {
        //     let _ = tx.send(updates);
        // }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let state = Arc::new(RwLock::new(GridState::new(
        MAP_WIDTH,
        MAP_HEIGHT,
        STARTER_TILE,
    )));

    let (tx, _) = broadcast::channel::<Vec<TileState>>(100);

    let state_clone = state.clone();

    let tx_clone = tx.clone();

    tokio::spawn(async move {
        game_loop(state_clone, tx_clone).await;
    });

    let ws_handler = WebSocketServer::new("/ws".to_string());
    ws_handler
        .start_server(SERVER_URL.parse().unwrap(), state, tx)
        .await;
}
