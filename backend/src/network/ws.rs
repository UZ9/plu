use std::{net::SocketAddr, sync::{Arc}};

use axum::{extract::{ws::{Message, WebSocket}, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};

use futures::StreamExt;
use futures::SinkExt;
use tokio::sync::{broadcast::Sender, RwLock};

use crate::{api::grid_api::GridState, types::{ClientMessage, ServerMessage, TileState}, UpdateBroadcast, MAP_HEIGHT, MAP_WIDTH};

pub struct WebSocketHandler {
    path: String
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State((state, tx)): axum::extract::State<(Arc<RwLock<GridState>>, UpdateBroadcast)>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, tx))
}

impl WebSocketHandler {
    pub fn new(path: String) -> Self {
        WebSocketHandler {
            path
        }
    }

    pub async fn start_server(&self, addr: SocketAddr, state: Arc<RwLock<GridState>>, tx: Sender<Vec<TileState>>) {
        let app = Router::new().route(&self.path, get(ws_handler)).with_state((state, tx));

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        println!("listening on {addr}");

        axum::serve(listener, app).await.unwrap();
    }

    fn send_message(message: ServerMessage, state: Arc<RwLock<GridState>>, tx: Sender<Vec<TileState>>) {
        todo!()
    }

    fn on_receive_message(message: ClientMessage) {
        todo!()
    }

    fn get_router() -> axum::Router {
        todo!()
    }
}


async fn handle_socket(socket: WebSocket, state: Arc<RwLock<GridState>>, broadcast_tx: UpdateBroadcast) {

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
                        .tiles
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
                        grid.tiles.insert((col, row), data.clone());
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
