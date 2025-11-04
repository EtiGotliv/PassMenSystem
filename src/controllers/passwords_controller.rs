use chrono::{NaiveDateTime, Utc};
use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::{SqlitePool, Row};
use crate::models::passwords::{Password, CreatePasswordDto, UpdatePasswordDto};
use crate::utils::encryption::{encrypt_password, decrypt_password};
use crate::controllers::password_history_controller::create_password_history_internal; // נדרש בשביל היסטוריית סיסמאות

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

// // ===================== CREATE PASSWORD =====================
// #[post("/passwords")]
// pub async fn create_password(pool: web::Data<SqlitePool>, password: web::Json<CreatePasswordDto>) -> impl Responder {
//     let now = Utc::now().naive_utc();

//     let encrypted = match encrypt_password(&password.password_encrypted) {
//         Ok(enc) => enc,
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Encryption error: {}", e)),
//     };

//     let insert_result = sqlx::query(
//         "INSERT INTO passwords (user_id, domain, password_encrypted, created_at, updated_at)
//          VALUES (?, ?, ?, ?, ?)"
//     )
//     .bind(password.user_id)
//     .bind(&password.domain)
//     .bind(&encrypted)
//     .bind(now)
//     .bind(now)
//     .execute(&**pool)
//     .await;

//     let password_id = match insert_result {
//         Ok(result) => result.last_insert_rowid(),
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
//     };

//     // הפקת סיומת לדומיין (ליצירת קטגוריה)
//     let domain = password.domain.to_lowercase();
//     let parts: Vec<&str> = domain.split('.').collect();
//     let suffix = if parts.len() >= 2 {
//         if parts.len() >= 3 && parts[parts.len() - 2].len() <= 3 {
//             format!(".{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
//         } else {
//             format!(".{}", parts.last().unwrap())
//         }
//     } else {
//         ".unknown".to_string()
//     };

//     // בדיקה או יצירה של קטגוריה תואמת
//     let category_id: i64 = match sqlx::query("SELECT category_id FROM categories WHERE category_name = ?")
//         .bind(&suffix)
//         .fetch_optional(&**pool)
//         .await
//     {
//         Ok(Some(row)) => row.get("category_id"),
//         Ok(None) => {
//             match sqlx::query("INSERT INTO categories (category_name) VALUES (?)")
//                 .bind(&suffix)
//                 .execute(&**pool)
//                 .await
//             {
//                 Ok(result) => result.last_insert_rowid(),
//                 Err(e) => return HttpResponse::InternalServerError().body(format!("Error creating category: {}", e)),
//             }
//         }
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Error checking category: {}", e)),
//     };

//     // קישור בין הסיסמה לקטגוריה
//     if let Err(e) = sqlx::query(
//         "INSERT OR IGNORE INTO password_category (password_id, category_id) VALUES (?, ?)"
//     )
//     .bind(password_id)
//     .bind(category_id)
//     .execute(&**pool)
//     .await
//     {
//         return HttpResponse::InternalServerError().body(format!("Error linking password to category: {}", e));
//     }

//     let new_password = Password {
//         password_id,
//         user_id: password.user_id,
//         domain: password.domain.clone(),
//         password_encrypted: password.password_encrypted.clone(),
//         created_at: now,
//         updated_at: now,
//     };

//     HttpResponse::Created().json(new_password)
// }

// ===================== CREATE PASSWORD =====================
#[post("/passwords")]
pub async fn create_password(
    pool: web::Data<SqlitePool>,
    password: web::Json<CreatePasswordDto>
) -> impl Responder {
    let now = Utc::now().naive_utc();

    // 🔹 בדיקה אם כבר קיימת סיסמה לאותו user ולדומיין
    let existing: Option<(i64,)> = match sqlx::query_as(
        "SELECT password_id FROM passwords WHERE user_id = ? AND domain = ?"
    )
    .bind(password.user_id)
    .bind(&password.domain)
    .fetch_optional(&**pool)
    .await
    {
        Ok(res) => res,
        Err(e) => return HttpResponse::InternalServerError().body(format!("DB error: {}", e)),
    };

    if existing.is_some() {
        return HttpResponse::BadRequest().body("Password for this domain already exists for this user");
    }

    // 🔹 הצפנת הסיסמה
    let encrypted = match encrypt_password(&password.password_encrypted) {
        Ok(enc) => enc,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Encryption error: {}", e)),
    };

    // 🔹 הכנסה לטבלת passwords
    let insert_result = match sqlx::query(
        "INSERT INTO passwords (user_id, domain, password_encrypted, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(password.user_id)
    .bind(&password.domain)
    .bind(&encrypted)
    .bind(now)
    .bind(now)
    .execute(&**pool)
    .await
    {
        Ok(res) => res,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    };

    let password_id = insert_result.last_insert_rowid();

    // 🔹 הפקת סיומת לדומיין (ליצירת קטגוריה)
    let domain = password.domain.to_lowercase();
    let parts: Vec<&str> = domain.split('.').collect();
    let suffix = if parts.len() >= 2 {
        if parts.len() >= 3 && parts[parts.len() - 2].len() <= 3 {
            format!(".{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
        } else {
            format!(".{}", parts.last().unwrap())
        }
    } else {
        ".unknown".to_string()
    };

    // 🔹 בדיקה או יצירה של קטגוריה תואמת
    let category_id: i64 = match sqlx::query("SELECT category_id FROM categories WHERE category_name = ?")
        .bind(&suffix)
        .fetch_optional(&**pool)
        .await
    {
        Ok(Some(row)) => row.get("category_id"),
        Ok(None) => {
            match sqlx::query("INSERT INTO categories (category_name) VALUES (?)")
                .bind(&suffix)
                .execute(&**pool)
                .await
            {
                Ok(result) => result.last_insert_rowid(),
                Err(e) => return HttpResponse::InternalServerError().body(format!("Error creating category: {}", e)),
            }
        }
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error checking category: {}", e)),
    };

    // 🔹 קישור בין הסיסמה לקטגוריה
    if let Err(e) = sqlx::query(
        "INSERT OR IGNORE INTO password_category (password_id, category_id) VALUES (?, ?)"
    )
    .bind(password_id)
    .bind(category_id)
    .execute(&**pool)
    .await
    {
        return HttpResponse::InternalServerError().body(format!("Error linking password to category: {}", e));
    }

    let new_password = Password {
        password_id,
        user_id: password.user_id,
        domain: password.domain.clone(),
        password_encrypted: password.password_encrypted.clone(),
        created_at: now,
        updated_at: now,
    };

    HttpResponse::Created().json(new_password)
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
    let password_id = path.into_inner();
    let now = Utc::now().naive_utc();

    // נביא את הסיסמה הישנה ונשמור אותה בהיסטוריה
    let old_row = match sqlx::query("SELECT password_encrypted FROM passwords WHERE password_id = ?")
        .bind(password_id)
        .fetch_optional(&**pool)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return HttpResponse::NotFound().body("Password not found"),
        Err(e) => return HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    };

    let old_password: String = old_row.get("password_encrypted");

    if let Err(e) = create_password_history_internal(&**pool, password_id, &old_password).await {
        eprintln!("Failed to save password history: {}", e);
    }

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
    .bind(password_id)
    .execute(&**pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().body("Password updated successfully and history recorded")
            } else {
                HttpResponse::NotFound().body("Password not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// // ===================== DELETE PASSWORD =====================
// #[delete("/passwords/{id}")]
// pub async fn delete_password(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
//     let id = path.into_inner();

//     match sqlx::query("DELETE FROM passwords WHERE password_id = ?")
//         .bind(id)
//         .execute(&**pool)
//         .await
//     {
//         Ok(result) => {
//             if result.rows_affected() > 0 {
//                 HttpResponse::Ok().body("Password deleted successfully")
//             } else {
//                 HttpResponse::NotFound().body("Password not found")
//             }
//         }
//         Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
//     }
// }
// ===================== DELETE PASSWORD =====================
// #[delete("/passwords/{id}")]
// pub async fn delete_password(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
//     let password_id = path.into_inner();

//     // נביא קודם את הסיסמה הישנה כדי לשמור בהיסטוריה
//     let old_row = match sqlx::query("SELECT password_encrypted FROM passwords WHERE password_id = ?")
//         .bind(password_id)
//         .fetch_optional(&**pool)
//         .await
//     {
//         Ok(Some(row)) => row,
//         Ok(None) => return HttpResponse::NotFound().body("Password not found"),
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
//     };

//     let old_password: String = old_row.get("password_encrypted");

//     // נשמור בהיסטוריה
//     if let Err(e) = create_password_history_internal(&**pool, password_id, &old_password).await {
//         eprintln!("Failed to save password history before deletion: {}", e);
//     }

//     // עכשיו נמחק את הסיסמה
//     match sqlx::query("DELETE FROM passwords WHERE password_id = ?")
//         .bind(password_id)
//         .execute(&**pool)
//         .await
//     {
//         Ok(result) => {
//             if result.rows_affected() > 0 {
//                 HttpResponse::Ok().body("Password deleted successfully and history recorded")
//             } else {
//                 HttpResponse::NotFound().body("Password not found")
//             }
//         }
//         Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
//     }
// }
// ===================== DELETE PASSWORD =====================
#[delete("/passwords/{id}")]
pub async fn delete_password(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>
) -> impl Responder {
    let password_id = path.into_inner();

    // 1️⃣ נביא את הסיסמה הקיימת לפני מחיקה
    let old_row = match sqlx::query("SELECT password_encrypted FROM passwords WHERE password_id = ?")
        .bind(password_id)
        .fetch_optional(&**pool)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return HttpResponse::NotFound().body("Password not found"),
        Err(e) => {
            eprintln!("DB error fetching old password: {}", e);
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
        }
    };

    let old_password: String = old_row.get("password_encrypted");
    println!("Old password fetched for history: {}", old_password);

    // 2️⃣ שמירה בהיסטוריה
    if let Err(e) = create_password_history_internal(&**pool, password_id, &old_password).await {
        eprintln!("Failed to save password history: {}", e);
        // אנחנו ממשיכים למחיקה גם אם ההיסטוריה לא נשמרת
    } else {
        println!("Password history saved successfully for password_id={}", password_id);
    }

    // 3️⃣ מחיקה מהטבלה passwords
    match sqlx::query("DELETE FROM passwords WHERE password_id = ?")
        .bind(password_id)
        .execute(&**pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().body("Password deleted and history recorded successfully")
            } else {
                HttpResponse::NotFound().body("Password not found")
            }
        }
        Err(e) => {
            eprintln!("DB error deleting password: {}", e);
            HttpResponse::InternalServerError().body(format!("Database error: {}", e))
        }
    }
}

// ===================== USERS WITH 3+ PASSWORDS =====================
#[derive(serde::Serialize)]
pub struct PasswordSummary {
    pub user_id: i64,
    pub user_first_name: String,
    pub user_last_name: String,
    pub password_count: i64,
}

#[get("/passwords/users_with_3_or_more")]
pub async fn get_users_with_3_or_more_passwords(pool: web::Data<SqlitePool>) -> impl Responder {
    let query_result: Result<Vec<sqlx::sqlite::SqliteRow>, sqlx::Error> = sqlx::query(
        r#"
        SELECT 
            u.user_id, 
            u.user_first_name, 
            u.user_last_name, 
            COUNT(p.password_id) AS password_count
        FROM users u
        JOIN passwords p ON u.user_id = p.user_id
        GROUP BY u.user_id
        HAVING COUNT(p.password_id) >= 3
        ORDER BY password_count DESC;
        "#
    )
    .fetch_all(&**pool)
    .await;

    match query_result {
        Ok(rows) => {
            let result: Vec<PasswordSummary> = rows.iter().map(|row| PasswordSummary {
                user_id: row.get("user_id"),
                user_first_name: row.get("user_first_name"),
                user_last_name: row.get("user_last_name"),
                password_count: row.get::<i64, _>("password_count"),
            }).collect();

            HttpResponse::Ok().json(result)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
