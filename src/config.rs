use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub to_email: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    #[serde(default = "default_auto_start_scheduler")]
    pub auto_start_scheduler: bool,
}

fn default_auto_start_scheduler() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub check_interval: u64, // in minutes
    pub database: String,
    pub max_concurrent_requests: usize,
    pub user_agent: String,
    pub email: Option<EmailConfig>,
    pub api: Option<ApiConfig>,
    pub google_oauth: Option<GoogleOAuthConfig>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let primary_path = Path::new("config/config.toml");
        let fallback_path = Path::new("config/config.toml.example");

        let content = if primary_path.exists() {
            fs::read_to_string(primary_path)?
        } else if fallback_path.exists() {
            fs::read_to_string(fallback_path)?
        } else {
            return Err("Neither 'config/config.toml' nor 'config/config.toml.example' was found.".into());
        };

        let mut config: Config = toml::from_str(&content)?;

        // Override port from PORT env var (standard for Railway/Heroku/Render)
        if let Ok(port_str) = std::env::var("PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                if let Some(ref mut api) = config.api {
                    api.port = port;
                } else {
                    config.api = Some(ApiConfig {
                        host: "0.0.0.0".to_string(),
                        port,
                        jwt_secret: "pricepulse_secret_jwt_key_2026_dev_mode".to_string(),
                        auto_start_scheduler: true,
                    });
                }
            }
        }

        // Override JWT secret from JWT_SECRET env var
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            if let Some(ref mut api) = config.api {
                api.jwt_secret = jwt_secret;
            }
        }

        // Override Database path from DATABASE_PATH or DATABASE_URL env var
        if let Ok(db_url) = std::env::var("DATABASE_PATH").or_else(|_| std::env::var("DATABASE_URL")) {
            config.database = db_url;
        }

        // Override SMTP details from env vars
        if let Ok(pass) = std::env::var("SMTP_PASSWORD") {
            if let Some(ref mut email) = config.email {
                email.smtp_password = pass;
                email.enabled = true;
            }
        }
        if let Ok(user) = std::env::var("SMTP_USERNAME") {
            if let Some(ref mut email) = config.email {
                email.smtp_username = user.clone();
                if email.from_email.contains("your-email") {
                    email.from_email = user;
                }
            }
        }

        // Override Google OAuth from env vars
        if let Ok(client_id) = std::env::var("GOOGLE_CLIENT_ID") {
            if let Ok(client_secret) = std::env::var("GOOGLE_CLIENT_SECRET") {
                let redirect_url = std::env::var("GOOGLE_REDIRECT_URL")
                    .unwrap_or_else(|_| "http://localhost:3000/api/auth/google/callback".to_string());

                config.google_oauth = Some(GoogleOAuthConfig {
                    client_id,
                    client_secret,
                    redirect_url,
                });
            }
        }

        Ok(config)
    }
}
