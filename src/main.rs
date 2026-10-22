mod config;
mod error;
mod routes;
mod handlers;
mod services;
mod model;
mod db;
mod dto;
mod states;

use axum::Router;

use crate::handlers::html::error_controller::not_found;
use crate::routes::fee_route::fee_routes;
use crate::routes::html_route::html_routes;
use crate::routes::order_route::order_routes;
use crate::services::fee_service::FeeService;
use crate::services::order_service::OrderService;
use crate::states::AppState;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let state = AppState {
        fee_service: FeeService::new(),
        order_service: OrderService::new(),
    };

    let app = Router::new()
        .merge(html_routes())
        .nest("/api", fee_routes())
        .nest("/api", order_routes())
        .fallback(not_found)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

