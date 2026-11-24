pub mod sync;

use axum::routing::{get, post};
use axum::{Json, Router};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/status", get(server_status))
        .route("/api/v1/sync/push", post(sync::push))
        .route("/api/v1/sync/pull", post(sync::pull))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn server_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "online",
        "service": "Mamori Central Security Server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now()
    }))
}
