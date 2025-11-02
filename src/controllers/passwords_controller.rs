use chrono::{NaiveDateTime, Utc};
use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::{SqlitePool, Row};
use crate::models::passwords::{Password, CreatePasswordDto, UpdatePasswordDto};
use crate::utils::encryption::{encrypt_password, decrypt_password};

// ===================== INIT DATABASE =====================
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let database_url = "sqlite://src/passwords_management_system.db";
    let pool = SqlitePool::connect(database_url).await?;

    println!("Creating passwords table if not exists...");
    sqlx::query(
        r#"
        CREATE TABLE passwords (
        password_id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        domain TEXT NOT NULL,
        password_encrypted TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(user_id) REFERENCES users(user_id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;
    println!("passwords table ready");
    Ok(pool)
}

// ===================== CREATE PASSWORD =====================
#[post("/passwords")]
pub async fn create_password(pool: web::Data<SqlitePool>, password: web::Json<CreatePasswordDto>) -> impl Responder {
    let now = Utc::now().naive_utc();

    // הצפנת הסיסמה לפני השמירה
    let encrypted = match encrypt_password(&password.password_encrypted) {
        Ok(enc) => enc,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Encryption error: {}", e)),
    };

    match sqlx::query(
        "INSERT INTO passwords (user_id, domain, password_encrypted, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(password.user_id)
    .bind(&password.domain)
    .bind(&encrypted) // שומר מוצפן
    .bind(now)
    .bind(now)
    .execute(&**pool)
    .await
    {
        Ok(result) => {
            let new_password = Password {
                password_id: result.last_insert_rowid(),
                user_id: password.user_id,
                domain: password.domain.clone(),
                password_encrypted: password.password_encrypted.clone(), // מחזיר למשתמש מפורש
                created_at: now,
                updated_at: now,
            };
            HttpResponse::Created().json(new_password)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== READ ALL PASSWORDS =====================
#[get("/passwords")]
pub async fn get_passwords(pool: web::Data<SqlitePool>) -> impl Responder {
    match sqlx::query("SELECT * FROM passwords ORDER BY password_id")
        .fetch_all(&**pool)
        .await
    {
        Ok(rows) => {
            let passwords: Vec<Password> = rows.iter().map(|row| {
                let encrypted: String = row.get("password_encrypted");
                let decrypted = decrypt_password(&encrypted).unwrap_or("[decryption error]".to_string());
                Password {
                    password_id: row.get("password_id"),
                    user_id: row.get("user_id"),
                    domain: row.get("domain"),
                    password_encrypted: decrypted,
                    created_at: row.get::<NaiveDateTime,_>("created_at"),
                    updated_at: row.get::<NaiveDateTime,_>("updated_at"),
                }
            }).collect();
            HttpResponse::Ok().json(passwords)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== READ ONE PASSWORD =====================
#[get("/passwords/{id}")]
pub async fn get_password(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let id = path.into_inner();

    match sqlx::query("SELECT * FROM passwords WHERE password_id = ?")
        .bind(id)
        .fetch_optional(&**pool)
        .await
    {
        Ok(Some(row)) => {
            let encrypted: String = row.get("password_encrypted");
            let decrypted = decrypt_password(&encrypted).unwrap_or("[decryption error]".to_string());
            let password = Password {
                password_id: row.get("password_id"),
                user_id: row.get("user_id"),
                domain: row.get("domain"),
                password_encrypted: decrypted,
                created_at: row.get::<NaiveDateTime,_>("created_at"),
                updated_at: row.get::<NaiveDateTime,_>("updated_at"),
            };
            HttpResponse::Ok().json(password)
        }
        Ok(None) => HttpResponse::NotFound().body("Password not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== UPDATE PASSWORD =====================
#[put("/passwords/{id}")]
pub async fn update_password(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    updated: web::Json<UpdatePasswordDto>,
) -> impl Responder {
    let id = path.into_inner();
    let now = Utc::now().naive_utc();

    // הצפנה לפני עדכון
    let encrypted = match &updated.password_encrypted {
        Some(pwd) => match encrypt_password(pwd) {
            Ok(enc) => Some(enc),
            Err(e) => return HttpResponse::InternalServerError().body(format!("Encryption error: {}", e)),
        },
        None => None,
    };

    match sqlx::query(
        "UPDATE passwords SET
            domain = COALESCE(?, domain),
            password_encrypted = COALESCE(?, password_encrypted),
            updated_at = ?
         WHERE password_id = ?"
    )
    .bind(&updated.domain)
    .bind(&encrypted)
    .bind(now)
    .bind(id)
    .execute(&**pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().body("Password updated successfully")
            } else {
                HttpResponse::NotFound().body("Password not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== DELETE PASSWORD =====================
#[delete("/passwords/{id}")]
pub async fn delete_password(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let id = path.into_inner();

    match sqlx::query("DELETE FROM passwords WHERE password_id = ?")
        .bind(id)
        .execute(&**pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().body("Password deleted successfully")
            } else {
                HttpResponse::NotFound().body("Password not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
