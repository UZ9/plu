use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};

use futures::SinkExt;
use futures::StreamExt;
use log::{debug, info};
use tokio::sync::RwLock;

use crate::{
    UpdateBroadcast,
    api::grid_api::GridState,
    types::{ClientMessage, HexTile, ServerMessage, TileState},
};

pub struct WebSocketServer {
    path: String,
}

impl WebSocketServer {
    pub fn new(path: String) -> Self {
        WebSocketServer { path }
    }

    pub async fn start_server(
        &self,
        addr: SocketAddr,
        state: Arc<RwLock<GridState>>,
        tx: UpdateBroadcast,
    ) {
        let app = Router::new()
            .route(&self.path, get(ws_handler))
            .with_state((state, tx));

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        info!("websocket server listening on {addr}");

        axum::serve(listener, app).await.unwrap();
    }
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<RwLock<GridState>>,
    broadcast_tx: UpdateBroadcast,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = broadcast_tx.subscribe();

    debug!("New socket connection formed");

    loop {
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
}

async fn on_receive_message(
    state: &Arc<RwLock<GridState>>,
    tx: &UpdateBroadcast,
    message: ClientMessage,
) -> Option<ServerMessage> {
    match message {
        ClientMessage::RequestGridState => {
            debug!("[REQUEST] request grid state");

            let grid = state.read().await;
            let tiles: &Vec<HexTile> = &grid.tiles;

            let mut update = Vec::new();

            (0..grid.width).for_each(|i| {
                for j in 0..grid.height {
                    update.push(TileState {
                        data: tiles[grid.get_index(i as u32, j as u32)].clone(),
                        row: j as i32,
                        col: i as i32,
                    });
                }
            });

            Some(ServerMessage::GridState {
                tiles: update,
                width: grid.width,
                height: grid.height,
            })
        }
        ClientMessage::TileUpdate { col, row, data } => {
            debug!("[REQUEST] tile update for <col: {col}, row: {row}>");
            {
                let mut grid = state.write().await;

                grid.set_tile(col as u32, row as u32, data.clone())
            }

            // acknowledged on backend, now update client
            Some(ServerMessage::TileUpdate { row, col, data })
        }
        _ => None,
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State((state, tx)): axum::extract::State<(
        Arc<RwLock<GridState>>,
        UpdateBroadcast,
    )>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, tx))
}

#[cfg(test)]
mod tests {
    use std::cmp::{max, min};

    use tokio::sync::broadcast;

    use super::*;

    #[test]
    fn it_sets_websocket_path_on_initialization() {
        let path = "ws://somesocketaddr";

        let ws = WebSocketServer::new(path.to_string());

        assert_eq!(ws.path, path);
    }

    #[tokio::test]
    async fn it_returns_nothing_if_invalid_received_message() {
        let state: Arc<RwLock<GridState>> =
            Arc::new(RwLock::new(GridState::new(10, 10, HexTile::Wild)));

        let (tx, _) = broadcast::channel::<Vec<TileState>>(100);

        let response = on_receive_message(&state, &tx, ClientMessage::None).await;

        assert!(response.is_none());
    }

    #[tokio::test]
    async fn it_returns_tile_update_for_tile_request() {
        let state: Arc<RwLock<GridState>> =
            Arc::new(RwLock::new(GridState::new(10, 10, HexTile::Wild)));

        let (tx, _) = broadcast::channel::<Vec<TileState>>(100);

        let update = ClientMessage::TileUpdate {
            col: 1,
            row: 0,
            data: HexTile::Slime,
        };
        let expected_response = ServerMessage::TileUpdate {
            col: 1,
            row: 0,
            data: HexTile::Slime,
        };

        let response = on_receive_message(&state, &tx, update).await;

        assert!(response.is_some());

        let response = response.unwrap();

        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn it_returns_grid_state_for_request() {
        let state: Arc<RwLock<GridState>> =
            Arc::new(RwLock::new(GridState::new(15, 10, HexTile::Wild)));

        let grid = state.read().await;

        let (tx, _) = broadcast::channel::<Vec<TileState>>(100);

        let update = ClientMessage::RequestGridState;

        let response = on_receive_message(&state, &tx, update).await;

        assert!(response.is_some());

        let response = response.unwrap();

        match response {
            ServerMessage::GridState {
                width,
                height,
                tiles,
            } => {
                assert_eq!(width, 15);
                assert_eq!(height, 10);
                assert_eq!(tiles.len(), grid.tiles.len());
                assert_eq!(tiles.len(), 15 * 10);

                let mut min_x = 999999;
                let mut max_x = -999999;
                let mut min_y = 999999;
                let mut max_y = -999999;

                tiles.iter().for_each(|tile| {
                    min_x = min(tile.col, min_x);
                    max_x = max(tile.col, max_x);

                    min_y = min(tile.row, min_y);
                    max_y = max(tile.row, max_y);

                    assert_eq!(tile.data.clone(), HexTile::Wild);
                });

                assert_eq!(min_x, 0);
                assert_eq!(min_y, 0);
                assert_eq!(max_x, grid.width as i32 - 1);
                assert_eq!(max_y, grid.height as i32 - 1);
            }
            _ => {
                panic!("invalid response")
            }
        }
    }
}
