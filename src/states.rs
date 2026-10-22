use axum::extract::FromRef;
use crate::services::fee_service::FeeService;

#[derive(Clone)]
pub struct AppState {
    pub fee_service: FeeService,
}


impl FromRef<AppState> for FeeService {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.fee_service.clone()
    }
}