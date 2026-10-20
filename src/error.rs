use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};

use serde_json::json;
pub enum AppError {
    NotFound,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
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