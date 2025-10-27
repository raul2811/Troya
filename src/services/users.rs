use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use tigerbeetle_unofficial as tb;

use crate::{
    models::{
        database::User,
        api::users::{UpdateUserRequest, UserResponse, UserAccountsResponse},
    },
    repositories::UserRepository,
    services::TigerBeetleService,
};

#[derive(Clone)]
pub struct UserService {
    user_repository: UserRepository,
    tigerbeetle_service: TigerBeetleService,
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool, tb_client: Arc<tb::Client>) -> Self {
        Self {
            user_repository: UserRepository::new(),
            tigerbeetle_service: TigerBeetleService::new(tb_client),
            pool,
        }
    }

    /// Obtener perfil del usuario actual
    pub async fn get_current_user(&self, user_id: Uuid) -> Result<UserResponse, anyhow::Error> {
        let user = self.user_repository.find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Usuario no encontrado"))?;

        Ok(self.user_to_response(user))
    }

    /// Actualizar perfil del usuario
    pub async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest
    ) -> Result<UserResponse, anyhow::Error> {
        let mut user = self.user_repository.find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Usuario no encontrado"))?;

        // Actualizar campos permitidos
        if let Some(first_name) = request.first_name {
            user.first_name = first_name;
        }
        if let Some(last_name) = request.last_name {
            user.last_name = last_name;
        }
        if let Some(phone) = request.phone {
            user.phone = Some(phone);
        }

        user.updated_at = chrono::Utc::now();

        let updated_user = self.user_repository.update(&self.pool, &user).await?;

        Ok(self.user_to_response(updated_user))
    }

    /// Obtener usuario con sus cuentas (desde TigerBeetle)
    pub async fn get_user_with_accounts(&self, user_id: Uuid) -> Result<UserAccountsResponse, anyhow::Error> {
        let user = self.user_repository.find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Usuario no encontrado"))?;

        // TODO: En una implementación real, necesitaríamos mapear user_id a account_id
        // Por ahora, asumimos que el account_id es el mismo que user_id (como en el registro)
        let account_id = user_id;

        // Consultar TigerBeetle para obtener información de la cuenta
        let account_info = self.tigerbeetle_service.get_account(account_id).await?;
        let balance = self.tigerbeetle_service.get_account_balance(account_id).await?;
        let account_number = self.tigerbeetle_service.generate_account_number(user_id);

        let account_response = crate::models::api::account::AccountResponse {
            id: account_id,
            account_number,
            account_type: "Savings".to_string(), // Por defecto
            currency: "USD".to_string(), // Por defecto
            current_balance: balance as i64,
        };

        Ok(UserAccountsResponse {
            user: self.user_to_response(user),
            accounts: vec![account_response],
        })
    }

    /// Desactivar usuario (soft delete)
    pub async fn deactivate_user(&self, user_id: Uuid) -> Result<(), anyhow::Error> {
        self.user_repository.deactivate(&self.pool, user_id).await?;
        Ok(())
    }

    /// Convertir User de DB a UserResponse
    fn user_to_response(&self, user: User) -> UserResponse {
        UserResponse {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            document_number: user.document_number,
            phone: user.phone,
            date_of_birth: user.date_of_birth,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}