use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;
use validator::Validate;
use chrono::{DateTime, Utc};

// ==================== REQUEST DTOs ====================

/// JSON esperado para el registro de un nuevo usuario
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email, length(max = 255))]
    pub email: String,

    #[validate(length(min = 8, max = 100))]
    pub password: String,

    #[validate(length(min = 1, max = 100))]
    pub first_name: String,

    #[validate(length(min = 1, max = 100))]
    pub last_name: String,

    #[validate(length(min = 5, max = 50))]
    pub document_number: String,

    #[validate(length(max = 20))]
    pub phone: Option<String>,

    pub date_of_birth: NaiveDate,
    pub tb_account_id: Option<Uuid>,
}

/// JSON esperado para el inicio de sesión
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1))]
    pub password: String,
}

/// JSON para refrescar token
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// JSON para cambiar contraseña
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 8))]
    pub current_password: String,

    #[validate(length(min = 8))]
    pub new_password: String,
}

// ==================== RESPONSE DTOs ====================

/// Respuesta al iniciar sesión exitosamente
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user_id: Uuid,
    pub email: String,
}

/// Respuesta al registrar usuario exitosamente
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user: UserResponse,
    pub account: AccountResponse,
    pub tokens: TokenResponse,
}

/// Información básica del usuario para respuestas
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub document_number: String,
    pub phone: Option<String>,
    pub date_of_birth: NaiveDate,
}

/// Información básica de la cuenta para respuestas
#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub account_number: String,
    pub account_type: String,
    pub currency: String,
    pub current_balance: i64,
}

/// Claims para JWT tokens
#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct TokenClaims {
    pub sub: Uuid,      // user_id
    pub exp: usize,     // expiration timestamp
    pub iat: usize,     // issued at timestamp
    pub email: String,
}


#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub user_id: Uuid,
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}
