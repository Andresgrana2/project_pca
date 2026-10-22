use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};

use serde_json::json;
use crate::dto::packing_dto::PackingError;
pub enum AppError {
    NotFound,
    Internal(String),
    Packing(PackingError),
}

impl From<PackingError> for AppError {
    fn from(error: PackingError) -> Self {
        Self::Packing(error)
    }
}

pub trait WebError {
    fn status_code(&self) -> StatusCode;
    fn error_message(&self) -> String;
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Packing(err) => {
                return (err.status_code(), Json(json!({"error": err.error_message()}))).into_response();
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Recurso no encontrado"),
            AppError::Internal(err) => {
                eprintln!("Error interno: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Error interno del servidor")
            }

        };

        let body = Json(json!({"error": message}));

        (status, body).into_response()
    }
}
