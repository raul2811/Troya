use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
    http::StatusCode,
};
use {
    // Para rutas, usa crate::routes::...
    crate::routes::AppState,

    // Para middleware, usa crate::middleware::...
    crate::middleware::AuthUserId,

    // Para models, usa crate::models::...
    crate::models::api::auth::{RefreshRequest, LogoutResponse, TokenResponse, CreateUserRequest, LoginRequest, RegisterResponse},

    // Si hubieras tenido una dependencia externa, se importaría normal (ejemplo)
    // std::sync::Arc,
};

// Define un tipo de error que implemente IntoResponse
pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.0)
        ).into_response()
    }
}

// Convierte automáticamente anyhow::Error en AppError
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// Actualiza los handlers
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    let response = state.auth_service.register(request).await?;
    Ok(Json(response))
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let response = state.auth_service.login(request).await?;
    Ok(Json(response))
}

pub async fn logout(
    AuthUserId(user_id): AuthUserId,
    State(state): State<AppState>,
) -> Result<Json<LogoutResponse>, AppError> {
    let response = state.auth_service.logout(user_id).await?;
    Ok(Json(response))
}

pub async fn refresh_token(
    AuthUserId(user_id): AuthUserId,
    State(state): State<AppState>,
    Json(refresh_request): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    if user_id != refresh_request.user_id {
        return Err(anyhow::anyhow!("Token no coincide con usuario").into());
    }
    let response = state.auth_service.refresh_token(refresh_request).await?;
    Ok(Json(response))
}