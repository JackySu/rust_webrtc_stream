use axum::{Router, routing::get};
use tokio::sync::{mpsc, Mutex};
use tracing_subscriber::prelude::*;

use std::{sync::Arc, collections::HashMap, net::SocketAddr};

pub mod ws;
use ws::ws_handler;


#[derive(Debug, Clone)]
pub struct AppState {
    pub txs: Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>,
}

#[tokio::main]
async fn main() {
    init_tracing();
    let state = AppState {
        txs: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/:uuid", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3536));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
            .without_time()
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_default(tracing::Level::DEBUG)
                .with_target("webrtc_stream_server", tracing::Level::INFO)
        )
        .init();
}