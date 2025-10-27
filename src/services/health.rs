use sqlx::PgPool;
use std::sync::Arc;
use tigerbeetle_unofficial as tb;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: String,
    pub tigerbeetle: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct HealthService {
    db_pool: PgPool,
    tb_client: Arc<tb::Client>,
}

impl HealthService {
    pub fn new(db_pool: PgPool, tb_client: Arc<tb::Client>) -> Self {
        Self {
            db_pool,
            tb_client,
        }
    }

    /// Verifica el estado con todas las dependencias
    pub async fn detailed_health(&self) -> HealthStatus {
        let db_health = self.check_database().await;
        let tb_health = self.check_tigerbeetle().await;

        let overall_status = if db_health && tb_health {
            "healthy"
        } else if !db_health && !tb_health {
            "critical"
        } else {
            "degraded"
        };

        HealthStatus {
            status: overall_status.to_string(),
            database: if db_health { "connected" } else { "disconnected" }.to_string(),
            tigerbeetle: if tb_health { "connected" } else { "disconnected" }.to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verifica solo TigerBeetle
    pub async fn tigerbeetle_health(&self) -> HealthStatus {
        let tb_health = self.check_tigerbeetle().await;

        HealthStatus {
            status: if tb_health { "healthy" } else { "unhealthy" }.to_string(),
            database: "unknown".to_string(),
            tigerbeetle: if tb_health { "connected" } else { "disconnected" }.to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verifica la conexión a PostgreSQL
    async fn check_database(&self) -> bool {
        match sqlx::query("SELECT 1")
            .execute(&self.db_pool)
            .await
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Verifica la conexión a TigerBeetle
    async fn check_tigerbeetle(&self) -> bool {
        match self.tb_client.lookup_accounts(vec![]).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}