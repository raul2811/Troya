// src/models/database/transaction.rs

use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Representa un registro en la tabla `transaction_history` de PostgreSQL
#[derive(Debug, Serialize, FromRow, Clone)]
pub struct TransactionHistory {
    /// ID único de esta entrada de historial (UUID v4)
    pub id: Uuid,

    /// ID del usuario (en Postgres) que *inició* la transacción
    pub user_id: Uuid,

    /// ID de la cuenta (en TigerBeetle) que envió el dinero
    pub from_account_id: Uuid,

    /// ID de la cuenta (en TigerBeetle) que recibió el dinero
    pub to_account_id: Uuid,

    /// Monto transferido (en centavos, p.ej., $10.50 = 1050)
    #[sqlx(rename = "amount")] // Asegura el mapeo correcto
    pub amount_cents: i64,

    /// Descripción opcional de la transacción (ej. "Pago de cena")
    pub description: Option<String>,

    /// Estado de la transacción (ej. "completed", "failed")
    pub status: String,

    /// Fecha y hora en que se registró la transacción
    pub timestamp: DateTime<Utc>,
}