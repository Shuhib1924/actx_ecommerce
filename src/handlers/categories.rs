use actix_web::{web, HttpResponse};
use crate::{models::{Category, CreateCategory, Product}, errors::Result, AppState};

// Get all categories
pub async fn get_categories(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories ORDER BY name"
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(HttpResponse::Ok().json(categories))
}

// Get category with products
pub async fn get_category_products(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let category_id = path.into_inner();
    
    // Get category
    let category = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE id = ?1"
    )
    .bind(category_id)
    .fetch_optional(&state.db)
    .await?;
    
    match category {
        Some(cat) => {
            // Get products in this category
            let products = sqlx::query_as::<_, Product>(
                "SELECT * FROM products WHERE category_id = ?1 ORDER BY name"
            )
            .bind(category_id)
            .fetch_all(&state.db)
            .await?;
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "category": cat,
                "products": products
            })))
        },
        None => Err(crate::errors::AppError::NotFound),
    }
}

// Create category (admin)
pub async fn create_category(
    state: web::Data<AppState>,
    category: web::Json<CreateCategory>,
) -> Result<HttpResponse> {
    let category = category.into_inner();
    
    let result = sqlx::query_as::<_, Category>(
        r#"
        INSERT INTO categories (name, description)
        VALUES (?1, ?2)
        RETURNING *
        "#
    )
    .bind(&category.name)
    .bind(&category.description)
    .fetch_one(&state.db)
    .await?;
    
    Ok(HttpResponse::Created().json(result))
}

// Update category (admin)
pub async fn update_category(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    category: web::Json<CreateCategory>,
) -> Result<HttpResponse> {
    let category_id = path.into_inner();
    let category = category.into_inner();
    
    let result = sqlx::query_as::<_, Category>(
        r#"
        UPDATE categories 
        SET name = ?1, description = ?2, updated_at = datetime('now')
        WHERE id = ?3
        RETURNING *
        "#
    )
    .bind(&category.name)
    .bind(&category.description)
    .bind(category_id)
    .fetch_optional(&state.db)
    .await?;
    
    match result {
        Some(c) => Ok(HttpResponse::Ok().json(c)),
        None => Err(crate::errors::AppError::NotFound),
    }
}

// Delete category (admin)
pub async fn delete_category(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let category_id = path.into_inner();
    
    let result = sqlx::query("DELETE FROM categories WHERE id = ?1")
        .bind(category_id)
        .execute(&state.db)
        .await?;
    
    if result.rows_affected() > 0 {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(crate::errors::AppError::NotFound)
    }
}