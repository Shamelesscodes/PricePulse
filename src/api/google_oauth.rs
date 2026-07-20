use crate::config::GoogleOAuthConfig;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl,
    Scope, TokenUrl, AuthorizationCode, TokenResponse,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub picture: Option<String>,
}

pub fn create_oauth_client(config: &GoogleOAuthConfig) -> Result<BasicClient, String> {
    let google_client_id = ClientId::new(config.client_id.clone());
    let google_client_secret = ClientSecret::new(config.client_secret.clone());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .map_err(|e| e.to_string())?;
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .map_err(|e| e.to_string())?;
    let redirect_url = RedirectUrl::new(config.redirect_url.clone())
        .map_err(|e| e.to_string())?;

    Ok(BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(redirect_url))
}

pub fn get_google_auth_url(config: &GoogleOAuthConfig) -> Result<String, String> {
    let client = create_oauth_client(config)?;
    let (authorize_url, _) = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("openid".to_string()))
        .url();

    Ok(authorize_url.to_string())
}

pub async fn exchange_code_for_google_user(
    config: &GoogleOAuthConfig,
    client: &reqwest::Client,
    code: &str,
) -> Result<GoogleUserInfo, String> {
    let oauth_client = create_oauth_client(config)?;

    let token_res = oauth_client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| format!("Failed to exchange Google OAuth code: {}", e))?;

    let access_token = token_res.access_token().secret();

    let user_info: GoogleUserInfo = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Google user profile: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse Google user profile JSON: {}", e))?;

    Ok(user_info)
}
