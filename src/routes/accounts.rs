use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use crate::{
    handlers::accounts::{
        get_balance, get_account_balance, deposit, withdraw, transfer, get_account
    },
    middleware::auth_middleware,
};
use crate::services::AccountService;

// 👈 CAMBIA: Router sin genérico
pub fn routes(account_service: AccountService) -> Router {
    Router::new()
        //.route("/balance", get(get_balance)) No funciona
        .route("/me/balance", get(get_account_balance))
        .route("/deposit", post(deposit))
        .route("/withdraw", post(withdraw))
        .route("/transfer", post(transfer))
        .route("/{account_id}", get(get_account)) //comprobar id
        .layer(middleware::from_fn(auth_middleware))
        .with_state(account_service)
}