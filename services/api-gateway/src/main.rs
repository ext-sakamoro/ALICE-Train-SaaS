//! ALICE SaaS — API Gateway

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

use train_gateway::{
    build_health_response, build_listen_addr, build_proxy_url, validate_body_size, GatewayConfig,
};

/// ヘルスチェックエンドポイント
async fn health() -> Json<train_gateway::GatewayHealth> {
    Json(build_health_response(env!("CARGO_PKG_VERSION")))
}

/// プロキシハンドラ — 全リクエストをアップストリームに転送する
async fn proxy(req: Request<Body>) -> Result<Response, StatusCode> {
    let config = GatewayConfig::from_env();
    let path = req.uri().path().to_string();
    let query = req.uri().query();
    let upstream = build_proxy_url(&config.upstream_url, &path, query);
    let client = reqwest::Client::new();
    let method = req.method().clone();

    // ボディ読み込み（サイズ制限付き）
    let body = match axum::body::to_bytes(req.into_body(), config.max_body_size).await {
        Ok(b) => b,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // 明示的なサイズ検証
    if validate_body_size(body.len(), config.max_body_size).is_err() {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    // アップストリームへ転送
    match client
        .request(method, &upstream)
        .body(body.to_vec())
        .send()
        .await
    {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let body = resp.bytes().await.unwrap_or_default();
            Ok((status, body).into_response())
        }
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let config = GatewayConfig::from_env();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .fallback(proxy)
        .layer(cors);

    let addr = build_listen_addr(config.port);
    info!("Gateway listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
