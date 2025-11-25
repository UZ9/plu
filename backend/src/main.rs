use std::{collections::HashMap, sync::Arc, time::Duration};
use axum::{
    extract::{ws::{Message, WebSocket}, WebSocketUpgrade}, response::IntoResponse, routing::get, Router
};

use futures::StreamExt;
use futures::SinkExt;
use rand::seq::IndexedRandom;
use tokio::sync::{broadcast, RwLock};

use crate::types::{ClientMessage, HexData, ServerMessage, TerrainType, TileState};

const MAP_WIDTH: u32 = 20;
const MAP_HEIGHT: u32 = 40;

pub mod types;

pub type UpdateBroadcast = broadcast::Sender<Vec<TileState>>;
type GameState = Arc<RwLock<HashMap<(i32, i32), HexData>>>;

fn initialize_grid() -> HashMap<(i32, i32), HexData> {
    let mut grid = HashMap::new();

    let terrains = [ TerrainType::Wild, TerrainType::Mine ];

    for row in 0..MAP_WIDTH {
        for col in 0..MAP_HEIGHT {
            if let Some(terrain) = terrains.choose(&mut rand::rng()) {
                grid.insert((col as i32, row as i32), HexData {
                    terrain: terrain.clone(),
                });
            }
        }
    }

    grid
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State((state, tx)): axum::extract::State<(GameState, UpdateBroadcast)>,
) -> impl IntoResponse {
    println!("ws_handler");
    ws.on_upgrade(|socket| handle_socket(socket, state, tx))
}

async fn game_loop(state: GameState, tx: UpdateBroadcast) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        let mut updates = Vec::new();

        {
            let mut grid = state.write().await;

            let positions: Vec<_> = grid.keys().copied().collect();

            println!("sending update...");

            for pos in positions.iter().take(3) {
                if let Some(hex) = grid.get_mut(pos)
                    && hex.terrain == TerrainType::Wild {
                        hex.terrain = TerrainType::Mine;

                        updates.push(TileState {
                            col: pos.0,
                            row: pos.1,
                            data: hex.clone()
                        });
                }
            }
        }

        if !updates.is_empty() {
            let _ = tx.send(updates);
        }
    }

}

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(initialize_grid()));

    let (tx, _) = broadcast::channel::<Vec<TileState>>(100);

    let state_clone = state.clone();

    let tx_clone = tx.clone();

    tokio::spawn(async move {
        game_loop(state_clone, tx_clone).await;
    });

    let app = Router::new().route("/ws", get(ws_handler)).with_state((state, tx));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9001").await.unwrap();

    println!("listening on ws://localhost:9001");

    axum::serve(listener, app).await.unwrap();
}

async fn handle_socket(socket: WebSocket, state: GameState, broadcast_tx: UpdateBroadcast) {

    let (sender, mut receiver) = socket.split();

    let sender = Arc::new(tokio::sync::Mutex::new(sender));
    let sender_clone = sender.clone();

    let mut broadcast_rx = broadcast_tx.subscribe();

    // broadcast 
    let send_task = tokio::spawn(async move {
        while let Ok(updates) = broadcast_rx.recv().await {
            let mut sender = sender_clone.lock().await;

            for tile in updates {
                let response = ServerMessage::TileUpdate {
                    col: tile.col,
                    row: tile.row,
                    data: tile.data
                };

                if let Ok(json) = serde_json::to_string(&response)
                    && sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                }
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::RequestGridState) => {
                    println!("[REQUEST] request grid state");
                    let grid = state.read().await;
                    let tiles: Vec<TileState> = grid
                        .iter()
                        .map(|((col, row), data)| TileState {
                            col: *col,
                            row: *row,
                            data: data.clone(),
                        })
                    .collect();

                    let response = ServerMessage::GridState { tiles, width: MAP_WIDTH, height: MAP_HEIGHT };
                    if let Ok(json) = serde_json::to_string(&response) {
                        let mut sender = sender.lock().await;
                        let _ = sender.send(Message::Text(json.into())).await;
                    }
                }
                Ok(ClientMessage::TileUpdate { col, row, data }) => {
                    println!("[REQUEST] tile update");
                    {
                        let mut grid = state.write().await;
                        grid.insert((col, row), data.clone());
                    }

                    let update = vec![TileState { col, row, data }];
                    let _ = broadcast_tx.send(update);

                }
                Err(e) => {
                    eprintln!("Failed to parse message: {}", e);
                }
            }
        }
    }
    send_task.abort();
}
