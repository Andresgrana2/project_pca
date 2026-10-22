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

use crate::routes::fee_route::fee_routes;
use crate::services::fee_service::FeeService;
use crate::states::AppState;

#[tokio::main]
async fn main() {
    let state = AppState {
        fee_service: FeeService::new(),
    };

    let app = Router::new()
        .nest("/api", fee_routes())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await.unwrap();

    axum::serve(listener, app).await.unwrap();



}

