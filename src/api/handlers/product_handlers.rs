use crate::api::auth::AuthUser;
use crate::api::dto::{
    AddProductRequest, ProductDetailResponse, ProductResponse, UpdateStatusRequest,
    UpdateTargetRequest,
};
use crate::api::router::AppState;
use crate::scraper::get_scraper_for_url;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

pub async fn list_products(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let products = match state.repo.list_products_by_user(auth.id).await {
        Ok(list) => list,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Database error: {}", e) })),
            )
                .into_response();
        }
    };

    let mut response_list = Vec::new();
    for p in products {
        let history = state
            .repo
            .get_price_history(p.id.unwrap_or(0))
            .await
            .unwrap_or_default();
        response_list.push(ProductResponse::from_product(p, history.first()));
    }

    (StatusCode::OK, Json(response_list)).into_response()
}

pub async fn get_product(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let product = match state.repo.get_user_product_by_id(auth.id, id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Product with ID {} not found", id) })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Database error: {}", e) })),
            )
                .into_response()
        }
    };

    let history = state
        .repo
        .get_price_history(id)
        .await
        .unwrap_or_default();

    let product_resp = ProductResponse::from_product(product, history.first());

    (
        StatusCode::OK,
        Json(ProductDetailResponse {
            product: product_resp,
            history,
        }),
    )
        .into_response()
}

pub async fn add_product(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<AddProductRequest>,
) -> impl IntoResponse {
    let url = payload.url.trim();
    if url.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "URL cannot be empty" })),
        )
            .into_response();
    }

    // Check if URL is already monitored for this user
    if let Ok(Some(existing)) = state.repo.get_product_by_url(url).await {
        if existing.user_id == Some(auth.id) {
            return (
                StatusCode::CONFLICT,
                Json(json!({ "error": "Product URL is already monitored in your account" })),
            )
                .into_response();
        }
    }

    let (scraper, website) = match get_scraper_for_url(url) {
        Ok(pair) => pair,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Unsupported URL: {}", e) })),
            )
                .into_response()
        }
    };

    let client = reqwest::Client::new();
    let user_agent = &state.config.user_agent;

    let scraped = match scraper.fetch(&client, url, user_agent).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Failed to scrape product page: {}", e) })),
            )
                .into_response()
        }
    };

    let product = match state
        .repo
        .add_product(Some(auth.id), &scraped.title, url, &website, payload.target_price)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Database error saving product: {}", e) })),
            )
                .into_response()
        }
    };

    let pid = product.id.unwrap();
    let history = match state.repo.add_price_history(pid, scraped.price, &scraped.currency).await {
        Ok(h) => Some(h),
        Err(_) => None,
    };

    (
        StatusCode::CREATED,
        Json(ProductResponse::from_product(product, history.as_ref())),
    )
        .into_response()
}

pub async fn delete_product(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.repo.remove_user_product(auth.id, id).await {
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Product with ID {} not found", id) })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": format!("Product ID {} successfully removed", id) })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
            .into_response(),
    }
}

pub async fn update_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    match state
        .repo
        .set_user_product_active_status(auth.id, id, payload.active)
        .await
    {
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Product with ID {} not found", id) })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": format!("Product ID {} active status updated to {}", id, payload.active) })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
            .into_response(),
    }
}

pub async fn update_target_price(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTargetRequest>,
) -> impl IntoResponse {
    match state
        .repo
        .set_user_product_target_price(auth.id, id, payload.target_price)
        .await
    {
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Product with ID {} not found", id) })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": format!("Product ID {} target price updated", id) })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
            .into_response(),
    }
}

pub async fn check_product_price(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let product = match state.repo.get_user_product_by_id(auth.id, id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Product with ID {} not found", id) })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Database error: {}", e) })),
            )
                .into_response()
        }
    };

    let (scraper, _) = match get_scraper_for_url(&product.url) {
        Ok(pair) => pair,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Scraper dispatch failed: {}", e) })),
            )
                .into_response()
        }
    };

    let client = reqwest::Client::new();
    let user_agent = &state.config.user_agent;

    let scraped = match scraper.fetch(&client, &product.url, user_agent).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("Scraping product failed: {}", e) })),
            )
                .into_response()
        }
    };

    let history = match state.repo.add_price_history(id, scraped.price, &scraped.currency).await {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to record price history: {}", e) })),
            )
                .into_response()
        }
    };

    (
        StatusCode::OK,
        Json(json!({
            "message": "Price check completed successfully",
            "price": history.price,
            "currency": history.currency,
            "timestamp": history.timestamp.to_rfc3339()
        })),
    )
        .into_response()
}
