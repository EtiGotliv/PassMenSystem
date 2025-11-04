// use actix_web::{get, post, web, HttpResponse, Responder};
// use sqlx::{SqlitePool, Row};
// use chrono::Utc;
// use crate::models::password_history::{PasswordHistory, CreatePasswordHistoryDto};

// // שמירה מתוך קוד פנימי (למשל מ-update)
// pub async fn create_password_history_internal(
//     pool: &SqlitePool,
//     password_id: i64,
//     old_password_encrypted: &str,
// ) -> Result<PasswordHistory, sqlx::Error> {
//     let now = Utc::now().naive_utc();

//     let result = sqlx::query(
//         "INSERT INTO password_history (password_id, old_password_encrypted, changed_at)
//          VALUES (?, ?, ?)"
//     )
//     .bind(password_id)
//     .bind(old_password_encrypted)
//     .bind(now)
//     .execute(pool)
//     .await?;

//     Ok(PasswordHistory {
//         history_id: result.last_insert_rowid(),
//         password_id,
//         old_password_encrypted: old_password_encrypted.to_string(),
//         changed_at: now,
//     })
// }

// // שמירה גם דרך בקשת POST (לבדיקה ידנית ב־Postman)
// #[post("/password_history")]
// pub async fn create_password_history(
//     pool: web::Data<SqlitePool>,
//     item: web::Json<CreatePasswordHistoryDto>,
// ) -> impl Responder {
//     match create_password_history_internal(&**pool, item.password_id, &item.old_password_encrypted).await {
//         Ok(history) => HttpResponse::Created().json(history),
//         Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
//     }
// }

// // שליפה של כל ההיסטוריה
// #[get("/password_history")]
// pub async fn get_password_history(pool: web::Data<SqlitePool>) -> impl Responder {
//     match sqlx::query("SELECT * FROM password_history ORDER BY changed_at DESC")
//         .fetch_all(&**pool)
//         .await
//     {
//         Ok(rows) => {
//             let history: Vec<PasswordHistory> = rows.iter().map(|row| PasswordHistory {
//                 history_id: row.get("history_id"),
//                 password_id: row.get("password_id"),
//                 old_password_encrypted: row.get("old_password_encrypted"),
//                 changed_at: row.get("changed_at"),
//             }).collect();
//             HttpResponse::Ok().json(history)
//         }
//         Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
//     }
// }

// #[get("/password_history/most_changed_domain")]
// pub async fn get_most_changed_domain(pool: web::Data<SqlitePool>) -> impl Responder {
//     match sqlx::query(
//         r#"
//         SELECT p.domain, COUNT(ph.history_id) AS change_count
//         FROM passwords p
//         JOIN password_history ph ON p.password_id = ph.password_id
//         GROUP BY p.domain
//         ORDER BY change_count DESC
//         LIMIT 1;
//         "#
//     )
//     .fetch_one(&**pool)
//     .await
//     {
//         Ok(row) => {
//             let domain: String = row.get("domain");
//             let count: i64 = row.get("change_count");
//             HttpResponse::Ok().body(format!("Most changed domain: {} ({} changes)", domain, count))
//         }
//         Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
//     }
// }
use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::{SqlitePool, Row};
use chrono::Utc;
use crate::models::password_history::{PasswordHistory, CreatePasswordHistoryDto};

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

/// פונקציה פנימית לשמירת היסטוריה (למשל מעדכון או מחיקה)
// pub async fn create_password_history_internal(
//     pool: &SqlitePool,
//     password_id: i64,
//     old_password_encrypted: &str,
// ) -> Result<PasswordHistory, sqlx::Error> {
//     let now = Utc::now().naive_utc();

//     // הכנסת השורה לטבלת היסטוריה
//     let result = sqlx::query(
//         "INSERT INTO password_history (password_id, old_password_encrypted, changed_at)
//          VALUES (?, ?, ?)"
//     )
//     .bind(password_id)
//     .bind(old_password_encrypted)
//     .bind(now)
//     .execute(pool)
//     .await?;

//     println!("Inserted password history for password_id {}", password_id); // לוג לווידוא

//     Ok(PasswordHistory {
//         history_id: result.last_insert_rowid(),
//         password_id,
//         old_password_encrypted: old_password_encrypted.to_string(),
//         changed_at: now,
//     })
// }
pub async fn create_password_history_internal(
    pool: &SqlitePool,
    password_id: i64,
    old_password_encrypted: &str,
) -> Result<PasswordHistory, sqlx::Error> {
    let now = Utc::now().naive_utc();

    // בדיקת דיבאג לראות מה באמת הגיע
    println!("📜 Saving to history -> password_id={}, old_password_encrypted={}", password_id, old_password_encrypted);

    // נוודא שלא נשמר ID ריק או 0 בטעות
    if password_id <= 0 {
        eprintln!("⚠️ Warning: Tried to insert history with invalid password_id={}", password_id);
        return Err(sqlx::Error::Protocol(
            "Invalid password_id (0 or negative) when saving history".into(),
        ));
    }

    // הכנסת הנתונים עם RETURNING כדי לקבל את ה-id החדש
    let row = sqlx::query(
        r#"
        INSERT INTO password_history (password_id, old_password_encrypted, changed_at)
        VALUES (?, ?, ?)
        RETURNING history_id;
        "#
    )
    .bind(password_id)
    .bind(old_password_encrypted)
    .bind(now)
    .fetch_one(pool)
    .await?;

    let history_id: i64 = row.get("history_id");

    println!("✅ History entry created successfully -> history_id={}, password_id={}", history_id, password_id);

    Ok(PasswordHistory {
        history_id,
        password_id,
        old_password_encrypted: old_password_encrypted.to_string(),
        changed_at: now,
    })
}


/// יצירת היסטוריה דרך בקשת POST (לבדיקה ידנית)
#[post("/password_history")]
pub async fn create_password_history(
    pool: web::Data<SqlitePool>,
    item: web::Json<CreatePasswordHistoryDto>,
) -> impl Responder {
    match create_password_history_internal(&**pool, item.password_id, &item.old_password_encrypted).await {
        Ok(history) => HttpResponse::Created().json(history),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

/// שליפה של כל ההיסטוריה
// #[get("/password_history")]
// pub async fn get_password_history(pool: web::Data<SqlitePool>) -> impl Responder {
//     match sqlx::query("SELECT * FROM password_history ORDER BY changed_at DESC")
//         .fetch_all(&**pool)
//         .await
//     {
//         Ok(rows) => {
//             let history: Vec<PasswordHistory> = rows.iter().map(|row| PasswordHistory {
//                 history_id: row.get("history_id"),
//                 password_id: row.get("password_id"),
//                 old_password_encrypted: row.get("old_password_encrypted"),
//                 changed_at: row.get("changed_at"),
//             }).collect();
//             HttpResponse::Ok().json(history)
//         }
//         Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
//     }
// }



#[get("/password_history")]
pub async fn get_password_history(pool: web::Data<SqlitePool>) -> impl Responder {
    match sqlx::query(
        r#"
        SELECT DISTINCT history_id, password_id, old_password_encrypted, changed_at
        FROM password_history
        ORDER BY changed_at DESC
        "#
    )
    .fetch_all(&**pool)
    .await
    {
        Ok(rows) => {
            let history: Vec<PasswordHistory> = rows.iter().map(|row| PasswordHistory {
                history_id: row.get("history_id"),
                password_id: row.get("password_id"),
                old_password_encrypted: row.get("old_password_encrypted"),
                changed_at: row.get("changed_at"),
            }).collect();
            HttpResponse::Ok().json(history)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}



/// שליפת הדומיין ששינו לו סיסמה הכי הרבה פעמים הכי הרבה
#[get("/password_history/most_changed_domain")]
pub async fn get_most_changed_domain(pool: web::Data<SqlitePool>) -> impl Responder {
    match sqlx::query(
        r#"
        SELECT p.domain, COUNT(ph.history_id) AS change_count
        FROM passwords p
        JOIN password_history ph ON p.password_id = ph.password_id
        GROUP BY p.domain
        ORDER BY change_count DESC
        LIMIT 1;
        "#
    )
    .fetch_one(&**pool)
    .await
    {
        Ok(row) => {
            let domain: String = row.get("domain");
            let count: i64 = row.get("change_count");
            HttpResponse::Ok().body(format!("Most changed domain: {} ({} changes)", domain, count))
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
