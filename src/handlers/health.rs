use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use crate::routes::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    service: String,
    version: String,
}

#[derive(Serialize)]
pub struct DetailedHealthResponse {
    status: String,
    database: String,
    tigerbeetle: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    service: String,
    version: String,
}

/// Health check básico
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        service: "sistema_bancario_hnl".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Health check con bases de datos
pub async fn health_check_db(
    State(state): State<AppState>,
) -> Result<Json<DetailedHealthResponse>, StatusCode> {
    let (db_health, tb_health) = tokio::join!(
        check_postgres(&state.db_pool),
        check_tigerbeetle(&state.tb_client)
    );

    let status = determine_overall_status(db_health, tb_health);

    if status == "critical" {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok(Json(DetailedHealthResponse {
        status: status.to_string(),
        database: status_string(db_health),
        tigerbeetle: status_string(tb_health),
        timestamp: chrono::Utc::now(),
        service: "sistema_bancario_hnl".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

/// Health check solo TigerBeetle
pub async fn health_check_tigerbeetle(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let tb_health = check_tigerbeetle(&state.tb_client).await;

    if !tb_health {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        service: "sistema_bancario_hnl".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

// Helper functions
async fn check_postgres(pool: &sqlx::PgPool) -> bool {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .is_ok()
}

async fn check_tigerbeetle(tb_client: &std::sync::Arc<tigerbeetle_unofficial::Client>) -> bool {
    tb_client.lookup_accounts(vec![]).await.is_ok()
}

fn determine_overall_status(db: bool, tb: bool) -> &'static str {
    match (db, tb) {
        (true, true) => "healthy",
        (false, false) => "critical",
        _ => "degraded",
    }
}

fn status_string(healthy: bool) -> String {
    if healthy { "connected" } else { "disconnected" }.to_string()
}