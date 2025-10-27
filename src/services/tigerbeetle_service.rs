use std::sync::Arc;
use tigerbeetle_unofficial_sys::generated_safe::AccountFlags;
use tigerbeetle_unofficial as tb;
use uuid::Uuid;

#[derive(Clone)]
pub struct TigerBeetleService {
    client: Arc<tb::Client>,
}

impl TigerBeetleService {
    pub fn new(client: Arc<tb::Client>) -> Self {
        Self { client }
    }

    /// Crear una cuenta en TigerBeetle
    pub async fn create_account(
        &self,
        account_id: Uuid,
        user_id: Uuid,
        initial_balance: u64, // Nota: este argumento no se está usando
    ) -> Result<(), anyhow::Error> {
        // Convertir UUID a u128 para TigerBeetle
        let tb_account_id = self.uuid_to_u128(account_id);
        let tb_user_data = self.uuid_to_u128(user_id);

        // 1. Define el flag de seguridad
        let flags = AccountFlags::DEBITS_MUST_NOT_EXCEED_CREDITS;

        // 2. Aplica el flag al crear la cuenta
        let account = tb::Account::new(tb_account_id, 1, 1) // ledger=1, code=1
            .with_user_data_128(tb_user_data)
            .with_flags(flags); // <-- ¡SOLUCIÓN!

        self.client.create_accounts(vec![account]).await?;

        // Nota: Si 'initial_balance' es > 0, aquí deberías crear una
        // transferencia desde la cuenta del banco a esta nueva cuenta.

        Ok(())
    }

    /// Obtener información de una cuenta
    pub async fn get_account(&self, account_id: Uuid) -> Result<tb::Account, anyhow::Error> {
        let tb_account_id = self.uuid_to_u128(account_id);
        let accounts = self.client.lookup_accounts(vec![tb_account_id]).await?;

        accounts.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Cuenta no encontrada en TigerBeetle"))
    }

    /// Obtener saldo de una cuenta
    pub async fn get_account_balance(&self, account_id: Uuid) -> Result<u64, anyhow::Error> {
        let account = self.get_account(account_id).await?;

        // En TigerBeetle, el saldo disponible es: credits_posted - debits_posted
        let balance = account.credits_posted().saturating_sub(account.debits_posted());
        Ok(balance as u64)
    }

    /// Realizar una transferencia
    pub async fn create_transfer(
        &self,
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: u64,
    ) -> Result<(), anyhow::Error> {
        let transfer_id = self.generate_transfer_id();
        let debit_account_id = self.uuid_to_u128(from_account_id);
        let credit_account_id = self.uuid_to_u128(to_account_id);

        let transfer = tb::Transfer::new(transfer_id)
            .with_debit_account_id(debit_account_id)
            .with_credit_account_id(credit_account_id)
            .with_ledger(1)
            .with_code(1)
            .with_amount(amount as u128);

        self.client.create_transfers(vec![transfer]).await?;
        Ok(())
    }
    /// Obtener número de cuentas (health check mejorado)
    pub async fn get_accounts_count(&self) -> Result<usize, anyhow::Error> {
        // Estrategia: crear una cuenta de prueba temporal
        let test_account_id = Uuid::new_v4();
        let test_user_id = Uuid::new_v4();

        // Intentar crear una cuenta con saldo 0
        match self.create_account(test_account_id, test_user_id, 0).await {
            Ok(_) => {
                // Si se creó exitosamente, TigerBeetle está funcionando
                // En una implementación real, aquí contarías las cuentas existentes
                // Por ahora retornamos un valor simulado
                Ok(1)
            }
            Err(e) => {
                // Si falla por "cuenta ya existe", también es una respuesta válida
                if e.to_string().contains("exists") {
                    Ok(1)
                } else {
                    Err(anyhow::anyhow!("TigerBeetle no responde: {}", e))
                }
            }
        }
    }
    
    /// Generar número de cuenta único
    pub fn generate_account_number(&self, user_id: Uuid) -> String {
        format!("001-{:08}", user_id.as_u128() % 100000000)
    }

    /// Generar ID único para transferencia
    fn generate_transfer_id(&self) -> u128 {
        Uuid::new_v4().as_u128()
    }

    // Helper para convertir UUID a u128
    fn uuid_to_u128(&self, uuid: Uuid) -> u128 {
        uuid.as_u128()
    }
    /// Verificar salud de TigerBeetle
    pub async fn check_health(&self) -> Result<bool, anyhow::Error> {
        // Intentar una operación simple de lookup
        let test_account_id = Uuid::new_v4();

        match self.get_account(test_account_id).await {
            Ok(_) => Ok(true), // Esto no debería pasar normalmente
            Err(e) => {
                // Si el error es "cuenta no encontrada", TigerBeetle está respondiendo
                if e.to_string().contains("no encontrada") || e.to_string().contains("not found") {
                    Ok(true)
                } else {
                    // Cualquier otro error significa problemas de conexión
                    Ok(false)
                }
            }
        }
    }
}
