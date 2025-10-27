use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use validator::Validate;
use crate::{
    models::api::users::{UpdateUserRequest, UserResponse, UserAccountsResponse},
    routes::AppState,
    services::UserService,
    middleware::AuthUserId,  // ← Extractor del JWT
};

/// Obtener perfil del usuario actual (usa JWT, NO path)
pub async fn get_current_user(
    State(state): State<AppState>,
    AuthUserId(user_id): AuthUserId,  // ← Extraído del JWT, NO del path
) -> Result<Json<UserResponse>, StatusCode> {
    let user_service = UserService::new(state.db_pool.clone(), state.tb_client.clone());

    match user_service.get_current_user(user_id).await {
        Ok(user) => {
            tracing::info!("✅ Perfil obtenido para usuario: {}", user.email);
            Ok(Json(user))
        }
        Err(e) => {
            tracing::error!("❌ Error obteniendo perfil: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Actualizar perfil del usuario (usa JWT, NO path)
pub async fn update_user(
    State(state): State<AppState>,
    AuthUserId(user_id): AuthUserId,  // ← Extraído del JWT
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    // Validar payload
    if let Err(validation_errors) = payload.validate() {
        tracing::warn!("Validation errors: {:?}", validation_errors);
        return Err(StatusCode::BAD_REQUEST);
    }

    let user_service = UserService::new(state.db_pool.clone(), state.tb_client.clone());

    match user_service.update_user(user_id, payload).await {
        Ok(user) => {
            tracing::info!("✅ Perfil actualizado para usuario: {}", user.email);
            Ok(Json(user))
        }
        Err(e) => {
            tracing::error!("❌ Error actualizando perfil: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Obtener usuario con sus cuentas (usa JWT, NO path)
pub async fn get_user_accounts(
    State(state): State<AppState>,
    AuthUserId(user_id): AuthUserId,  // ← Extraído del JWT
) -> Result<Json<UserAccountsResponse>, StatusCode> {
    let user_service = UserService::new(state.db_pool.clone(), state.tb_client.clone());

    match user_service.get_user_with_accounts(user_id).await {
        Ok(response) => {
            tracing::info!("✅ Cuentas obtenidas para usuario: {}", response.user.email);
            tracing::info!("   Número de cuentas: {}", response.accounts.len());
            if let Some(account) = response.accounts.first() {
                tracing::info!("   Saldo: {} {}", account.current_balance, account.currency);
            }
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("❌ Error obteniendo cuentas: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Desactivar usuario (usa JWT, NO path)
pub async fn deactivate_user(
    State(state): State<AppState>,
    AuthUserId(user_id): AuthUserId,  // ← Extraído del JWT
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_service = UserService::new(state.db_pool.clone(), state.tb_client.clone());

    match user_service.deactivate_user(user_id).await {
        Ok(()) => {
            tracing::info!("✅ Usuario desactivado: {}", user_id);
            Ok(Json(serde_json::json!({
                "message": "Usuario desactivado exitosamente"
            })))
        }
        Err(e) => {
            tracing::error!("❌ Error desactivando usuario: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}