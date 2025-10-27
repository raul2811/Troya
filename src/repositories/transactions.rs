use sqlx::PgPool;
use uuid::Uuid;
use crate::models::TransactionHistory; // 👈 Importa el modelo que me diste

#[derive(Clone)]
pub struct TransactionRepository;

impl TransactionRepository {
    pub fn new() -> Self {
        Self
    }

    /// Guarda un nuevo registro de transacción en la base de datos
    pub async fn create(
        &self,
        pool: &PgPool,
        tx_history: &TransactionHistory,
    ) -> Result<TransactionHistory, sqlx::Error> {
        sqlx::query_as!(
            TransactionHistory,
            r#"
            INSERT INTO transaction_history (
                id, user_id, from_account_id, to_account_id,
                amount_cents, description, status, timestamp
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            tx_history.id,
            tx_history.user_id,
            tx_history.from_account_id,
            tx_history.to_account_id,
            tx_history.amount_cents, // Mapeado a 'amount'
            tx_history.description,
            tx_history.status,
            tx_history.timestamp
        )
            .fetch_one(pool)
            .await
    }

    /// Lista el historial de transacciones para un usuario (ID de Postgres)
    pub async fn list_by_user_id(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<TransactionHistory>, sqlx::Error> {
        sqlx::query_as!(
            TransactionHistory,
            r#"
            SELECT * FROM transaction_history
            WHERE user_id = $1
            ORDER BY timestamp DESC
            LIMIT 100
            "#,
            user_id
        )
            .fetch_all(pool)
            .await
    }
}