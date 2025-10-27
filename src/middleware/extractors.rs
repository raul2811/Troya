use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use super::auth::AuthUser;

/// Extractor para obtener el user_id del JWT
pub struct AuthUserId(pub Uuid);

impl<S> FromRequestParts<S> for AuthUserId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = parts
            .extensions
            .get::<AuthUser>()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Usuario no autenticado"
                    })),
                )
            })?;

        Ok(AuthUserId(auth_user.user_id))
    }
}

/// Extractor para obtener todo el AuthUser
pub struct AuthUserExtractor(pub AuthUser);

impl<S> FromRequestParts<S> for AuthUserExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Usuario no autenticado"
                    })),
                )
            })?;

        Ok(AuthUserExtractor(auth_user))
    }
}