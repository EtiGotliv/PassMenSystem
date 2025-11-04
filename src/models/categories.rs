use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub category_id: i64,
    pub category_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryDto {
    pub category_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCategoryDto {
    pub category_name: Option<String>,
}
