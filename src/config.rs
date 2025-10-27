use dotenvy::dotenv;
use std::env;
use uuid::Uuid;
use std::error::Error; // 👈 NECESARIO para devolver Box<dyn Error>

// Definimos un tipo de alias para el error de carga
type ConfigResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub tigerbeetle_address: String,
    pub tigerbeetle_cluster_id: u32,
    pub tigerbeetle_replica_id: u32,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub bcrypt_cost: u32,
    pub bank_account_uuid: Option<Uuid>,
    pub bank_initial_balance: Option<u64>,
}

impl Config {
    // ⚠️ ESTA ES LA FUNCIÓN REQUERIDA POR EL MAIN.RS DE DIAGNÓSTICO
    pub fn from_env_with_diagnostics() -> ConfigResult<Self> {
        dotenv().ok();

        // --- Extracción de variables con error handling ---

        // 1. DATABASE_URL (Requerido)
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL debe estar definida.")?;

        // 2. JWT_SECRET (Requerido)
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET debe estar definida.")?;

        // 3. HOST (Opcional con default)
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        // 4. PORT (Opcional con default y parseo)
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .map_err(|e| format!("PORT debe ser un número válido: {}", e))?;

        // 5. TIGERBEETLE_ADDRESS (Opcional con default)
        let tigerbeetle_address = env::var("TIGERBEETLE_ADDRESS")
            .unwrap_or_else(|_| "127.0.0.1:3000".to_string());

        // 6. TIGERBEETLE_CLUSTER_ID (Opcional con default y parseo)
        let tigerbeetle_cluster_id = env::var("TIGERBEETLE_CLUSTER_ID")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<u32>()
            .map_err(|e| format!("TIGERBEETLE_CLUSTER_ID debe ser un número: {}", e))?;

        // 7. TIGERBEETLE_REPLICA_ID (Opcional con default y parseo)
        let tigerbeetle_replica_id = env::var("TIGERBEETLE_REPLICA_ID")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<u32>()
            .map_err(|e| format!("TIGERBEETLE_REPLICA_ID debe ser un número: {}", e))?;

        // 8. JWT_EXPIRATION_HOURS (Opcional con default y parseo)
        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<i64>()
            .map_err(|e| format!("JWT_EXPIRATION_HOURS debe ser un número: {}", e))?;

        // 9. BCRYPT_COST (Opcional con default y parseo)
        let bcrypt_cost = env::var("BCRYPT_COST")
            .unwrap_or_else(|_| "12".to_string())
            .parse::<u32>()
            .map_err(|e| format!("BCRYPT_COST debe ser un número: {}", e))?;

        // 10. BANK_ACCOUNT_UUID (Opcional)
        let bank_account_uuid = env::var("BANK_ACCOUNT_UUID")
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok());

        // 11. BANK_INITIAL_BALANCE (Opcional)
        let bank_initial_balance = env::var("BANK_INITIAL_BALANCE")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());


        Ok(Self {
            database_url,
            host,
            port,
            tigerbeetle_address,
            tigerbeetle_cluster_id,
            tigerbeetle_replica_id,
            jwt_secret,
            jwt_expiration_hours,
            bcrypt_cost,
            bank_account_uuid,
            bank_initial_balance,
        })
    }

    // Dejamos la función from_env() original por si aún la usas en otro lado
    // y la renombramos a from_env_default, aunque es mejor reemplazarla
    pub fn from_env() -> Self {
        // En una aplicación real, esta función debería llamar al método con diagnóstico
        // por ahora, simplemente implementamos la lógica de carga para que no falte
        Config::from_env_with_diagnostics().unwrap()
    }
}