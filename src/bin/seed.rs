use sqlx::SqlitePool;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")?;
    let pool = SqlitePool::connect(&database_url).await?;
    
    println!("Seeding database...");
    
    // Insert categories
    let electronics_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO categories (name, description) VALUES (?1, ?2) RETURNING id"
    )
    .bind("Electronics")
    .bind("Electronic devices and gadgets")
    .fetch_one(&pool)
    .await?;
    
    let clothing_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO categories (name, description) VALUES (?1, ?2) RETURNING id"
    )
    .bind("Clothing")
    .bind("Fashion and apparel")
    .fetch_one(&pool)
    .await?;
    
    let books_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO categories (name, description) VALUES (?1, ?2) RETURNING id"
    )
    .bind("Books")
    .bind("Books and literature")
    .fetch_one(&pool)
    .await?;
    
    // Insert products
    let products = vec![
        ("Laptop", "High-performance laptop", 999.99, 10, electronics_id),
        ("Smartphone", "Latest smartphone model", 699.99, 15, electronics_id),
        ("Headphones", "Wireless noise-canceling headphones", 199.99, 20, electronics_id),
        ("T-Shirt", "Comfortable cotton t-shirt", 29.99, 50, clothing_id),
        ("Jeans", "Classic denim jeans", 79.99, 30, clothing_id),
        ("Sneakers", "Comfortable running shoes", 89.99, 25, clothing_id),
        ("Programming Book", "Learn Rust programming", 49.99, 40, books_id),
        ("Novel", "Bestselling fiction novel", 24.99, 35, books_id),
        ("Cookbook", "Delicious recipes from around the world", 34.99, 20, books_id),
    ];
    
    for (name, desc, price, stock, category_id) in products {
        sqlx::query(
            "INSERT INTO products (name, description, price, stock_quantity, category_id) VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(name)
        .bind(desc)
        .bind(price)
        .bind(stock)
        .bind(category_id)
        .execute(&pool)
        .await?;
        
        println!("Added product: {}", name);
    }
    
    println!("Database seeded successfully!");
    
    Ok(())
}