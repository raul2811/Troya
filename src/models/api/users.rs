use serde::{Deserialize, Serialize};
use validator::Validate;

// Request DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,

    #[validate(length(max = 20))]
    pub phone: Option<String>,
}

// Response DTOs  
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub document_number: String,
    pub phone: Option<String>,
    pub date_of_birth: chrono::NaiveDate,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserAccountsResponse {
    pub user: UserResponse,
    pub accounts: Vec<crate::models::api::account::AccountResponse>,
}