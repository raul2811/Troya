use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::api::account::{
        DepositRequest, WithdrawRequest, TransferRequest,
        BalanceResponse, AccountBalanceResponse, AccountResponse,
    },
    services::AccountService,
    middleware::AuthUserId,  // ← Extractor del JWT
};

/// GET /api/accounts/balance
/// Obtener balance de la cuenta del usuario autenticado
pub async fn get_balance(
    State(service): State<AccountService>,
    AuthUserId(user_id): AuthUserId,  // ← Del JWT, no del path
) -> Result<Json<BalanceResponse>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_balance(user_id).await {
        Ok(balance) => Ok(Json(balance)),
        Err(e) => {
            tracing::error!("Error al obtener balance: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Error al obtener balance",
                    "details": e.to_string()
                })),
            ))
        }
    }
}

/// GET /api/accounts/me
/// Obtener información detallada de la cuenta del usuario
pub async fn get_account_balance(
    State(service): State<AccountService>,
    AuthUserId(user_id): AuthUserId,  // ← Del JWT
) -> Result<Json<AccountBalanceResponse>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_account_balance(user_id).await {
        Ok(balance) => Ok(Json(balance)),
        Err(e) => {
            tracing::error!("Error al obtener información de cuenta: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Error al obtener información de cuenta",
                    "details": e.to_string()
                })),
            ))
        }
    }
}

/// POST /api/accounts/deposit
/// Realizar un depósito a la cuenta del usuario
pub async fn deposit(
    State(service): State<AccountService>,
    AuthUserId(user_id): AuthUserId,  // ← Del JWT
    Json(request): Json<DepositRequest>,
) -> Result<Json<BalanceResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Validar el request
    if let Err(e) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Datos de entrada inválidos",
                "details": e.to_string()
            })),
        ));
    }

    match service.deposit(user_id, request).await {
        Ok(balance) => Ok(Json(balance)),
        Err(e) => {
            tracing::error!("Error al realizar depósito: {:?}", e);

            let status = if e.to_string().contains("no encontrado") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(json!({
                    "error": "Error al realizar depósito",
                    "details": e.to_string()
                })),
            ))
        }
    }
}

/// POST /api/accounts/withdraw
/// Realizar un retiro de la cuenta del usuario
pub async fn withdraw(
    State(service): State<AccountService>,
    AuthUserId(user_id): AuthUserId,  // ← Del JWT
    Json(request): Json<WithdrawRequest>,
) -> Result<Json<BalanceResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Validar el request
    if let Err(e) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Datos de entrada inválidos",
                "details": e.to_string()
            })),
        ));
    }

    match service.withdraw(user_id, request).await {
        Ok(balance) => Ok(Json(balance)),
        Err(e) => {
            tracing::error!("Error al realizar retiro: {:?}", e);

            let status = if e.to_string().contains("Fondos insuficientes") {
                StatusCode::BAD_REQUEST
            } else if e.to_string().contains("no encontrado") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(json!({
                    "error": "Error al realizar retiro",
                    "details": e.to_string()
                })),
            ))
        }
    }
}

/// POST /api/accounts/transfer
/// Realizar una transferencia a otra cuenta
pub async fn transfer(
    State(service): State<AccountService>,
    AuthUserId(user_id): AuthUserId,  // ← Del JWT
    Json(request): Json<TransferRequest>,
) -> Result<Json<BalanceResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Validar el request
    if let Err(e) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Datos de entrada inválidos",
                "details": e.to_string()
            })),
        ));
    }

    match service.transfer(user_id, request).await {
        Ok(balance) => Ok(Json(balance)),
        Err(e) => {
            tracing::error!("Error al realizar transferencia: {:?}", e);

            let status = if e.to_string().contains("Fondos insuficientes") {
                StatusCode::BAD_REQUEST
            } else if e.to_string().contains("no encontrado") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("misma cuenta") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(json!({
                    "error": "Error al realizar transferencia",
                    "details": e.to_string()
                })),
            ))
        }
    }
}

/// GET /api/accounts/{account_id}
/// Obtener información de una cuenta específica
/// (Solo permite consultar la propia cuenta del usuario autenticado)
pub async fn get_account(
    State(service): State<AccountService>,
    AuthUserId(user_id): AuthUserId,  // ← Del JWT
    Path(account_id): Path<Uuid>,
) -> Result<Json<AccountResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Verificar que el usuario solo pueda consultar su propia cuenta
    if user_id != account_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "No autorizado para consultar esta cuenta"
            })),
        ));
    }

    match service.get_user_account(account_id).await {
        Ok(account) => Ok(Json(account)),
        Err(e) => {
            tracing::error!("Error al obtener cuenta: {:?}", e);

            let status = if e.to_string().contains("no encontrado") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(json!({
                    "error": "Error al obtener cuenta",
                    "details": e.to_string()
                })),
            ))
        }
    }
}