use axum::{response::Redirect, routing::get, Router};

use crate::handlers::html::order_controller::new_order;
use crate::states::AppState;

pub fn html_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::to("/orders/new") }))
        .route("/orders/new", get(new_order))
}
