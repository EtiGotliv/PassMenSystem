use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordHistory {
    pub history_id: i64,
    pub password_id: i64,
    pub old_password_encrypted: String,
    pub changed_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePasswordHistoryDto {
    pub password_id: i64,
    pub old_password_encrypted: String,
}
