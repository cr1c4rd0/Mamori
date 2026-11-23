use axum::{routing::get, Json, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "mamori_server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Iniciando Mamori Central Server v0.1.0...");

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/status", get(server_status));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Servidor escuchando en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn server_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "online",
        "service": "Mamori Central Security Server",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now()
    }))
}
