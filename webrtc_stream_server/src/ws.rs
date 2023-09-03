use crate::AppState;

use axum::{extract::{ws::{WebSocket, Message}, WebSocketUpgrade, Path, State}, response::Response};
use tokio::sync::mpsc;
use futures_util::{StreamExt, SinkExt};
use tracing::{error, info, warn};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageData {
    pub msg_type: String,
    pub sender: String,
    pub receiver: String,
    pub msg: String,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>, Path(uuid): Path<String>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state, uuid))
}

async fn handle_socket(socket: WebSocket, state: AppState, uuid: String) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(10);

    {
        let mut sess = state.txs.lock().await;
        if !sess.contains_key(&uuid) {
            info!("New session: {}", &uuid);
            sess.insert(uuid.clone(), tx);
        } else {
            info!("Session resumed: {}", &uuid);
        }
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                error!("Error sending message to client");
                break;
            }
        }
    });

    // This task will receive messages from client and send them to broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            let mut sess = state.txs.lock().await;
            info!("Received message: {:?}", &msg);
            match msg {
                Message::Close(_) => {
                    sess.remove(&uuid);
                    break;
                },
                Message::Text(msg) => {
                    if let Ok(data) = serde_json::from_str::<MessageData>(&msg) {
                        if data.msg_type == "heartbeat" {
                            info!("Heartbeat received: {:?}", &uuid);
                        } else {
                            match sess.get(&data.receiver) {
                                Some(v) => {
                                    v.send(msg).await.unwrap();
                                }
                                None => { warn!("Receiver not found: {:?}", &data.receiver); }
                            }
                        }
                    } else {
                        error!("Invalid message: {:?}", &msg);
                    }
                }
                _ => { }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}