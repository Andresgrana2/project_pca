mod config;
mod error;
mod routes;
mod handlers;
mod services;
mod model;
mod db;
mod dto;

use axum::Router;

use crate::routes::fee_route::fee_routes;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest("/api", fee_routes());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await.unwrap();

    axum::serve(listener, app).await.unwrap();



}

