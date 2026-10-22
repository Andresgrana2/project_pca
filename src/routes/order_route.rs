use axum::Router;
use axum::routing::post;
use crate::handlers::order_handler::create_order_handler;
use crate::states::AppState;

pub fn order_routes() -> Router<AppState> {

    Router::new()
        .route("/order/new", post(create_order_handler))
}