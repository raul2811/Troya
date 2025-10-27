use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate, Datelike};

/// Representa un Usuario en la tabla `users` de la base de datos
#[derive(Debug, Serialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,

    #[serde(skip)] //
    pub password_hash: String,

    pub first_name: String,
    pub last_name: String,
    pub document_number: String,
    pub phone: Option<String>,
    pub date_of_birth: NaiveDate,
    pub is_active: bool,
    pub tb_account_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

}

impl User {
    /// Comprueba si el usuario es mayor de 18 años
    pub fn is_adult(&self) -> bool {
        let today = Utc::now().date_naive();

        // 1. Calcula la diferencia de años
        let mut age = today.year() - self.date_of_birth.year();

        // 2. Comprueba si ya ha pasado el cumpleaños de este año
        //    (ordinal() es el "día del año", ej: 1ro de Feb es 32)
        if today.ordinal() < self.date_of_birth.ordinal() {
            age -= 1; // Aún no ha cumplido años este año
        }

        age >= 18
    }

    pub fn is_valid_document(&self) -> bool {
        // Implementar validación específica según país
        self.document_number.chars().all(|c| c.is_alphanumeric())
    }
}