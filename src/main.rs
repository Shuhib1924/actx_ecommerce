mod errors;
mod models;
mod handlers;

use actix_files::Files;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_web::cookie::Key;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

pub struct AppState {
    pub db: sqlx::SqlitePool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    env_logger::init();
    
    // Get configuration
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let server_host = env::var("SERVER_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Invalid SERVER_PORT");
    
    // Create database pool for SQLite
    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");
    
    // Enable foreign keys for SQLite
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&db_pool)
        .await
        .expect("Failed to enable foreign keys");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");
    
    let app_state = web::Data::new(AppState {
        db: db_pool,
    });
    
    // Session key handling
    let key = Key::generate();  // This will generate a secure random key
    
    log::info!("Starting server at http://{}:{}", server_host, server_port);
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    key.clone()
                )
                .cookie_secure(false)
                .build()
            )
            .service(Files::new("/static", "./static"))
            .route("/", web::get().to(handlers::index))
    })
    .bind((server_host, server_port))?
    .run()
    .await
}