use axum::extract::FromRef;
use crate::services::fee_service::FeeService;
use crate::services::order_service::OrderService;

#[derive(Clone)]
pub struct AppState {
    pub fee_service: FeeService,
    pub order_service: OrderService
}


impl FromRef<AppState> for FeeService {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.fee_service.clone()
    }
}