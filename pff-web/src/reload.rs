use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
};
use log::trace;
use tokio::sync::broadcast;

/// Maximum number of browsers that can be open.
const MAX_INSTANCES: usize = 16;

#[derive(Debug, Clone)]
pub struct AutoReload {
    tx: broadcast::Sender<()>,
}

impl AutoReload {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(MAX_INSTANCES);
        AutoReload { tx }
    }

    pub fn notify(&self) {
        let _ = self.tx.send(());
    }
}

pub async fn reload_req(ws: WebSocketUpgrade, auto_reload: AutoReload) -> impl IntoResponse {
    trace!("Reload request");
    ws.on_upgrade(move |ws| handle_reload(ws, auto_reload.tx.subscribe()))
}

async fn handle_reload(mut ws: WebSocket, mut rx: broadcast::Receiver<()>) {
    // wait for a message to arrive on rx and then send a signal
    // down the websocket instructing the client to reload
    trace!("Connected to WS client");
    while rx.recv().await.is_ok() {
        trace!("Sending reload signal");
        if ws.send(Message::Text("reload".to_string())).await.is_err() {
            // client has disconnected
            break;
        }
    }
    trace!("Disconnected from WS client");
}
