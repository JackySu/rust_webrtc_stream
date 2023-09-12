
use axum::{Router, routing::{get, post}};
use axum::http::header::{AUTHORIZATION, COOKIE, SEC_WEBSOCKET_PROTOCOL};
use tower::ServiceBuilder;
use tower_http::{cors::{Any, CorsLayer}, ServiceBuilderExt};
use tower_http::request_id::MakeRequestUuid;
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, DefaultMakeSpan, TraceLayer};
use tower_http::LatencyUnit;
use lazy_static::lazy_static;
use tokio::sync::{mpsc, Mutex};
use tracing_subscriber::prelude::*;
use tracing::Level;
use chrono::Duration;
use dotenv::dotenv;
use http::Method;

pub mod api;
pub mod ws;
pub mod auth;
pub mod token;
pub mod user;

use ws::ws_handler;
use auth::auth_handler;

use std::{sync::Arc, collections::HashMap, net::SocketAddr, iter::once};

lazy_static! {
    pub(crate) static ref JWT_SECRET: String = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    pub(crate) static ref AUTHORIZATION_TOKEN_EXPIRACY_TIME: Duration = Duration::hours(4);
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub txs: Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let state = AppState {
        txs: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/auth", post(auth_handler))
        .route("/:uuid", get(ws_handler))
        .layer(CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any)
            .allow_origin(Any))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                )
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        .layer(SetSensitiveRequestHeadersLayer::new(once(COOKIE)))
        .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
        .layer(SetSensitiveRequestHeadersLayer::new(once(
            SEC_WEBSOCKET_PROTOCOL,
        )))
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .propagate_x_request_id(),
        )
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(10)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3536));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    Ok(())
}

fn init_tracing() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
            .without_time()
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_default(Level::INFO)
                .with_target("webrtc_stream_server", tracing::Level::INFO)
        )
        .init();
}