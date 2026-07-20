pub mod r#trait;
pub mod amazon;
pub mod flipkart;
pub mod steam;
pub mod myntra;
pub mod nykaa;
pub mod savana;

pub use r#trait::PriceScraper;

use amazon::AmazonScraper;
use flipkart::FlipkartScraper;
use steam::SteamScraper;
use myntra::MyntraScraper;
use nykaa::NykaaScraper;
use savana::SavanaScraper;
use crate::errors::PricePulseError;

/// Dispatches the appropriate scraper based on the host domain of the URL.
/// Returns a Boxed PriceScraper trait object and the website name string.
pub fn get_scraper_for_url(url: &str) -> Result<(Box<dyn PriceScraper>, String), PricePulseError> {
    let lower_url = url.to_lowercase();
    if lower_url.contains("amazon.in") || lower_url.contains("amazon.com") {
        Ok((Box::new(AmazonScraper), "Amazon".to_string()))
    } else if lower_url.contains("flipkart.com") {
        Ok((Box::new(FlipkartScraper), "Flipkart".to_string()))
    } else if lower_url.contains("steampowered.com") || lower_url.contains("steamcommunity.com") {
        Ok((Box::new(SteamScraper), "Steam".to_string()))
    } else if lower_url.contains("myntra.com") {
        Ok((Box::new(MyntraScraper), "Myntra".to_string()))
    } else if lower_url.contains("nykaa.com") {
        Ok((Box::new(NykaaScraper), "Nykaa".to_string()))
    } else if lower_url.contains("savana.com") || lower_url.contains("savana.in") {
        Ok((Box::new(SavanaScraper), "Savana".to_string()))
    } else {
        Err(PricePulseError::Validation(format!(
            "Unsupported website URL: '{}'. We support Amazon, Flipkart, Steam, Myntra, Nykaa, and Savana.",
            url
        )))
    }
}
