use axum::{
    http::StatusCode,
    Json
};
use axum::extract::State;
use crate::dto::fee_dto::FeeOutput;
use crate::error::AppError;
use crate::services::fee_service::FeeService;

pub async fn get_fee_handler(
    State(service): State<FeeService>,
) -> Result<Json<FeeOutput>, AppError>{

    let fee = service.get_fee().await;

    Ok(Json(fee))

}