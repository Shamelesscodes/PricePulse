use crate::errors::PricePulseError;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct ScrapedProduct {
    pub title: String,
    pub price: f64,
    pub currency: String,
}

pub trait PriceScraper: Send + Sync {
    fn fetch<'a>(
        &'a self,
        client: &'a reqwest::Client,
        url: &'a str,
        user_agent: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ScrapedProduct, PricePulseError>> + Send + 'a>>;
}
