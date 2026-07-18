use crate::errors::PricePulseError;
use crate::scraper::r#trait::{PriceScraper, ScrapedProduct};
use crate::parser::product_parser::{extract_price_and_currency, parse_selectors, parse_meta_tag};
use scraper::Html;
use std::future::Future;
use std::pin::Pin;

pub struct FlipkartScraper;

impl PriceScraper for FlipkartScraper {
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
                return Err(PricePulseError::Scrape(format!("Flipkart returned status code {}", response.status())));
            }

            let html_content = response.text().await?;
            let document = Html::parse_document(&html_content);

            // Standard Flipkart selectors
            let title_selectors = &[".B_NuCI", ".VU-ZEg", "h1"];
            let price_selectors = &["._30jeq3", "._16Jk6d", ".yKfJKb", ".Nx93y"];

            let (title_opt, price_opt) = parse_selectors(&document, title_selectors, price_selectors);

            let title = title_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='og:title']"))
                .ok_or_else(|| PricePulseError::Scrape("Could not parse Flipkart title".to_string()))?;

            let price_raw = price_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='product:price:amount']"))
                .ok_or_else(|| PricePulseError::Scrape("Could not parse Flipkart price".to_string()))?;

            let (price, currency) = extract_price_and_currency(&price_raw)?;

            Ok(ScrapedProduct {
                title,
                price,
                currency,
            })
        })
    }
}
