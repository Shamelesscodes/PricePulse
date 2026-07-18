use std::fmt;

#[derive(Debug)]
pub enum PricePulseError {
    Database(sqlx::Error),
    Network(reqwest::Error),
    #[allow(dead_code)]
    Config(String),
    Io(std::io::Error),
    Scrape(String),
    Parser(String),
    Validation(String),
}

impl fmt::Display for PricePulseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PricePulseError::Database(e) => write!(f, "Database error: {}", e),
            PricePulseError::Network(e) => write!(f, "Network error: {}", e),
            PricePulseError::Config(s) => write!(f, "Configuration error: {}", s),
            PricePulseError::Io(e) => write!(f, "I/O error: {}", e),
            PricePulseError::Scrape(s) => write!(f, "Scrape error: {}", s),
            PricePulseError::Parser(s) => write!(f, "Parser error: {}", s),
            PricePulseError::Validation(s) => write!(f, "Validation error: {}", s),
        }
    }
}

impl std::error::Error for PricePulseError {}

impl From<sqlx::Error> for PricePulseError {
    fn from(err: sqlx::Error) -> Self {
        PricePulseError::Database(err)
    }
}

impl From<reqwest::Error> for PricePulseError {
    fn from(err: reqwest::Error) -> Self {
        PricePulseError::Network(err)
    }
}

impl From<std::io::Error> for PricePulseError {
    fn from(err: std::io::Error) -> Self {
        PricePulseError::Io(err)
    }
}
