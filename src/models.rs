use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub email: String,
    pub password_hash: Option<String>,
    pub google_id: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub title: String,
    pub url: String,
    pub website: String,
    pub target_price: Option<f64>,
    pub active: bool, // sqlx automatically maps INTEGER 0/1 to bool
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PriceHistory {
    pub id: Option<i64>,
    pub product_id: i64,
    pub price: f64,
    pub currency: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: Option<i64>,
    pub product_id: i64,
    pub trigger_price: f64,
    pub sent_at: DateTime<Utc>,
}
