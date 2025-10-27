pub mod config;
pub mod database;
pub mod models;
pub mod routes;
pub mod handlers;
pub mod services;
pub mod repositories;
mod middleware;

use std::net::SocketAddr;
use tracing_subscriber;
use uuid::Uuid;
use std::io::{self, Write}; // 👈 NECESARIO para el FLUSH

#[tokio::main]
async fn main() {
    // 1. Configurar logging y asegurar salida inmediata
    setup_logging();
    // Forzar el vaciado del buffer para asegurar que 'Logging configurado' aparezca
    io::stdout().flush().unwrap();

    // 2. Cargar configuración con chequeo de diagnóstico
    let config = match config::Config::from_env_with_diagnostics() { // 👈 REQUIERE CAMBIO en config::Config
        Ok(c) => {
            println!("✅ Configuración cargada correctamente.");
            println!("   - DATABASE_URL: {}", if c.database_url.starts_with("postgres://") { "OK (oculto)" } else { "ERROR" });
            println!("   - TIGERBEETLE_ADDRESS: {}", c.tigerbeetle_address);
            println!("🚀 Iniciando Sistema Bancario HNL en puerto: {}", c.port);
            io::stdout().flush().unwrap();
            c
        },
        Err(e) => {
            eprintln!("❌ ERROR CRÍTICO AL CARGAR CONFIGURACIÓN: {}", e);
            eprintln!("   Asegúrese de que el archivo .env sea accesible y las variables estén definidas.");
            std::process::exit(1);
        }
    };

    // 3. Conectar a bases de datos con manejo de error explícito
    let connections = match database::connect_to_databases(&config).await {
        Ok(c) => {
            println!("✅ Conexiones a bases de datos inicializadas.");
            io::stdout().flush().unwrap();
            c
        }
        Err(e) => {
            eprintln!("❌ ERROR CRÍTICO AL CONECTAR A BASES DE DATOS:");
            eprintln!("   Detalle: {}", e);
            eprintln!("   Verifique que los servicios de 'postgres' y 'tigerbeetle' estén accesibles en sus host/puertos configurados.");
            std::process::exit(1);
        }
    };

    // 4. Probar conexiones (Ya implementado, pero seguirá el flujo)
    test_database_connections(&connections).await;

    // 5. INICIALIZAR CUENTA DEL BANCO
    initialize_bank_account(&connections.tb_client, &config).await;

    // 6. Crear estado de la aplicación
    let app_state = routes::AppState::new(
        connections.db_pool,
        connections.tb_client,
        config.clone(),
    );

    // 7. Configurar y ejecutar servidor
    let app = routes::create_router(app_state);
    start_server(app, config).await;
}

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    println!("📝 Logging configurado");
}

async fn test_database_connections(connections: &database::DatabaseConnections) {
    println!("🔍 Probando conexiones a bases de datos...");

    match database::test_pg_connection(&connections.db_pool).await {
        Ok(_) => println!("✅ PostgreSQL: Conexión exitosa"),
        Err(e) => {
            eprintln!("❌ PostgreSQL: Error de conexión - {}", e);
            std::process::exit(1);
        }
    }

    match database::test_tb_connection(&connections.tb_client).await {
        Ok(_) => println!("✅ TigerBeetle: Conexión exitosa"),
        Err(e) => {
            eprintln!("❌ TigerBeetle: Error de conexión - {}", e);
        }
    }
}

async fn initialize_bank_account(
    tb_client: &std::sync::Arc<tigerbeetle_unofficial::Client>,
    config: &config::Config,
) {
    println!("🏦 Inicializando cuenta del banco...");

    let tb_service = services::TigerBeetleService::new(tb_client.clone());

    // Verificar que TigerBeetle esté corriendo
    if !check_tigerbeetle_health(&tb_service).await {
        eprintln!("❌ TigerBeetle no está respondiendo, no se puede crear cuenta del banco");
        return;
    }

    let bank_uuid = config.bank_account_uuid
        .unwrap_or_else(|| Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());

    let bank_initial_balance = config.bank_initial_balance.unwrap_or(100_000_000_00); // $100M

    // Verificar si la cuenta ya existe
    if check_bank_account_exists(&tb_service, bank_uuid).await {
        println!("✅ Cuenta del banco ya existe, continuando...");

        // 👈 NUEVO: Verificar y corregir saldo si es necesario
        let current_balance = tb_service.get_account_balance(bank_uuid).await.unwrap_or(0);
        if current_balance < 1_000_000_00 { // Menos de $1M
            println!("⚠️  Saldo del banco bajo (${:.2}), depositando fondos...", current_balance as f64 / 100.0);
            match deposit_initial_funds(&tb_service, bank_uuid, bank_initial_balance).await {
                Ok(_) => println!("✅ Fondos depositados al banco: ${:.2}", bank_initial_balance as f64 / 100.0),
                Err(e) => eprintln!("❌ Error depositando fondos al banco: {}", e),
            }
        } else {
            println!("💰 Saldo actual del banco: ${:.2}", current_balance as f64 / 100.0);
        }
        return;
    }

    // Crear la cuenta del banco con saldo inicial
    match tb_service.create_account(bank_uuid, bank_uuid, bank_initial_balance).await {
        Ok(_) => {
            println!("✅ Cuenta del banco creada exitosamente:");
            println!("   UUID: {}", bank_uuid);
            println!("   Saldo inicial: ${:.2}", bank_initial_balance as f64 / 100.0);
        }
        Err(e) => {
            eprintln!("❌ Error creando cuenta del banco: {}", e);
        }
    }

    // 👈 NUEVO: Debug para verificar la cuenta
    debug_bank_account(&tb_service, bank_uuid).await;
}

// 👈 NUEVA FUNCIÓN: Depositar fondos iniciales al banco
async fn deposit_initial_funds(
    tb_service: &services::TigerBeetleService,
    bank_account_id: Uuid,
    amount: u64,
) -> Result<(), anyhow::Error> {
    // Para depositar fondos al banco, necesitamos una "cuenta sistema" especial
    // que actúe como fuente de fondos infinitos
    let system_account_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002")?;

    // Crear cuenta del sistema si no existe
    if let Err(_) = tb_service.get_account(system_account_id).await {
        tb_service.create_account(system_account_id, system_account_id, u64::MAX).await?;
        println!("🔧 Cuenta del sistema creada con fondos infinitos");
    }

    // Transferir desde cuenta sistema al banco
    tb_service.create_transfer(system_account_id, bank_account_id, amount).await?;

    Ok(())
}

// 👈 NUEVA FUNCIÓN: Debug para ver detalles de la cuenta
async fn debug_bank_account(tb_service: &services::TigerBeetleService, bank_uuid: Uuid) {
    match tb_service.get_account(bank_uuid).await {
        Ok(account) => {
            println!("🔍 DEBUG - Cuenta banco:");
            println!("   ID: {}", bank_uuid);
            println!("   Créditos: {}", account.credits_posted());  // 👈 MÉTODO con ()
            println!("   Débitos: {}", account.debits_posted());    // 👈 MÉTODO con ()
            println!("   Saldo neto: ${:.2}",
                     (account.credits_posted() - account.debits_posted()) as f64 / 100.0); // 👈 MÉTODOS con ()
            println!("   Ledger: {}", account.ledger());           // 👈 MÉTODO con ()
            println!("   Code: {}", account.code());               // 👈 MÉTODO con ()
        }
        Err(e) => {
            eprintln!("❌ Error obteniendo cuenta banco: {}", e);
        }
    }
}

async fn check_tigerbeetle_health(tb_service: &services::TigerBeetleService) -> bool {
    println!("🔍 Verificando salud de TigerBeetle...");

    match tb_service.check_health().await {
        Ok(true) => {
            println!("✅ TigerBeetle saludable");
            true
        }
        Ok(false) => {
            eprintln!("❌ TigerBeetle no responde");
            false
        }
        Err(e) => {
            eprintln!("❌ Error verificando TigerBeetle: {}", e);
            false
        }
    }
}

async fn check_bank_account_exists(tb_service: &services::TigerBeetleService, bank_uuid: Uuid) -> bool {
    match tb_service.get_account(bank_uuid).await {
        Ok(account) => {
            let balance = account.credits_posted() - account.debits_posted(); // 👈 MÉTODOS con ()
            println!("📊 Cuenta del banco encontrada - Saldo: ${:.2}", balance as f64 / 100.0);
            true
        }
        Err(_) => {
            println!("📭 Cuenta del banco no existe, se creará...");
            false
        }
    }
}

async fn start_server(app: axum::Router, config: config::Config) {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    println!("🌐 Servidor iniciando en: http://{}", addr);
    println!("📚 Endpoints disponibles:");

    // Health checks
    println!("   GET  /health - Health check básico");
    println!("   GET  /health/db - Health check con bases de datos");
    println!("   GET  /health/tigerbeetle - Health check TigerBeetle");

    // Autenticación (públicos)
    println!("   🟢 POST /register - Registrar nuevo usuario");
    println!("   🟢 POST /login - Iniciar sesión");
    println!("   🔐 POST /logout - Cerrar sesión");
    println!("   🔐 POST /refresh - Refrescar token");

    // Usuario (requieren autenticación)
    println!("   🔐 GET  /me/current_user - Obtener información del usuario actual");
    println!("   🔐 PUT  /me/update_user - Actualizar información del usuario");
    println!("   🔐 DELETE /me/deactivate_user - Desactivar usuario");
    println!("   🔐 GET  /me/accounts - Obtener cuentas del usuario");

    // Cuentas y transacciones (requieren autenticación)
    println!("   🔐 GET  /balance - Obtener balance");
    println!("   🔐 GET  /me/balance - Obtener balance de mi cuenta");
    println!("   🔐 POST /deposit - Realizar depósito");
    println!("   🔐 POST /withdraw - Realizar retiro");
    println!("   🔐 POST /transfer - Realizar transferencia");
    println!("   🔐 GET  /:account_id - Obtener cuenta específica");
    println!("   🔐 GET  /transactions - Obtener historial de transacciones");
    // Información de autenticación
    println!("");
    println!("   🟢 = Público (sin autenticación)");
    println!("   🔐 = Requiere autenticación JWT");
    println!("   📍 Puerto: {}", config.port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("✅ Servidor escuchando en {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}