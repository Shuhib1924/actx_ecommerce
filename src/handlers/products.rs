use actix_web::{web, HttpResponse};
use crate::{models::{Product, CreateProduct}, errors::Result, AppState};

// Get all products
pub async fn get_products(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let products = sqlx::query_as::<_, Product>(
        "SELECT * FROM products ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(HttpResponse::Ok().json(products))
}

// Get single product
pub async fn get_product(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let product_id = path.into_inner();
    
    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = ?1"
    )
    .bind(product_id)
    .fetch_optional(&state.db)
    .await?;
    
    match product {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        _ => Err(crate::errors::AppError::NotFound),
    }
}

// Create product (admin)
pub async fn create_product(
    state: web::Data<AppState>,
    product: web::Json<CreateProduct>,
) -> Result<HttpResponse> {
    let product = product.into_inner();
    
    let result = sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO products (name, description, price, stock_quantity, category_id, image_url)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        RETURNING *
        "#
    )
    .bind(&product.name)
    .bind(&product.description)
    .bind(product.price)
    .bind(product.stock_quantity)
    .bind(product.category_id)
    .bind(&product.image_url)
    .fetch_one(&state.db)
    .await?;
    
    Ok(HttpResponse::Created().json(result))
}

// Update product (admin)
pub async fn update_product(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    product: web::Json<CreateProduct>,
) -> Result<HttpResponse> {
    let product_id = path.into_inner();
    let product = product.into_inner();
    
    let result = sqlx::query_as::<_, Product>(
        r#"
        UPDATE products 
        SET name = ?1, description = ?2, price = ?3, 
            stock_quantity = ?4, category_id = ?5, image_url = ?6,
            updated_at = datetime('now')
        WHERE id = ?7
        RETURNING *
        "#
    )
    .bind(&product.name)
    .bind(&product.description)
    .bind(product.price)
    .bind(product.stock_quantity)
    .bind(product.category_id)
    .bind(&product.image_url)
    .bind(product_id)
    .fetch_optional(&state.db)
    .await?;
    
    match result {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        _ => Err(crate::errors::AppError::NotFound),
    }
}

// Delete product (admin)
pub async fn delete_product(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let product_id = path.into_inner();
    
    let result = sqlx::query("DELETE FROM products WHERE id = ?1")
        .bind(product_id)
        .execute(&state.db)
        .await?;
    
    if result.rows_affected() > 0 {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(crate::errors::AppError::NotFound)
    }
}

// Search products
pub async fn search_products(
    state: web::Data<AppState>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse> {
    let search_term = format!("%{}%", query.q.trim());
    
    let products = sqlx::query_as::<_, Product>(
        r#"
        SELECT * FROM products 
        WHERE name LIKE ?1 OR description LIKE ?1
        ORDER BY created_at DESC
        "#
    )
    .bind(&search_term)
    .fetch_all(&state.db)
    .await?;
    
    Ok(HttpResponse::Ok().json(products))
}

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    pub q: String,
}