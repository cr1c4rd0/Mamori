use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("petición inválida: {0}")]
    BadRequest(String),
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),
    #[error("error de serialización: {0}")]
    Serde(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Database(_) | ApiError::Serde(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if status.is_server_error() {
            tracing::error!("{}", self);
        }
        (status, Json(serde_json::json!({ "error": self.to_string() }))).into_response()
    }
}
