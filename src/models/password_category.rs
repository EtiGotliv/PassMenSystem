use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePasswordCategoryDto {
    pub password_id: i64,
    pub category_id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PasswordCategory {
    pub password_id: i64,
    pub category_id: i64,
}
