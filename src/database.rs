use crate::config::Config;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use tigerbeetle_unofficial as tb;
use std::net::ToSocketAddrs;

// --- 1. Definimos nuestros Tipos ---

pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type PgPool = Pool<Postgres>;
pub type TbClient = Arc<tb::Client>;

#[derive(Clone)]
pub struct DatabaseConnections {
    pub db_pool: PgPool,
    pub tb_client: TbClient,
}

// --- 2. Funciones de Conexión Individuales ---

async fn create_pg_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
}

fn create_tb_client(config: &Config) -> Result<tb::Client, Box<dyn std::error::Error + Send + Sync>> {
    // Resolver el hostname DNS a dirección IP
    println!("🔍 Resolviendo dirección: {}", config.tigerbeetle_address);

    let socket_addr = config.tigerbeetle_address
        .to_socket_addrs()
        .map_err(|e| format!("Error resolviendo dirección {}: {}", config.tigerbeetle_address, e))?
        .next()
        .ok_or_else(|| format!("No se pudo resolver la dirección: {}", config.tigerbeetle_address))?;

    // Convertir SocketAddr a String (formato "IP:puerto")
    let resolved_address = socket_addr.to_string();
    println!("✅ Dirección resuelta: {}", resolved_address);

    // Crear el cliente (String implementa AsRef<[u8]>)
    let client = tb::Client::new(
        config.tigerbeetle_cluster_id.into(),
        resolved_address
    )?;

    println!("🎉 Cliente TigerBeetle creado exitosamente");
    Ok(client)
}

// --- 3. Función Principal de Conexión ---

pub async fn connect_to_databases(config: &Config) -> AppResult<DatabaseConnections> {
    let db_pool = create_pg_pool(config).await?;
    let tb_client = create_tb_client(config)?;

    Ok(DatabaseConnections {
        db_pool,
        tb_client: Arc::new(tb_client),
    })
}

// --- 4. Funciones de Prueba ---

pub async fn test_pg_connection(pool: &PgPool) -> AppResult<()> {
    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await?;
    println!("✅ Conexión a PostgreSQL exitosa: SELECT 1 = {}", row.0);
    Ok(())
}

pub async fn test_tb_connection(client: &tb::Client) -> AppResult<()> {
    // Probamos con un array vacío para verificar conexión
    let accounts = client.lookup_accounts(vec![]).await;

    match accounts {
        Ok(_) => {
            println!("✅ Conexión a TigerBeetle exitosa.");
            Ok(())
        },
        Err(e) => {
            println!("❌ Error al conectar con TigerBeetle: {:?}", e);
            Err(Box::new(e))
        }
    }
}