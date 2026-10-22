use askama::Template;
use axum::{http::StatusCode, response::Html};

use crate::error::AppError;

#[derive(Template)]
#[template(path = "errors/404.html")]
struct NotFoundTemplate;

pub async fn not_found() -> Result<(StatusCode, Html<String>), AppError> {
    let html = NotFoundTemplate
        .render()
        .map_err(|error| AppError::Internal(error.to_string()))?;

    Ok((StatusCode::NOT_FOUND, Html(html)))
}
