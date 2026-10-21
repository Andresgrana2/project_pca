use axum::Router;
use axum::routing::get;

use crate::handlers::fee_handler::get_fee_handler;

pub fn fee_routes() -> Router {
    Router::new()
        .route("/fee", get(get_fee_handler))
}