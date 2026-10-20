mod config;
mod error;
mod routes;
mod handlers;

use axum::{
    extract::{ Path, Query},
    routing::{get, post},
    Json, Router,
};

use serde::{ Serialize, Deserialize};


#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await.unwrap();

    axum::serve(listener, app).await.unwrap();



}

