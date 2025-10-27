use axum::{routing::get, Router};
use crate::routes::AppState;

pub fn routes(state: AppState) -> Router {  // ✅ Recibe estado y retorna Router
    Router::new()
        .route("/health", get(crate::handlers::health::health_check))
        .route("/health/db", get(crate::handlers::health::health_check_db))
        .route("/health/tigerbeetle", get(crate::handlers::health::health_check_tigerbeetle))
        .with_state(state)  // ✅ Aplica el estado aquí
}