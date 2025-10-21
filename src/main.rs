use actix_web::{App, HttpServer, web};
use sqlx::sqlite::SqlitePoolOptions;

mod models;
mod controllers;
mod utils;
mod routes;

use routes::user_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // הגדרת מסלול לקובץ SQLite ישירות בקוד
    // let database_url = "C:\\Users\\1\\Documents\\GitHub\\PassMenSystem\\src\\passwords_management_system.db";
    let database_url = "sqlite://passwords_management_system.db";



    // יוצר חיבור מאגר עם מקסימום 5 חיבורים בו זמנית
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("❌ Failed to connect to database");

    println!("✅ Connected to database");
    println!("🚀 Server running at http://127.0.0.1:8080");

    // מריץ את השרת
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(user_routes::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
