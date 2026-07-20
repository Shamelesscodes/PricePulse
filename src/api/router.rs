use crate::api::handlers::{auth_handlers, product_handlers};
use crate::config::Config;
use crate::storage::Repository;
use axum::{
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub repo: Arc<Repository>,
}

pub fn create_router(config: Config, repo: Repository) -> Router {
    let state = AppState {
        config: Arc::new(config),
        repo: Arc::new(repo),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let auth_routes = Router::new()
        .route("/register", post(auth_handlers::register))
        .route("/login", post(auth_handlers::login))
        .route("/google/url", get(auth_handlers::google_auth_url))
        .route("/google/callback", post(auth_handlers::google_auth_callback))
        .route("/me", get(auth_handlers::get_me));

    let product_routes = Router::new()
        .route("/", get(product_handlers::list_products))
        .route("/", post(product_handlers::add_product))
        .route("/:id", get(product_handlers::get_product))
        .route("/:id", delete(product_handlers::delete_product))
        .route("/:id/status", patch(product_handlers::update_status))
        .route("/:id/target", patch(product_handlers::update_target_price))
        .route("/:id/check", post(product_handlers::check_product_price));

    Router::new()
        .route("/api/health", get(|| async { Json(json!({ "status": "ok", "service": "PricePulse API" })) }))
        .nest("/api/auth", auth_routes)
        .nest("/api/products", product_routes)
        .layer(cors)
        .with_state(state)
}
