use axum::{
    // --- 👇 CAMBIO: Importa 'Extension' y 'debug_handler' ---
    extract::{State, Extension},
    response::{IntoResponse, Json},
    http::StatusCode,
    debug_handler,
};
use uuid::Uuid;
use crate::{
    routes::AppState,
    // --- 👇 CAMBIO: Importa 'AuthUser' (el struct del middleware) ---
    middleware::AuthUser,
    services::TransactionService,
};

#[debug_handler] // 👈 Sigue necesitando esto
/// Handler para obtener el historial de transacciones del usuario autenticado
pub async fn get_transaction_history(
    State(state): State<AppState>,
    // --- 👇 CAMBIO: Extrae 'AuthUser' desde las 'Extensiones' ---
    // Esto será 'None' si el middleware no autenticó al usuario
    maybe_auth_user: Option<Extension<AuthUser>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    // --- 👇 CAMBIO: Verifica que el usuario fue autenticado ---
    let auth_user = match maybe_auth_user {
        Some(Extension(user)) => user,
        None => {
            // Esto pasa si no se proveyó un token o fue inválido
            let error_response = serde_json::json!({
                "error": "No autorizado",
                "details": "Token de autenticación requerido o inválido."
            });
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    // 'auth_user.user_id' ya es un Uuid, no necesitas parsear.
    let user_id = auth_user.user_id;

    match state.transaction_service.get_history_for_user(user_id).await {
        Ok(history) => Ok((StatusCode::OK, Json(history))),
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "No se pudo obtener el historial",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}