use axum::{
    routing::get,
    Router,
    middleware,
};
use crate::{
    routes::AppState,
    handlers::transactions::get_transaction_history,
    middleware::auth_middleware, // 👈 Importa tu middleware
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        // Esta ruta será GET /transactions/
        .route("/me/transactions", get(get_transaction_history))
        // Protegemos el historial con el middleware
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state)
}