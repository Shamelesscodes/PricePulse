use crate::models::{PriceHistory, Product, User};
use serde::{Deserialize, Serialize};

// --- AUTH DTOS ---

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleCallbackRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub avatar_url: Option<String>,
    pub created_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.unwrap_or(0),
            email: user.email,
            avatar_url: user.avatar_url,
            created_at: user.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

// --- PRODUCT DTOS ---

#[derive(Debug, Deserialize)]
pub struct AddProductRequest {
    pub url: String,
    pub target_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTargetRequest {
    pub target_price: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub website: String,
    pub current_price: Option<f64>,
    pub currency: Option<String>,
    pub target_price: Option<f64>,
    pub active: bool,
    pub created_at: String,
}

impl ProductResponse {
    pub fn from_product(p: Product, latest_history: Option<&PriceHistory>) -> Self {
        Self {
            id: p.id.unwrap_or(0),
            title: p.title,
            url: p.url,
            website: p.website,
            current_price: latest_history.map(|h| h.price),
            currency: latest_history.map(|h| h.currency.clone()),
            target_price: p.target_price,
            active: p.active,
            created_at: p.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProductDetailResponse {
    pub product: ProductResponse,
    pub history: Vec<PriceHistory>,
}
