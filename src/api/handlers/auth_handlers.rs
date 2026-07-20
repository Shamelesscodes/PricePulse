use crate::api::auth::{create_jwt, hash_password, verify_password, AuthUser};
use crate::api::dto::{
    AuthResponse, GoogleCallbackRequest, LoginRequest, RegisterRequest, UserResponse,
};
use crate::api::google_oauth::{exchange_code_for_google_user, get_google_auth_url};
use crate::api::router::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || payload.password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Email cannot be empty and password must be at least 6 characters" })),
        )
            .into_response();
    }

    if let Ok(Some(_)) = state.repo.find_user_by_email(&email).await {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "User with this email already exists" })),
        )
            .into_response();
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Password hashing failed: {}", e) })),
            )
                .into_response()
        }
    };

    let user = match state
        .repo
        .create_user(&email, Some(&password_hash), None, None)
        .await
    {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Database error: {}", e) })),
            )
                .into_response()
        }
    };

    let jwt_secret = state
        .config
        .api
        .as_ref()
        .map(|a| a.jwt_secret.as_str())
        .unwrap_or("pricepulse_secret_jwt_key_2026_dev_mode");

    let token = match create_jwt(user.id.unwrap(), &user.email, jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Token generation failed: {}", e) })),
            )
                .into_response()
        }
    };

    (
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: UserResponse::from(user),
        }),
    )
        .into_response()
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let email = payload.email.trim().to_lowercase();
    let user = match state.repo.find_user_by_email(&email).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid email or password" })),
            )
                .into_response();
        }
    };

    let password_hash = match user.password_hash.as_ref() {
        Some(h) => h,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "This account was registered using Google OAuth. Please login with Google." })),
            )
                .into_response();
        }
    };

    if !verify_password(&payload.password, password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid email or password" })),
        )
            .into_response();
    }

    let jwt_secret = state
        .config
        .api
        .as_ref()
        .map(|a| a.jwt_secret.as_str())
        .unwrap_or("pricepulse_secret_jwt_key_2026_dev_mode");

    let token = match create_jwt(user.id.unwrap(), &user.email, jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Token generation failed: {}", e) })),
            )
                .into_response()
        }
    };

    (
        StatusCode::OK,
        Json(AuthResponse {
            token,
            user: UserResponse::from(user),
        }),
    )
        .into_response()
}

pub async fn google_auth_url(State(state): State<AppState>) -> impl IntoResponse {
    let oauth_cfg = match state.config.google_oauth.as_ref() {
        Some(cfg) => cfg,
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(json!({ "error": "Google OAuth is not configured in config/config.toml" })),
            )
                .into_response()
        }
    };

    match get_google_auth_url(oauth_cfg) {
        Ok(url) => (StatusCode::OK, Json(json!({ "url": url }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to generate Google OAuth URL: {}", e) })),
        )
            .into_response(),
    }
}

pub async fn google_auth_callback(
    State(state): State<AppState>,
    Json(payload): Json<GoogleCallbackRequest>,
) -> impl IntoResponse {
    let oauth_cfg = match state.config.google_oauth.as_ref() {
        Some(cfg) => cfg,
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(json!({ "error": "Google OAuth is not configured in config/config.toml" })),
            )
                .into_response()
        }
    };

    let client = reqwest::Client::new();
    let google_user = match exchange_code_for_google_user(oauth_cfg, &client, &payload.code).await {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Google authentication failed: {}", e) })),
            )
                .into_response()
        }
    };

    // Find or create user by google_id / email
    let user = match state.repo.find_user_by_google_id(&google_user.id).await {
        Ok(Some(u)) => u,
        _ => match state.repo.find_user_by_email(&google_user.email).await {
            Ok(Some(u)) => u,
            _ => match state
                .repo
                .create_user(
                    &google_user.email,
                    None,
                    Some(&google_user.id),
                    google_user.picture.as_deref(),
                )
                .await
            {
                Ok(u) => u,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": format!("Failed to create Google user: {}", e) })),
                    )
                        .into_response()
                }
            },
        },
    };

    let jwt_secret = state
        .config
        .api
        .as_ref()
        .map(|a| a.jwt_secret.as_str())
        .unwrap_or("pricepulse_secret_jwt_key_2026_dev_mode");

    let token = match create_jwt(user.id.unwrap(), &user.email, jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Token generation failed: {}", e) })),
            )
                .into_response()
        }
    };

    (
        StatusCode::OK,
        Json(AuthResponse {
            token,
            user: UserResponse::from(user),
        }),
    )
        .into_response()
}

pub async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    match state.repo.find_user_by_id(auth.id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(UserResponse::from(user))).into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User profile not found" })),
        )
            .into_response(),
    }
}
