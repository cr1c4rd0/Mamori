//! Conexión de la terminal con el servidor de sincronización (D-29).
//!
//! - `Http`: servidor real Axum + PostgreSQL (por defecto, `MAMORI_SERVER_URL`).
//! - `Sim`: servidor simulado en archivos, para pruebas sin base de datos (`MAMORI_SERVER=sim`).

use crate::sim_server::SimServer;
use mamori_shared::sync::{PullRequest, PullResponse, PushRequest, PushResponse};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("servidor no disponible (sin conexión)")]
    Unreachable,
    #[error("el servidor respondió con error: {0}")]
    Failed(String),
}

#[derive(Clone)]
pub struct HttpServer {
    base_url: String,
    client: reqwest::Client,
}

impl HttpServer {
    pub fn new(base_url: &str) -> Result<Self, ServerError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ServerError::Failed(e.to_string()))?;
        Ok(Self { base_url: base_url.trim_end_matches('/').to_string(), client })
    }

    async fn post<Req: Serialize, Resp: DeserializeOwned>(&self, path: &str, req: &Req) -> Result<Resp, ServerError> {
        let resp = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .json(req)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    ServerError::Unreachable
                } else {
                    ServerError::Failed(e.to_string())
                }
            })?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ServerError::Failed(format!("HTTP {status}: {body}")));
        }
        resp.json().await.map_err(|e| ServerError::Failed(e.to_string()))
    }
}

#[derive(Clone)]
pub enum SyncServer {
    Http(HttpServer),
    Sim(SimServer),
}

impl SyncServer {
    pub async fn push(&self, req: PushRequest) -> Result<PushResponse, ServerError> {
        match self {
            SyncServer::Http(s) => s.post("/api/v1/sync/push", &req).await,
            SyncServer::Sim(s) => s.push(req).await,
        }
    }

    pub async fn pull(&self, req: PullRequest) -> Result<PullResponse, ServerError> {
        match self {
            SyncServer::Http(s) => s.post("/api/v1/sync/pull", &req).await,
            SyncServer::Sim(s) => s.pull(req).await,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            SyncServer::Http(s) => format!("servidor {}", s.base_url),
            SyncServer::Sim(_) => "servidor simulado en archivos".to_string(),
        }
    }
}
