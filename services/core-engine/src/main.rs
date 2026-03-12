//! ALICE SaaS — Core Engine エントリポイント

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Instant};
use tracing::info;
use tracing_subscriber::EnvFilter;

use train_engine::{
    build_health_response, handle_process_request, HealthResponse, ProcessRequest, ProcessResponse,
    DEFAULT_PORT,
};

/// アプリケーション共有状態
#[derive(Clone)]
struct AppState {
    start_time: Instant,
}

/// ヘルスチェックエンドポイント
async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(build_health_response(state.start_time.elapsed()))
}

/// データ処理エンドポイント
async fn process(Json(req): Json<ProcessRequest>) -> Result<Json<ProcessResponse>, StatusCode> {
    handle_process_request(&req)
        .map(Json)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let state = Arc::new(AppState {
        start_time: Instant::now(),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/process", post(process))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT));
    info!("Engine listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
