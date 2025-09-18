use actix_session::Session;
use actix_web::{web, HttpResponse};
use crate::{models::{Cart, Product}, errors::{Result, AppError}, AppState};

// Get cart
pub async fn get_cart(
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let mut cart = get_cart_from_session(&session)?;
    
    // Load product details for each cart item
    for item in &mut cart.items {
        if let Some(product) = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE id = ?1"
        )
        .bind(item.product_id)
        .fetch_optional(&state.db)
        .await? {
            item.product = Some(product);
        }
    }
    
    Ok(HttpResponse::Ok().json(cart))
}

// Add item to cart
pub async fn add_to_cart(
    session: Session,
    state: web::Data<AppState>,
    item: web::Json<AddCartItem>,
) -> Result<HttpResponse> {
    let item = item.into_inner();
    
    // Verify product exists and has stock
    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = ?1"
    )
    .bind(item.product_id)
    .fetch_optional(&state.db)
    .await?;
    
    match product {
        Some(p) if p.stock_quantity >= item.quantity => {
            let mut cart = get_cart_from_session(&session)?;
            cart.add_item(item.product_id, item.quantity);
            save_cart_to_session(&session, &cart)?;
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Item added to cart",
                "cart": cart
            })))
        },
        Some(_) => Err(AppError::BadRequest("Insufficient stock".to_string())),
        _ => Err(AppError::NotFound),
    }
}

// Update cart item quantity
pub async fn update_cart_item(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<i32>,
    update: web::Json<UpdateCartItem>,
) -> Result<HttpResponse> {
    let product_id = path.into_inner();
    let new_quantity = update.into_inner().quantity;
    
    if new_quantity > 0 {
        // Verify stock
        let product = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE id = ?1"
        )
        .bind(product_id)
        .fetch_optional(&state.db)
        .await?;
        
        match product {
            Some(p) if p.stock_quantity >= new_quantity => {
                let mut cart = get_cart_from_session(&session)?;
                cart.update_quantity(product_id, new_quantity);
                save_cart_to_session(&session, &cart)?;
                Ok(HttpResponse::Ok().json(cart))
            },
            Some(_) => Err(AppError::BadRequest("Insufficient stock".to_string())),
            _ => Err(AppError::NotFound),
        }
    } else {
        // Remove item if quantity is 0 or less
        let mut cart = get_cart_from_session(&session)?;
        cart.remove_item(product_id);
        save_cart_to_session(&session, &cart)?;
        Ok(HttpResponse::Ok().json(cart))
    }
}

// Remove item from cart
pub async fn remove_from_cart(
    session: Session,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let product_id = path.into_inner();
    
    let mut cart = get_cart_from_session(&session)?;
    cart.remove_item(product_id);
    save_cart_to_session(&session, &cart)?;
    
    Ok(HttpResponse::Ok().json(cart))
}

// Clear cart
pub async fn clear_cart(session: Session) -> Result<HttpResponse> {
    let cart = Cart::new();
    save_cart_to_session(&session, &cart)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Cart cleared"
    })))
}

// Helper functions
pub fn get_cart_from_session(session: &Session) -> Result<Cart> {
    Ok(session.get::<Cart>("cart")
        .map_err(|_| AppError::SessionError)?
        .unwrap_or_else(Cart::new))
}

pub fn save_cart_to_session(session: &Session, cart: &Cart) -> Result<()> {
    session.insert("cart", cart)
        .map_err(|_| AppError::SessionError)?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct AddCartItem {
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(serde::Deserialize)]
pub struct UpdateCartItem {
    pub quantity: i32,
}