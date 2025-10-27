use sqlx::PgPool;
use uuid::Uuid;

use crate::models::database::User;

#[derive(Clone)]
pub struct UserRepository;

impl UserRepository {
    pub fn new() -> Self {
        Self
    }

    /// Crear un nuevo usuario
    pub async fn create(&self, pool: &PgPool, user: &User) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (
            id, email, password_hash, first_name, last_name,
            document_number, phone, date_of_birth, is_active,
            created_at, updated_at,
            tb_account_id -- <-- 1. Agregado aquí
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) -- <-- 2. Aumentado a $12
        RETURNING *
        "#,
        user.id,
        user.email,
        user.password_hash,
        user.first_name,
        user.last_name,
        user.document_number,
        user.phone,
        user.date_of_birth,
        user.is_active,
        user.created_at,
        user.updated_at,
        user.tb_account_id // <-- 3. Y agregado aquí
    )
            .fetch_one(pool)
            .await
    }

    /// Buscar usuario por email
    pub async fn find_by_email(&self, pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
            .fetch_optional(pool)
            .await
    }

    /// Buscar usuario por ID
    pub async fn find_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        )
            .fetch_optional(pool)
            .await
    }

    /// Verificar si un email ya existe
    pub async fn email_exists(&self, pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as exists",
            email
        )
            .fetch_one(pool)
            .await?;

        Ok(result.exists.unwrap_or(false))
    }
    /// Actualizar usuario
    pub async fn update(&self, pool: &PgPool, user: &User) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            UPDATE users 
            SET first_name = $1, last_name = $2, phone = $3, updated_at = $4
            WHERE id = $5
            RETURNING *
            "#,
            user.first_name,
            user.last_name,
            user.phone,
            user.updated_at,
            user.id
        )
            .fetch_one(pool)
            .await
    }

    /// Desactivar usuario (soft delete)
    pub async fn deactivate(&self, pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1",
            user_id
        )
            .execute(pool)
            .await?;

        Ok(())
    }
}