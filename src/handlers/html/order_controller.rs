use askama::Template;
use axum::response::Html;

use crate::error::AppError;

#[derive(Template)]
#[template(path = "orders/new.html")]
struct OrderTemplate;

pub async fn new_order() -> Result<Html<String>, AppError> {
    let html = OrderTemplate
        .render()
        .map_err(|error| AppError::Internal(error.to_string()))?;

    Ok(Html(html))
}
