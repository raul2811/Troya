// src/models/api/account.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate; // Para validar la entrada

// --- DTOs de Entrada (Deserializar y Validar) ---
// (El ID del usuario/cuenta de origen se tomará del token JWT)

/// JSON para una solicitud de DEPÓSITO
/// (El 'from_account' es la cuenta del banco, se maneja en el backend)
#[derive(Debug, Deserialize, Validate)]
pub struct DepositRequest {
    /// Monto en centavos a depositar
    #[validate(range(min = 1))]
    pub amount_cents: i64,

    #[validate(length(max = 255))]
    pub description: Option<String>,
}

/// JSON para una solicitud de RETIRO
/// (El 'to_account' es la cuenta del banco, se maneja en el backend)
#[derive(Debug, Deserialize, Validate)]
pub struct WithdrawRequest {
    /// Monto en centavos a retirar
    #[validate(range(min = 1))]
    pub amount_cents: i64,

    #[validate(length(max = 255))]
    pub description: Option<String>,
}

/// JSON para una solicitud de TRANSFERENCIA (de usuario a usuario)
#[derive(Debug, Deserialize, Validate)]
pub struct TransferRequest {
    /// ID de la cuenta (en TigerBeetle) a la que se envía el dinero.
    pub to_account_id: Uuid,

    /// Monto en centavos a transferir
    #[validate(range(min = 1))]
    pub amount_cents: i64,

    #[validate(length(max = 255))]
    pub description: Option<String>,
}

// --- DTOs de Salida (Serializar) ---

/// JSON de respuesta al consultar el saldo
#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub account_id: Uuid,

    /// Saldo final en centavos
    pub balance: i64,

    /// El "ledger" o libro contable (ej. 1 = USD)
    pub ledger: u32,

    /// Total de créditos (dinero que ha entrado)
    pub credits_posted: i64,

    /// Total de débitos (dinero que ha salido)
    pub debits_posted: i64,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub account_number: String,
    pub account_type: String,
    pub currency: String,
    pub current_balance: i64,
}

#[derive(Debug, Serialize)]
pub struct AccountBalanceResponse {
    pub account_id: Uuid,
    pub account_number: String,
    pub current_balance: i64,
    pub available_balance: i64,
    pub currency: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}