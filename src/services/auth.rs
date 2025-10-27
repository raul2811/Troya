use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation};
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use tigerbeetle_unofficial as tb;
use chrono::{Utc, Duration};

use crate::models::{
                     database::User,
                     api::auth::{
                         CreateUserRequest, LoginRequest, RegisterResponse, TokenResponse, UserResponse,
                         AccountResponse, TokenClaims, LogoutResponse, RefreshRequest,
                     },
};
use crate::repositories::UserRepository;
use crate::services::TigerBeetleService;
#[derive(Clone)]
pub struct AuthService {
    user_repository: UserRepository,
    tigerbeetle_service: TigerBeetleService,
    pool: PgPool,
    jwt_secret: String,
    jwt_expiration_hours: i64,
}

impl AuthService {
    pub fn new(pool: PgPool, tb_client: Arc<tb::Client>, jwt_secret: String, jwt_expiration_hours: i64) -> Self {
        Self {
            user_repository: UserRepository::new(),
            tigerbeetle_service: TigerBeetleService::new(tb_client),
            pool,
            jwt_secret,
            jwt_expiration_hours,
        }
    }

    /// Registrar un nuevo usuario CON cuenta en TigerBeetle
    pub async fn register(&self, request: CreateUserRequest) -> Result<RegisterResponse, anyhow::Error> {
        // ... tu código existente de register ...
        // 1. Verificar si el usuario ya existe
        if self.user_repository.email_exists(&self.pool, &request.email).await? {
            return Err(anyhow::anyhow!("El usuario ya existe"));
        }

        // 2. Hashear contraseña
        let password_hash = hash(&request.password, DEFAULT_COST)?;
        let account_id = Uuid::new_v4();
        // 3. Crear usuario en PostgreSQL
        let user = User {
            id: Uuid::new_v4(),
            email: request.email,
            password_hash,
            first_name: request.first_name,
            last_name: request.last_name,
            document_number: request.document_number,
            phone: request.phone,
            date_of_birth: request.date_of_birth,
            tb_account_id: Some(account_id),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created_user = self.user_repository.create(&self.pool, &user).await?;

        // 4. Crear cuenta bancaria en TigerBeetle
        let account_number = self.tigerbeetle_service.generate_account_number(created_user.id);
        let initial_balance = 0; // Saldo inicial en centavos

        self.tigerbeetle_service.create_account(
            account_id,
            created_user.id,
            initial_balance,
        ).await?;

        // 5. Generar JWT token
        let token_response = self.generate_token_response(&created_user).await?;

        // 6. Crear respuesta
        Ok(RegisterResponse {
            user: self.user_to_response(created_user),
            account: AccountResponse {
                id: account_id,
                account_number,
                account_type: "Savings".to_string(),
                currency: "USD".to_string(),
                current_balance: initial_balance as i64,
            },
            tokens: token_response,
        })
    }

    /// Iniciar sesión
    pub async fn login(&self, request: LoginRequest) -> Result<TokenResponse, anyhow::Error> {
        // ... tu código existente de login ...
        let user = self.user_repository.find_by_email(&self.pool, &request.email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Credenciales inválidas"))?;

        if !verify(&request.password, &user.password_hash)? {
            return Err(anyhow::anyhow!("Credenciales inválidas"));
        }

        if !user.is_active {
            return Err(anyhow::anyhow!("Usuario inactivo"));
        }

        self.generate_token_response(&user).await
    }

    /// 👈 NUEVO: Cerrar sesión (invalida el token)
    pub async fn logout(&self, user_id: Uuid) -> Result<LogoutResponse, anyhow::Error> {
        // Verificar que el usuario existe
        let user = self.user_repository.find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Usuario no encontrado"))?;

        // En un sistema real, aquí invalidarías el token en una blacklist
        // Para esta implementación, simplemente confirmamos el logout
        tracing::info!("👋 Usuario {} cerró sesión", user.email);

        Ok(LogoutResponse {
            message: "Sesión cerrada exitosamente".to_string(),
            user_id,
            timestamp: Utc::now(),
        })
    }

    /// 👈 NUEVO: Refrescar token
    pub async fn refresh_token(&self, refresh_request: RefreshRequest) -> Result<TokenResponse, anyhow::Error> {
        // Verificar el refresh token (en un sistema real, tendrías una tabla de refresh tokens)
        // Por ahora, validamos que el usuario exista y generamos un nuevo token

        let user = self.user_repository.find_by_id(&self.pool, refresh_request.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Usuario no encontrado"))?;

        if !user.is_active {
            return Err(anyhow::anyhow!("Usuario inactivo"));
        }

        // Generar nuevo token
        self.generate_token_response(&user).await
    }

    /// 👈 NUEVO: Validar token (para middleware)
    pub async fn validate_token(&self, token: &str) -> Result<TokenClaims, anyhow::Error> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let validation = Validation::default();

        let token_data = decode::<TokenClaims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("Token inválido: {}", e))?;

        Ok(token_data.claims)
    }

    async fn generate_token_response(&self, user: &User) -> Result<TokenResponse, anyhow::Error> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(self.jwt_expiration_hours))
            .expect("Invalid timestamp")
            .timestamp() as usize;

        let claims = TokenClaims {
            sub: user.id,
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
            email: user.email.clone(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        // En un sistema real, generarías un refresh token diferente
        let refresh_token = format!("refresh_{}_{}", user.id, Utc::now().timestamp());

        Ok(TokenResponse {
            access_token: token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: (self.jwt_expiration_hours * 3600) as i64,
            user_id: user.id,
            email: user.email.clone(),
        })
    }

    fn user_to_response(&self, user: User) -> UserResponse {
        UserResponse {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            document_number: user.document_number,
            phone: user.phone,
            date_of_birth: user.date_of_birth,
        }
    }
}