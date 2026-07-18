use crate::errors::PricePulseError;
use crate::scraper::r#trait::{PriceScraper, ScrapedProduct};
use crate::parser::product_parser::{extract_price_and_currency, parse_selectors, parse_meta_tag};
use scraper::Html;
use std::future::Future;
use std::pin::Pin;

pub struct NykaaScraper;

impl PriceScraper for NykaaScraper {
    fn fetch<'a>(
        &'a self,
        client: &'a reqwest::Client,
        url: &'a str,
        user_agent: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ScrapedProduct, PricePulseError>> + Send + 'a>> {
        Box::pin(async move {
            let response = client.get(url)
                .header("User-Agent", user_agent)
                .header("Accept-Language", "en-US,en;q=0.9")
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(PricePulseError::Scrape(format!("Nykaa returned status code {}", response.status())));
            }

            let html_content = response.text().await?;
            let document = Html::parse_document(&html_content);

            // Nykaa CSS selectors
            let title_selectors = &["h1.css-11277z2", ".css-1jczsfr", "h1"];
            let price_selectors = &[".css-1723zc5", ".css-11277z2", ".css-p6n771", ".css-iy0k72"];

            let (title_opt, price_opt) = parse_selectors(&document, title_selectors, price_selectors);

            let title = title_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='og:title']"))
                .ok_or_else(|| PricePulseError::Scrape("Could not parse Nykaa product title".to_string()))?;

            let price_raw = price_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='product:price:amount']"))
                .or_else(|| parse_meta_tag(&document, "meta[property='og:price:amount']"))
                .ok_or_else(|| PricePulseError::Scrape("Could not parse Nykaa price".to_string()))?;

            let (price, currency) = extract_price_and_currency(&price_raw)?;

            Ok(ScrapedProduct {
                title,
                price,
                currency,
            })
        })
    }
}
