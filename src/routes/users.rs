use axum::{
    middleware,
    routing::{get, put, delete},
    Router,
};
use crate::{
    handlers::users::{
        get_current_user,
        update_user,
        get_user_accounts,
        deactivate_user,
    },
    routes::AppState,
    middleware::auth::auth_middleware,
};


pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/me/current_user", get(get_current_user))
        .route("/me/update_user", put(update_user))
        .route("/me/deactivate_user",delete(deactivate_user))
        .route("/me/accounts", get(get_user_accounts))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state)
}