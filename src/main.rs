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
    dotenv::dotenv().ok();
    env_logger::init();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let server_host = env::var("SERVER_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Invalid SERVER_PORT");
    
    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");
    
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&db_pool)
        .await
        .expect("Failed to enable foreign keys");
    
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");
    
    let app_state = web::Data::new(AppState {
        db: db_pool,
    });
    
    // Generate a secure random key if not provided in environment
    let key = if let Ok(key_str) = env::var("SESSION_KEY") {
        if key_str.len() < 64 {
            log::warn!("SESSION_KEY is too short. Using a randomly generated key.");
            Key::generate()
        } else {
            Key::from(key_str.as_bytes())
        }
    } else {
        log::warn!("SESSION_KEY not set. Using a randomly generated key. For production, set SESSION_KEY environment variable to a secure, random value.");
        Key::generate()
    };
    
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
            // Pages
            .route("/", web::get().to(handlers::index))
            .route("/store", web::get().to(handlers::store_page))
            .route("/admin", web::get().to(handlers::admin_page))
            .route("/cart", web::get().to(handlers::cart_page))
            // API Routes - Products
            .route("/api/products", web::get().to(handlers::products::get_products))
            .route("/api/products", web::post().to(handlers::products::create_product))
            .route("/api/products/search", web::get().to(handlers::products::search_products))
            .route("/api/products/{id}", web::get().to(handlers::products::get_product))
            .route("/api/products/{id}", web::put().to(handlers::products::update_product))
            .route("/api/products/{id}", web::delete().to(handlers::products::delete_product))
            // API Routes - Categories
            .route("/api/categories", web::get().to(handlers::categories::get_categories))
            .route("/api/categories", web::post().to(handlers::categories::create_category))
            .route("/api/categories/{id}", web::put().to(handlers::categories::update_category))
            .route("/api/categories/{id}", web::delete().to(handlers::categories::delete_category))
            .route("/api/categories/{id}/products", web::get().to(handlers::categories::get_category_products))
            // API Routes - Cart
            .route("/api/cart", web::get().to(handlers::cart::get_cart))
            .route("/api/cart", web::post().to(handlers::cart::add_to_cart))
            .route("/api/cart/clear", web::post().to(handlers::cart::clear_cart))
            .route("/api/cart/{id}", web::put().to(handlers::cart::update_cart_item))
            .route("/api/cart/{id}", web::delete().to(handlers::cart::remove_from_cart))
            // Add these routes after the cart routes in main.rs
            // API Routes - Orders
            .route("/api/orders", web::post().to(handlers::orders::create_order))
            .route("/api/orders", web::get().to(handlers::orders::get_orders))
            .route("/api/orders/{id}", web::get().to(handlers::orders::get_order))
    })
    .bind((server_host, server_port))?
    .run()
    .await
}