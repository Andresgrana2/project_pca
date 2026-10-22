use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use crate::dto::order_dto::{OrderInput, OrderOutput};
use crate::error::AppError;
use crate::states::AppState;

pub async fn create_order_handler(
    State(state): State<AppState>,
    Json(order_input): Json<OrderInput>
) -> Result<(StatusCode, Json<OrderOutput>), AppError> {

    let order = state.order_service
        .create_order(order_input, &state.fee_service).await?;

    Ok(( StatusCode::CREATED, Json(order)))
}
