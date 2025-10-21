use chrono::{NaiveDateTime, Utc};
use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::{SqlitePool, Row};
use crate::models::users::{User, CreateUserDto, UpdateUserDto};
use crate::utils::hash::hash_password;

// ===================== INIT DATABASE =====================
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let database_url = "C:\\Users\\1\\Documents\\GitHub\\PassMenSystem\\src\\passwords_management_system.db";
    let pool = SqlitePool::connect(database_url).await?;

    println!("Creating users table if not exists...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            user_id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_first_name TEXT NOT NULL,
            user_last_name TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            phone TEXT,
            password_hash_to_login TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            last_login TIMESTAMP,
            is_active BOOLEAN DEFAULT 1
        );
        "#,
    )
    .execute(&pool)
    .await?;
    println!("users table ready");
    Ok(pool)
}

// ===================== CREATE USER =====================
#[post("/users")]
pub async fn create_user(pool: web::Data<SqlitePool>, user: web::Json<CreateUserDto>) -> impl Responder {
    let now: NaiveDateTime = Utc::now().naive_utc();

    match sqlx::query(
        "INSERT INTO users (user_first_name, user_last_name, email, phone, password_hash_to_login, created_at, updated_at, last_login, is_active)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)"
    )
    .bind(&user.user_first_name)
    .bind(&user.user_last_name)
    .bind(&user.email)
    .bind(&user.phone)
    .bind(&user.password_hash_to_login)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&**pool)
    .await
    {
        Ok(result) => {
            let new_user = User {
                user_id: result.last_insert_rowid(),
                user_first_name: user.user_first_name.clone(),
                user_last_name: user.user_last_name.clone(),
                email: user.email.clone(),
                phone: user.phone.clone().unwrap_or_default(),
                password_hash_to_login: user.password_hash_to_login.clone(),
                created_at: now,
                updated_at: now,
                last_login: Some(now),
                is_active: true,
            };
            HttpResponse::Created().json(new_user)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== READ ALL USERS =====================
#[get("/users")]
pub async fn get_users(pool: web::Data<SqlitePool>) -> impl Responder {
    match sqlx::query("SELECT * FROM users ORDER BY user_id")
        .fetch_all(&**pool)
        .await
    {
        Ok(rows) => {
            let users: Vec<User> = rows
                .iter()
                .map(|row| User {
                    user_id: row.get("user_id"),
                    user_first_name: row.get("user_first_name"),
                    user_last_name: row.get("user_last_name"),
                    email: row.get("email"),
                    phone: row.get("phone"),
                    password_hash_to_login: row.get("password_hash_to_login"),
                    created_at: row.get::<NaiveDateTime, _>("created_at"),
                    updated_at: row.get::<NaiveDateTime, _>("updated_at"),
                    last_login: row.get::<Option<NaiveDateTime>, _>("last_login"),
                    is_active: row.get("is_active"),
                })
                .collect();
            HttpResponse::Ok().json(users)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== READ ONE USER =====================
#[get("/users/{id}")]
pub async fn get_user(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let id = path.into_inner();

    match sqlx::query("SELECT * FROM users WHERE user_id = ?")
        .bind(id)
        .fetch_optional(&**pool)
        .await
    {
        Ok(Some(row)) => {
            let user = User {
                user_id: row.get("user_id"),
                user_first_name: row.get("user_first_name"),
                user_last_name: row.get("user_last_name"),
                email: row.get("email"),
                phone: row.get("phone"),
                password_hash_to_login: row.get("password_hash_to_login"),
                created_at: row.get::<NaiveDateTime, _>("created_at"),
                updated_at: row.get::<NaiveDateTime, _>("updated_at"),
                last_login: row.get::<Option<NaiveDateTime>, _>("last_login"),
                is_active: row.get("is_active"),
            };
            HttpResponse::Ok().json(user)
        }
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== UPDATE USER =====================
#[put("/users/{id}")]
pub async fn update_user(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    updated: web::Json<UpdateUserDto>,
) -> impl Responder {
    let id = path.into_inner();
    let now: NaiveDateTime = Utc::now().naive_utc();

    match sqlx::query(
        "UPDATE users SET
            user_first_name = COALESCE(?, user_first_name),
            user_last_name = COALESCE(?, user_last_name),
            email = COALESCE(?, email),
            phone = COALESCE(?, phone),
            password_hash_to_login = COALESCE(?, password_hash_to_login),
            is_active = COALESCE(?, is_active),
            last_login = COALESCE(?, last_login),
            updated_at = ?
        WHERE user_id = ?"
    )
    .bind(&updated.user_first_name)
    .bind(&updated.user_last_name)
    .bind(&updated.email)
    .bind(&updated.phone)
    .bind(&updated.password_hash_to_login)
    .bind(&updated.is_active)
    .bind(&updated.last_login)
    .bind(now)
    .bind(&id)
    .execute(&**pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().body("User updated successfully")
            } else {
                HttpResponse::NotFound().body("User not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== DELETE USER =====================
#[delete("/users/{id}")]
pub async fn delete_user(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let id = path.into_inner();

    match sqlx::query("DELETE FROM users WHERE user_id = ?")
        .bind(id)
        .execute(&**pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().body("User deleted successfully")
            } else {
                HttpResponse::NotFound().body("User not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
