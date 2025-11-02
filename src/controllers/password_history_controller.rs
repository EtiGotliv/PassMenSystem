use actix_web::{post, get, web, HttpResponse, Responder};
use sqlx::{SqlitePool, Row};
use crate::models::password_history::{PasswordHistory, CreatePasswordHistoryDto};
use chrono::{NaiveDateTime, Utc};

// ===================== INIT DATABASE =====================
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // let database_url = "C:\\Users\\1\\Documents\\GitHub\\PassMenSystem\\src\\passwords_management_system.db";
    let database_url = "sqlite://src/passwords_management_system.db";
    let pool = SqlitePool::connect(database_url).await?;

    println!("Creating passwords history table if not exists...");
    sqlx::query(
        r#"
        CREATE TABLE password_history (
        history_id INTEGER PRIMARY KEY AUTOINCREMENT,
        password_id INTEGER NOT NULL,
        old_password_encrypted TEXT NOT NULL,
        changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(password_id) REFERENCES passwords(password_id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;
    println!("passwords history table ready");
    Ok(pool)
}

// ===================== CREATE PASSWORD HISTORY =====================
#[post("/password-history")]
pub async fn create_password_history(pool: web::Data<SqlitePool>, ph: web::Json<CreatePasswordHistoryDto>) -> impl Responder {
    let now = Utc::now().naive_utc();

    match sqlx::query(
        "INSERT INTO password_history (password_id, old_password_encrypted, changed_at) VALUES (?, ?, ?)"
    )
    .bind(ph.password_id)
    .bind(&ph.old_password_encrypted)
    .bind(now)
    .execute(&**pool)
    .await
    {
        Ok(result) => {
            let new_history = PasswordHistory {
                history_id: result.last_insert_rowid(),
                password_id: ph.password_id,
                old_password_encrypted: ph.old_password_encrypted.clone(),
                changed_at: now,
            };
            HttpResponse::Created().json(new_history)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== GET HISTORY FOR PASSWORD =====================
#[get("/password-history/{password_id}")]
pub async fn get_password_history(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let password_id = path.into_inner();

    match sqlx::query("SELECT * FROM password_history WHERE password_id = ? ORDER BY changed_at DESC")
        .bind(password_id)
        .fetch_all(&**pool)
        .await
    {
        Ok(rows) => {
            let histories: Vec<PasswordHistory> = rows.iter().map(|row| PasswordHistory {
                history_id: row.get("history_id"),
                password_id: row.get("password_id"),
                old_password_encrypted: row.get("old_password_encrypted"),
                changed_at: row.get::<NaiveDateTime,_>("changed_at"),
            }).collect();
            HttpResponse::Ok().json(histories)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
