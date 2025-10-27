pub mod auth;
pub mod users;
pub mod accounts;
pub mod transactions;
pub mod health;

use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use tigerbeetle_unofficial as tb;
use crate::services::{AccountService, UserService, AuthService, TransactionService};

/// Estado compartido de la aplicación
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub tb_client: Arc<tb::Client>,
    pub config: crate::config::Config,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub account_service: AccountService,
    pub transaction_service: TransactionService,
}

impl AppState {
    pub fn new(db_pool: PgPool, tb_client: Arc<tb::Client>, config: crate::config::Config) -> Self {
        Self {
            db_pool: db_pool.clone(),
            user_service: UserService::new(db_pool.clone(), tb_client.clone()),
            auth_service: AuthService::new(
                db_pool.clone(),
                tb_client.clone(),
                config.jwt_secret.clone(),
                config.jwt_expiration_hours
            ),
            account_service: AccountService::new(
                db_pool.clone(),
                tb_client.clone(),
                config.clone()
            ),
            transaction_service:TransactionService::new(db_pool.clone()),
            tb_client,
            config,
        }
    }
}

/// Crea el router con el estado de la aplicación
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(health::routes(state.clone()))
        .merge(auth::create_auth_router(state.clone()))
        .merge(users::routes(state.clone()))
        .merge(accounts::routes(state.account_service.clone()))
        .merge(transactions::routes(state.clone()))
}