use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub check_interval: u64, // in minutes
    pub database: String,
    pub max_concurrent_requests: usize,
    pub user_agent: String,
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
