use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::{
    models::TransactionHistory,
    repositories::TransactionRepository,
};

#[derive(Clone)]
pub struct TransactionService {
    repo: TransactionRepository,
    pool: PgPool,
}

impl TransactionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repo: TransactionRepository::new(),
            pool,
        }
    }

    /// Función principal para registrar una operación en el historial
    pub async fn log_transaction(
        &self,
        user_id: Uuid,         // Quién inició
        from_account_id: Uuid, // De (TB)
        to_account_id: Uuid,   // A (TB)
        amount_cents: i64,
        status: &str,
        description: Option<String>,
    ) -> Result<TransactionHistory, anyhow::Error> {

        let history_entry = TransactionHistory {
            id: Uuid::new_v4(),
            user_id,
            from_account_id,
            to_account_id,
            amount_cents,
            description,
            status: status.to_string(),
            timestamp: Utc::now(),
        };

        match self.repo.create(&self.pool, &history_entry).await {
            Ok(result) => {
                // Opcional: Loguear el éxito si quieres estar 100% seguro
                println!("✅ Transacción registrada exitosamente: {}", result.id);
                Ok(result)
            }
            Err(e) => {
                // ¡ESTO TE DIRÁ EL PROBLEMA!
                eprintln!("❌ ERROR al registrar la transacción en la BD: {:?}", e);
                // Devolvemos el error para que el '?' del llamador lo capture
                Err(anyhow::Error::from(e))
            }
        }
    }

    /// Obtiene el historial para un usuario
    pub async fn get_history_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<TransactionHistory>, anyhow::Error> {
        let history = self.repo.list_by_user_id(&self.pool, user_id).await?;
        Ok(history)
    }
}