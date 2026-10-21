use axum::http::StatusCode;
use serde::Serialize;
use crate::error::WebError;

#[derive(Serialize)]
pub struct PackingInput{
    pub weight: f32
}

pub enum PackingError{
    InvalidPacking
}

impl WebError for PackingError {
    fn status_code(&self) -> StatusCode {
        match self {
            PackingError::InvalidPacking => StatusCode::NOT_FOUND,
        }
    }

    fn error_message(&self) -> String {
        match self {
            PackingError::InvalidPacking => "Excede el peso máximo".into(),
        }
    }
}