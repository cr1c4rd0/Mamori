//! Endpoints de sincronización. Contrato: `specs/contratos_api/protocolo_sincronizacion.md`.

use super::AppState;
use crate::error::ApiError;
use crate::repository::sync_repo;
use axum::extract::State;
use axum::Json;
use mamori_shared::sync::{PullRequest, PullResponse, PushRequest, PushResponse};

pub async fn push(State(state): State<AppState>, Json(req): Json<PushRequest>) -> Result<Json<PushResponse>, ApiError> {
    Ok(Json(sync_repo::push(&state.pool, &req).await?))
}

pub async fn pull(State(state): State<AppState>, Json(req): Json<PullRequest>) -> Result<Json<PullResponse>, ApiError> {
    Ok(Json(sync_repo::pull(&state.pool, &req).await?))
}
