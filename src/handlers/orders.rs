use actix_session::Session;
use actix_web::{web, HttpResponse};
use crate::{
    models::Order, 
    errors::{Result, AppError}, 
    AppState,
    handlers::cart::{get_cart_from_session, save_cart_to_session}
};

#[derive(serde::Deserialize)]
pub struct CreateOrder {
    pub customer_name: String,
    pub customer_email: String,
    pub shipping_address: String,
}

pub async fn create_order(
    session: Session,
    state: web::Data<AppState>,
    order_data: web::Json<CreateOrder>,
) -> Result<HttpResponse> {
    let order_data = order_data.into_inner();
    let mut cart = get_cart_from_session(&session)?;
    
    // Load product details for cart
    let mut total_amount = 0.0;
    for item in &mut cart.items {
        let product = sqlx::query_as::<_, crate::models::Product>(
            "SELECT * FROM products WHERE id = ?1"
        )
        .bind(item.product_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
        
        // Check stock
        if product.stock_quantity < item.quantity {
            return Err(AppError::BadRequest(
                format!("Insufficient stock for {}", product.name)
            ));
        }
        
        total_amount += product.price * item.quantity as f64;
        item.product = Some(product);
    }
    
    // Begin transaction
    let mut tx = state.db.begin().await?;
    
    // Create order
    let order_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO orders (total_amount, customer_name, customer_email, shipping_address, status)
        VALUES (?1, ?2, ?3, ?4, 'pending')
        RETURNING id
        "#
    )
    .bind(total_amount)
    .bind(&order_data.customer_name)
    .bind(&order_data.customer_email)
    .bind(&order_data.shipping_address)
    .fetch_one(&mut *tx)
    .await?;
    
    // Create order items and update stock
    for item in &cart.items {
        let product = item.product.as_ref().unwrap();
        
        // Insert order item
        sqlx::query(
            r#"
            INSERT INTO order_items (order_id, product_id, quantity, price)
            VALUES (?1, ?2, ?3, ?4)
            "#
        )
        .bind(order_id)
        .bind(item.product_id)
        .bind(item.quantity)
        .bind(product.price)
        .execute(&mut *tx)
        .await?;
        
        // Update stock
        sqlx::query(
            "UPDATE products SET stock_quantity = stock_quantity - ?1 WHERE id = ?2"
        )
        .bind(item.quantity)
        .bind(item.product_id)
        .execute(&mut *tx)
        .await?;
    }
    
    // Commit transaction
    tx.commit().await?;
    
    // Clear cart
    cart.clear();
    save_cart_to_session(&session, &cart)?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Order created successfully",
        "order_id": order_id,
        "total": total_amount
    })))
}

pub async fn get_orders(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;
    
    Ok(HttpResponse::Ok().json(orders))
}

pub async fn get_order(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let order_id = path.into_inner();
    
    // Get order
    let order = sqlx::query_as!(
        Order,
        r#"
        SELECT 
            id as "id: i64",
            total_amount,
            status,
            customer_name,
            customer_email,
            shipping_address,
            created_at,
            updated_at
        FROM orders 
        WHERE id = ?1
        "#,
        order_id
    )
    .fetch_optional(&state.db)
    .await?;
    
    match order {
        Some(o) => {
            // Get order items
            #[derive(sqlx::FromRow, serde::Serialize)]
            struct OrderItem {
                id: i64,
                order_id: i64,
                product_id: i64,
                quantity: i32,
                price: f64,
                created_at: String,
                #[sqlx(rename = "name")]
                product_name: String
            }
            
            let items = sqlx::query_as::<_, OrderItem>(
                r#"
                SELECT oi.*, p.name
                FROM order_items oi
                JOIN products p ON oi.product_id = p.id
                WHERE oi.order_id = ?1
                "#
            )
            .bind(order_id)
            .fetch_all(&state.db)
            .await?;
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "order": o,
                "items": items
            })))
        },
        _ => Err(AppError::NotFound),
    }
}