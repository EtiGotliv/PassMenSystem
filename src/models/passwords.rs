use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Password {
    pub password_id: i64,
    pub user_id: i64,
    pub domain: String,
    pub password_encrypted: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePasswordDto {
    pub user_id: i64,
    pub domain: String,
    pub password_encrypted: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordDto {
    pub domain: Option<String>,
    pub password_encrypted: Option<String>,
}