pub mod health;
pub mod auth;
pub mod accounts;
pub mod tigerbeetle_service;
pub mod transactions;
pub mod users;
pub use auth::AuthService;
pub use accounts::AccountService;
pub use tigerbeetle_service::TigerBeetleService;
use crate::database::DatabaseConnections;
pub use users::UserService;
pub use transactions::TransactionService;
/// Contenedor principal de servicios
#[derive(Clone)]
pub struct AppServices {
    pub health: health::HealthService,
}

impl AppServices {
    pub fn new(connections: DatabaseConnections) -> Self {
        Self {
            health: health::HealthService::new(
                connections.db_pool.clone(),
                connections.tb_client.clone(),
            ),
        }
    }
}