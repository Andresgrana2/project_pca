use axum::Router;
use axum::routing::get;

use crate::handlers::fee_handler::get_fee_handler;
use crate::states::AppState;

pub fn fee_routes() -> Router<AppState> {
    Router::new()
        .route("/fee", get(get_fee_handler))
}
