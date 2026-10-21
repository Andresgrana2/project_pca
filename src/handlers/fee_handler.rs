use axum::{
    http::StatusCode,
    Json
};

use crate::dto::fee_dto::FeeOutput;
use crate::services::fee_service;

pub async fn get_fee_handler() -> (StatusCode, Json<FeeOutput>){

    let fee = fee_service::get_fee().await;

    (StatusCode::OK, Json(fee))

}