pub mod auth;
pub mod extractors;

// Re-export de los componentes principales
pub use auth::{auth_middleware, AuthUser, Claims};
pub use extractors::{AuthUserId, AuthUserExtractor};

// Implementación de FromRef para que Axum pueda extraer AccountService de AppState
use axum::extract::FromRef;
use crate::routes::AppState;
use crate::services::AccountService;

impl FromRef<AppState> for AccountService {
    fn from_ref(state: &AppState) -> Self {
        state.account_service.clone()
    }
}