use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: i64,
    pub user_first_name: String,
    pub user_last_name: String,
    pub email: String,
    pub phone: String,
    pub password_hash_to_login: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_login: Option<NaiveDateTime>,
    pub is_active: bool,
}

//create
#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
    pub user_first_name: String,
    pub user_last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub password_hash_to_login: String,
}

//update
#[derive(Debug, Deserialize, Default)]
pub struct UpdateUserDto {
    pub user_first_name: Option<String>,
    pub user_last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password_hash_to_login: Option<String>,
    pub is_active: Option<bool>,
    pub last_login: Option<NaiveDateTime>,
}
