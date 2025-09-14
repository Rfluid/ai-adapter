mod apidoc;
mod config;
mod handlers;
mod models;
mod routes;
mod services;
mod synch;
mod utils;

use std::sync::Arc;

use axum::{Router, routing::post};
use config::Config;
use reqwest;
use synch::mutex_swapper::MutexSwapper;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub http: reqwest::Client,
    pub mutex_swapper: Arc<MutexSwapper<String>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = Config::from_env().expect("Failed to load configuration");
    let http = reqwest::Client::new();
    // Compute before moving state anywhere
    let addr = format!("{}:{}", cfg.app_host, cfg.app_port);

    // Create a new instance of the MutexSwapper
    let mutex_swapper = Arc::new(MutexSwapper::new());

    // Now build state and move it into the app (no clone needed)
    let state = AppState {
        cfg,
        http,
        mutex_swapper,
    };

    let app = Router::new()
        .route("/webhooks/waha", post(routes::waha::receive_waha))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", apidoc::ApiDoc::openapi()))
        .with_state(state);

    // axum 0.7 style:
    let listener = TcpListener::bind(&addr).await.unwrap();

    tracing::info!("AI Adapter listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
