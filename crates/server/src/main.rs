mod api;
mod error;
mod repository;

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Credenciales locales de desarrollo en `.env` (fuera de git).
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "mamori_server=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Iniciando Mamori Central Server v{}...", env!("CARGO_PKG_VERSION"));

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "Falta DATABASE_URL (defínala en el archivo .env de la raíz del proyecto)")?;
    let pool = PgPoolOptions::new().max_connections(10).connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Base de datos lista (migraciones aplicadas)");

    if repository::sync_repo::seed_if_empty(&pool).await? {
        tracing::info!("Base vacía: datos iniciales cargados");
    }
    let towers = repository::sync_repo::ensure_seed_towers(&pool).await?;
    if towers > 0 {
        tracing::info!("Torres iniciales agregadas: {}", towers);
    }

    let app = api::router(api::AppState { pool }).layer(TraceLayer::new_for_http());

    // Solo escucha en local mientras no haya autenticación de terminales (DOM-07d).
    let addr: SocketAddr = std::env::var("MAMORI_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()?;
    tracing::info!("Servidor escuchando en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
