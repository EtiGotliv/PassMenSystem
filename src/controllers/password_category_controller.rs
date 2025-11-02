use actix_web::{post, delete, get, web, HttpResponse, Responder};
use sqlx::SqlitePool;
use crate::models::password_category::{PasswordCategory, CreatePasswordCategoryDto};
use sqlx::Row;
// ===================== CREATE =====================
#[post("/password-category")]
pub async fn create_password_category(
    pool: web::Data<SqlitePool>,
    pc: web::Json<CreatePasswordCategoryDto>
) -> impl Responder {
    let result = sqlx::query("INSERT INTO password_category (password_id, category_id) VALUES (?, ?)")
        .bind(pc.password_id)
        .bind(pc.category_id)
        .execute(&**pool)
        .await;

    match result {
        Ok(_) => HttpResponse::Created().body("Password-Category link created"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== DELETE =====================
#[delete("/password-category")]
pub async fn delete_password_category(
    pool: web::Data<SqlitePool>,
    pc: web::Json<CreatePasswordCategoryDto>
) -> impl Responder {
    let result = sqlx::query("DELETE FROM password_category WHERE password_id = ? AND category_id = ?")
        .bind(pc.password_id)
        .bind(pc.category_id)
        .execute(&**pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => HttpResponse::Ok().body("Password-Category link deleted"),
        Ok(_) => HttpResponse::NotFound().body("Link not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== GET ALL =====================
#[get("/password-category")]
pub async fn get_all_password_categories(pool: web::Data<SqlitePool>) -> impl Responder {
    let result = sqlx::query("SELECT password_id, category_id FROM password_category")
        .fetch_all(&**pool)
        .await;

    match result {
        Ok(rows) => {
            let links: Vec<PasswordCategory> = rows.into_iter().map(|row| PasswordCategory {
                password_id: row.get::<i64, _>("password_id"),
                category_id: row.get::<i64, _>("category_id"),
            }).collect();

            HttpResponse::Ok().json(links)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
