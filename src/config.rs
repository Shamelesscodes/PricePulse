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
        let config_path = Path::new("config/config.toml");
        if !config_path.exists() {
            return Err("Configuration file 'config/config.toml' not found.".into());
        }
        let content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
