use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::{SqlitePool, Row};
use crate::models::categories::{Category, CreateCategoryDto, UpdateCategoryDto};

// ===================== INIT DATABASE =====================
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // let database_url = "C:\\Users\\1\\Documents\\GitHub\\PassMenSystem\\src\\passwords_management_system.db";
    let database_url = "sqlite://src/passwords_management_system.db";
    let pool = SqlitePool::connect(database_url).await?;

    println!("Creating categories table if not exists...");
    sqlx::query(
        r#"
        CREATE TABLE categories (
        category_id INTEGER PRIMARY KEY AUTOINCREMENT,
        category_name TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;
    println!("categories table ready");
    Ok(pool)
}

// ===================== CREATE CATEGORY =====================
#[post("/categories")]
pub async fn create_category(pool: web::Data<SqlitePool>, category: web::Json<CreateCategoryDto>) -> impl Responder {
    match sqlx::query(
        "INSERT INTO categories (category_name) VALUES (?)"
    )
    .bind(&category.category_name)
    .execute(&**pool)
    .await
    {
        Ok(result) => {
            let new_category = Category {
                category_id: result.last_insert_rowid(),
                category_name: category.category_name.clone(),
            };
            HttpResponse::Created().json(new_category)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== READ ALL CATEGORIES =====================
#[get("/categories")]
pub async fn get_categories(pool: web::Data<SqlitePool>) -> impl Responder {
    match sqlx::query("SELECT * FROM categories ORDER BY category_id")
        .fetch_all(&**pool)
        .await
    {
        Ok(rows) => {
            let categories: Vec<Category> = rows.iter().map(|row| Category {
                category_id: row.get("category_id"),
                category_name: row.get("category_name"),
            }).collect();
            HttpResponse::Ok().json(categories)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== READ ONE CATEGORY =====================
#[get("/categories/{id}")]
pub async fn get_category(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let id = path.into_inner();

    match sqlx::query("SELECT * FROM categories WHERE category_id = ?")
        .bind(id)
        .fetch_optional(&**pool)
        .await
    {
        Ok(Some(row)) => {
            let category = Category {
                category_id: row.get("category_id"),
                category_name: row.get("category_name"),
            };
            HttpResponse::Ok().json(category)
        }
        Ok(None) => HttpResponse::NotFound().body("Category not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== UPDATE CATEGORY =====================
#[put("/categories/{id}")]
pub async fn update_category(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    updated: web::Json<UpdateCategoryDto>,
) -> impl Responder {
    let id = path.into_inner();

    match sqlx::query(
        "UPDATE categories SET category_name = COALESCE(?, category_name) WHERE category_id = ?"
    )
    .bind(&updated.category_name)
    .bind(id)
    .execute(&**pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().body("Category updated successfully")
            } else {
                HttpResponse::NotFound().body("Category not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// ===================== DELETE CATEGORY =====================
#[delete("/categories/{id}")]
pub async fn delete_category(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let id = path.into_inner();

    match sqlx::query("DELETE FROM categories WHERE category_id = ?")
        .bind(id)
        .execute(&**pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().body("Category deleted successfully")
            } else {
                HttpResponse::NotFound().body("Category not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
#[derive(serde::Serialize)]
pub struct CategorySearchResult {
    pub category_id: i64,
    pub category_name: String,
}

#[get("/categories/search/{keyword}")]
pub async fn search_categories(pool: web::Data<SqlitePool>, path: web::Path<String>) -> impl Responder {
    let keyword = path.into_inner();
    let pattern = format!("%{}%", keyword);

    // שמרנו על אותו סגנון כמו בשאר הפונקציות שלך:
    let query_result: Result<Vec<sqlx::sqlite::SqliteRow>, sqlx::Error> = sqlx::query(
        "SELECT * FROM categories WHERE category_name LIKE ?"
    )
    .bind(pattern)
    .fetch_all(&**pool)
    .await;

    match query_result {
        Ok(rows) => {
            // נמפה ידנית לשדה שיכול להיות Serialized ל-JSON
            let results: Vec<CategorySearchResult> = rows.iter().map(|row| CategorySearchResult {
                category_id: row.get("category_id"),
                category_name: row.get("category_name"),
            }).collect();

            HttpResponse::Ok().json(results)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
