use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use tigerbeetle_unofficial as tb;
use chrono::Utc;

use crate::{
    models::{
        api::account::{
            BalanceResponse, AccountBalanceResponse, DepositRequest,
            WithdrawRequest, TransferRequest, AccountResponse
        },
        database::User, // 👈 Importa el User struct
    },
    repositories::UserRepository,
    services::{TigerBeetleService, TransactionService},
    config::Config,
};

#[derive(Clone)]
pub struct AccountService {
    tigerbeetle_service: TigerBeetleService,
    user_repository: UserRepository,
    pool: PgPool,
    config: Config,
    transaction_service: TransactionService,
}

impl AccountService {
    pub fn new(pool: PgPool, tb_client: Arc<tb::Client>, config: Config) -> Self {
        Self {
            tigerbeetle_service: TigerBeetleService::new(tb_client),
            user_repository: UserRepository::new(),
            transaction_service: TransactionService::new(pool.clone()),
            pool,
            config,
        }
    }

    /// Obtener el balance de una cuenta (Helper)
    pub async fn get_balance(&self, account_id: Uuid) -> Result<BalanceResponse, anyhow::Error> {
        let account = self.tigerbeetle_service.get_account(account_id).await?;
        let balance = (account.credits_posted() as i64) - (account.debits_posted() as i64);

        Ok(BalanceResponse {
            account_id,
            balance,
            ledger: account.ledger(),
            credits_posted: account.credits_posted() as i64,
            debits_posted: account.debits_posted() as i64,
        })
    }

    // -------------------------------------------------------------------------
    // ----- FUNCIONES CORREGIDAS PARA USAR tb_account_id -----
    // -------------------------------------------------------------------------

    /// Obtener balance detallado de la cuenta del usuario
    pub async fn get_account_balance(&self, user_id: Uuid) -> Result<AccountBalanceResponse, anyhow::Error> {
        // 👈 1. Obtener el usuario de Postgres
        let user = self.get_user_by_id(user_id).await?;
        // 👈 2. Obtener el ID de la cuenta de TB
        let tb_account_id = user.tb_account_id.ok_or_else(|| anyhow::anyhow!("Usuario no tiene una cuenta de TigerBeetle vinculada"))?;

        // 👈 3. Usar el tb_account_id para las operaciones
        let balance = self.tigerbeetle_service.get_account_balance(tb_account_id).await?;
        let account_number = self.tigerbeetle_service.generate_account_number(user_id);

        Ok(AccountBalanceResponse {
            account_id: tb_account_id, // 👈 Devuelve el ID de TB
            account_number,
            current_balance: balance as i64,
            available_balance: balance as i64,
            currency: "USD".to_string(),
            last_updated: Utc::now(),
        })
    }

    /// Realizar un depósito (desde cuenta del banco a cuenta del usuario)
    pub async fn deposit(
        &self,
        user_id: Uuid,
        request: DepositRequest,
    ) -> Result<BalanceResponse, anyhow::Error> {
        // 👈 1. Obtener el usuario y su tb_account_id
        let user = self.get_user_by_id(user_id).await?;
        let to_account_id = user.tb_account_id.ok_or_else(|| anyhow::anyhow!("Usuario no tiene una cuenta de TigerBeetle vinculada"))?;

        // 👈 2. Obtener la cuenta del banco
        let bank_account_id = self.get_bank_account_id()?;

        // 👈 3. Crear la transferencia
        self.tigerbeetle_service.create_transfer(
            bank_account_id,     // FROM: cuenta del banco
            to_account_id,       // TO: cuenta del usuario (tb_account_id)
            request.amount_cents as u64,
        ).await?;

        let _ = self.transaction_service.log_transaction(
            user_id,             // Quién inició
            bank_account_id,     // De (TB)
            to_account_id,       // A (TB)
            request.amount_cents, // Positivo para el usuario
            "completed",
            request.description.clone(),
        ).await;

        tracing::info!("✅ Depósito realizado: {} centavos a cuenta {}", request.amount_cents, to_account_id);

        // Retornar el nuevo balance
        self.get_balance(to_account_id).await
    }

    /// Realizar un retiro (desde cuenta del usuario a cuenta del banco)
    pub async fn withdraw(
        &self,
        user_id: Uuid,
        request: WithdrawRequest,
    ) -> Result<BalanceResponse, anyhow::Error> {
        // 👈 1. Obtener el usuario y su tb_account_id
        let user = self.get_user_by_id(user_id).await?;
        let from_account_id = user.tb_account_id.ok_or_else(|| anyhow::anyhow!("Usuario no tiene una cuenta de TigerBeetle vinculada"))?;

        // 👈 2. Verificar fondos
        let current_balance = self.tigerbeetle_service.get_account_balance(from_account_id).await?;
        if current_balance < request.amount_cents as u64 {
            return Err(anyhow::anyhow!("Fondos insuficientes"));
        }

        // 👈 3. Obtener la cuenta del banco
        let bank_account_id = self.get_bank_account_id()?;

        // 👈 4. Crear la transferencia
        self.tigerbeetle_service.create_transfer(
            from_account_id,     // FROM: cuenta del usuario (tb_account_id)
            bank_account_id,     // TO: cuenta del banco
            request.amount_cents as u64,
        ).await?;

        let _ = self.transaction_service.log_transaction(
            user_id,
            from_account_id,
            bank_account_id,
            -request.amount_cents, // Negativo (es un retiro)
            "completed",
            request.description.clone(),
        ).await;

        tracing::info!("✅ Retiro realizado: {} centavos de cuenta {}", request.amount_cents, from_account_id);

        // Retornar el nuevo balance
        self.get_balance(from_account_id).await
    }

    /// Realizar una transferencia entre usuarios
    pub async fn transfer(
        &self,
        from_user_id: Uuid, // 👈 ID del usuario (del token)
        request: TransferRequest, // 👈 Contiene el to_account_id (de TB)
    ) -> Result<BalanceResponse, anyhow::Error> {

        // 👈 1. Obtener la cuenta de origen (del usuario en el token)
        let from_user = self.get_user_by_id(from_user_id).await?;
        let from_account_id = from_user.tb_account_id.ok_or_else(|| anyhow::anyhow!("Usuario origen no tiene una cuenta de TigerBeetle vinculada"))?;

        // 👈 2. El ID de destino viene en el request
        let to_account_id = request.to_account_id;

        // 👈 3. Validar que la cuenta de destino existe
        //    (Esto evita enviar dinero a una cuenta inválida)
        let _to_account = self.tigerbeetle_service.get_account(to_account_id).await
            .map_err(|_| anyhow::anyhow!("La cuenta de destino no existe"))?;

        if from_account_id == to_account_id {
            return Err(anyhow::anyhow!("No se puede transferir a la misma cuenta"));
        }

        // 👈 4. Verificar fondos
        let current_balance = self.tigerbeetle_service.get_account_balance(from_account_id).await?;
        if current_balance < request.amount_cents as u64 {
            return Err(anyhow::anyhow!("Fondos insuficientes"));
        }

        // 👈 5. Crear la transferencia
        self.tigerbeetle_service.create_transfer(
            from_account_id,
            to_account_id,
            request.amount_cents as u64,
        ).await?;

        let _ = self.transaction_service.log_transaction(
            from_user_id,
            from_account_id,
            to_account_id,
            -request.amount_cents, // Negativo para el emisor
            "completed",
            request.description.clone(),
        ).await;

        tracing::info!("✅ Transferencia realizada: {} centavos de {} a {}",
            request.amount_cents, from_account_id, to_account_id);

        self.get_balance(from_account_id).await
    }

    /// Obtener información de cuenta del usuario
    pub async fn get_user_account(&self, user_id: Uuid) -> Result<AccountResponse, anyhow::Error> {
        // 👈 1. Obtener el usuario y su tb_account_id
        let user = self.get_user_by_id(user_id).await?;
        let tb_account_id = user.tb_account_id.ok_or_else(|| anyhow::anyhow!("Usuario no tiene una cuenta de TigerBeetle vinculada"))?;

        // 👈 2. Usar tb_account_id
        let balance = self.tigerbeetle_service.get_account_balance(tb_account_id).await?;
        let account_number = self.tigerbeetle_service.generate_account_number(user_id);

        Ok(AccountResponse {
            id: tb_account_id,
            account_number,
            account_type: "Savings".to_string(),
            currency: "USD".to_string(),
            current_balance: balance as i64,
        })
    }

    // -------------------------------------------------------------------------
    // ----- MÉTODOS PRIVADOS HELPER -----
    // -------------------------------------------------------------------------

    /// Obtener el UUID de la cuenta del banco desde la configuración
    fn get_bank_account_id(&self) -> Result<Uuid, anyhow::Error> {
        self.config.bank_account_uuid
            .ok_or_else(|| anyhow::anyhow!("BANK_ACCOUNT_UUID no está configurado"))
    }

    // 👈 NUEVO HELPER
    /// Obtiene un usuario de Postgres o devuelve un error
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, anyhow::Error> {
        self.user_repository.find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Usuario no encontrado"))
    }
}