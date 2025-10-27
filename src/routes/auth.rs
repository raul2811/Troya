use axum::{
    routing::{post},
    Router, middleware,
};
use crate::handlers::auth;
use crate::routes::AppState;
use crate::middleware::auth_middleware;

// Esta es la función que llamas desde src/routes/mod.rs
pub fn create_auth_router(state: AppState) -> Router {

    // --- 1. Rutas Públicas ---
    // Estas rutas NO tienen el middleware
    let public_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login));

    // --- 2. Rutas Protegidas ---
    // Estas rutas SÍ tienen el middleware
    let protected_routes = Router::new()
        .route("/logout", post(auth::logout))
        .route("/refresh", post(auth::refresh_token))
        // El middleware se aplica SÓLO a este grupo de rutas
        .layer(middleware::from_fn(auth_middleware));

    // --- 3. Combinar ambos routers ---
    // Finalmente, unes los routers público y protegido,
    // y les aplicas el estado.
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}