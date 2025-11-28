
use std::{net::SocketAddr, sync::Arc};

use axum::{extract::{ws::{Message, WebSocket}, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};

use futures::StreamExt;
use futures::SinkExt;
use log::{debug, info};
use tokio::sync::RwLock;

use crate::{api::grid_api::GridState, types::{ClientMessage, ServerMessage, TileState}, UpdateBroadcast, MAP_HEIGHT, MAP_WIDTH};

pub struct WebSocketServer {
    path: String
}

impl WebSocketServer {
    pub fn new(path: String) -> Self {
        WebSocketServer {
            path
        }
    }

    pub async fn start_server(&self, addr: SocketAddr, state: Arc<RwLock<GridState>>, tx: UpdateBroadcast) {
        let app = Router::new().route(&self.path, get(ws_handler)).with_state((state, tx));

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        info!("websocket server listening on {addr}");

        axum::serve(listener, app).await.unwrap();
    }
}

async fn handle_socket(socket: WebSocket, state: Arc<RwLock<GridState>>, broadcast_tx: UpdateBroadcast) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = broadcast_tx.subscribe();

    debug!("New socket connection formed");

    // select! documentation (wild):
    // https://tokio.rs/tokio/tutorial/select
    tokio::select! {
        Ok(updates) = broadcast_rx.recv() => {
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
        },
        Some(Ok(msg)) = receiver.next() => {
            if let Message::Text(text) = msg {
                let message = serde_json::from_str::<ClientMessage>(&text);

                if let Ok(message) = message {
                    // successfully received client message
                    let response = on_receive_message(&state, &broadcast_tx, message).await;

                    // potential server message
                    if let Some(response) = response
                        && let Ok(json) = serde_json::to_string(&response) {
                            let _ = sender.send(Message::Text(json.into())).await;
                        }
                }

            }
        }
    }
}

async fn on_receive_message(state: &Arc<RwLock<GridState>>, tx: &UpdateBroadcast, message: ClientMessage) -> Option<ServerMessage> {
    match message {
        ClientMessage::RequestGridState => {
            debug!("[REQUEST] request grid state");

            let grid = state.read().await;
            let tiles: Vec<TileState> = grid
                .tiles
                .iter()
                .map(|((col, row), data)| TileState {
                    col: *col,
                    row: *row,
                    data: data.clone(),
                })
            .collect();

            Some(ServerMessage::GridState { tiles, width: MAP_WIDTH, height: MAP_HEIGHT })
        }
        ClientMessage::TileUpdate { col, row, data } => {
            debug!("[REQUEST] tile update");
            {
                let mut grid = state.write().await;
                grid.tiles.insert((col, row), data.clone());
            }

            let update = vec![TileState { col, row, data }];
            let _ = tx.send(update);

            None
        }
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State((state, tx)): axum::extract::State<(Arc<RwLock<GridState>>, UpdateBroadcast)>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, tx))
}

