use chrono::{DateTime, Utc, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Custom type for SQLite datetime handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteDateTime(pub DateTime<Utc>);

impl From<String> for SqliteDateTime {
    fn from(s: String) -> Self {
        let naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .unwrap_or_else(|_| Utc::now().naive_utc());
        SqliteDateTime(DateTime::from_naive_utc_and_offset(naive, Utc))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    #[sqlx(rename = "created_at")]
    pub created_at: String,
    #[sqlx(rename = "updated_at")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,  // SQLite uses REAL, which maps to f64
    pub stock_quantity: i32,
    pub category_id: Option<i32>,
    pub image_url: Option<String>,
    #[sqlx(rename = "created_at")]
    pub created_at: String,
    #[sqlx(rename = "updated_at")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: i32,
    pub quantity: i32,
    pub product: Option<Product>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub items: Vec<CartItem>,
}

#[allow(dead_code)]
impl Cart {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn add_item(&mut self, product_id: i32, quantity: i32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.product_id == product_id) {
            item.quantity += quantity;
        } else {
            self.items.push(CartItem {
                product_id,
                quantity,
                product: None,
            });
        }
    }
    
    pub fn remove_item(&mut self, product_id: i32) {
        self.items.retain(|i| i.product_id != product_id);
    }
    
    pub fn update_quantity(&mut self, product_id: i32, quantity: i32) {
        if quantity <= 0 {
            self.remove_item(product_id);
        } else if let Some(item) = self.items.iter_mut().find(|i| i.product_id == product_id) {
            item.quantity = quantity;
        }
    }
    
    pub fn clear(&mut self) {
        self.items.clear();
    }
    
    pub fn total_with_products(&self) -> f64 {
        self.items.iter()
            .filter_map(|item| {
                item.product.as_ref().map(|p| p.price * item.quantity as f64)
            })
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: i64,
    pub total_amount: f64,
    pub status: String,
    pub customer_name: String,
    pub customer_email: String,
    pub shipping_address: String,
    #[sqlx(rename = "created_at")]
    pub created_at: String,
    #[sqlx(rename = "updated_at")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub stock_quantity: i32,
    pub category_id: Option<i32>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    pub description: Option<String>,
}