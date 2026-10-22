use axum::http::StatusCode;
use crate::error::WebError;


#[derive(Debug)]
pub enum PackingError{
    InvalidPacking,
    InvalidWeight,
}

impl WebError for PackingError {
    fn status_code(&self) -> StatusCode {
        match self {
            PackingError::InvalidPacking | PackingError::InvalidWeight => StatusCode::BAD_REQUEST,
        }
    }

    fn error_message(&self) -> String {
        match self {
            PackingError::InvalidPacking => "Excede el peso máximo".into(),
            PackingError::InvalidWeight => "El peso debe ser un número finito mayor o igual a 0.01 kg".into(),
        }
    }
}
