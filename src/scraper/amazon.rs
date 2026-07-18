use crate::errors::PricePulseError;
use crate::scraper::r#trait::{PriceScraper, ScrapedProduct};
use crate::parser::product_parser::{extract_price_and_currency, parse_selectors, parse_meta_tag};
use scraper::Html;
use std::future::Future;
use std::pin::Pin;

pub struct AmazonScraper;

impl PriceScraper for AmazonScraper {
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
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
                .header("Referer", "https://www.google.com/")
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(PricePulseError::Scrape(format!("Amazon returned status code {}", response.status())));
            }

            let html_content = response.text().await?;
            let document = Html::parse_document(&html_content);

            // Try standard Amazon CSS selectors
            let title_selectors = &["#productTitle", ".qa-title-text"];
            let price_selectors = &[
                ".a-price-whole",
                ".a-offscreen",
                "#priceblock_ourprice",
                "#priceblock_dealprice",
                ".priceToPay",
                ".apexPriceToPay",
            ];

            let (title_opt, price_opt) = parse_selectors(&document, title_selectors, price_selectors);

            let title = title_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='og:title']"))
                .ok_or_else(|| PricePulseError::Scrape("Could not parse Amazon product title".to_string()))?;

            let price_raw = price_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='product:price:amount']"))
                .ok_or_else(|| {
                    if html_content.contains("Currently unavailable") || html_content.contains("Out of Stock") {
                        PricePulseError::Scrape("Product is currently unavailable or Out of Stock".to_string())
                    } else {
                        PricePulseError::Scrape("Could not parse Amazon price".to_string())
                    }
                })?;

            let (price, currency) = extract_price_and_currency(&price_raw)?;

            Ok(ScrapedProduct {
                title,
                price,
                currency,
            })
        })
    }
}
