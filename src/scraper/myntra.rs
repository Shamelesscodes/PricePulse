use crate::errors::PricePulseError;
use crate::scraper::r#trait::{PriceScraper, ScrapedProduct};
use crate::parser::product_parser::{extract_price_and_currency, parse_selectors, parse_meta_tag};
use scraper::Html;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub struct MyntraScraper;

impl PriceScraper for MyntraScraper {
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
                return Err(PricePulseError::Scrape(format!("Myntra returned status code {}", response.status())));
            }

            let html_content = response.text().await?;
            let document = Html::parse_document(&html_content);

            // Try standard selectors first
            let title_selectors = &[".pdp-title", ".pdp-name", "h1"];
            let price_selectors = &[".pdp-price", "strong.pdp-price", ".pdp-discounted-price"];

            let (title_opt, price_opt) = parse_selectors(&document, title_selectors, price_selectors);

            let title = title_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='og:title']"))
                .or_else(|| parse_meta_tag(&document, "meta[name='twitter:title']"));

            let price_raw = price_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='product:price:amount']"))
                .or_else(|| parse_meta_tag(&document, "meta[property='og:price:amount']"));

            // If we found both, return them
            if let (Some(t), Some(p)) = (title, price_raw.clone()) {
                let (price, currency) = extract_price_and_currency(&p)?;
                return Ok(ScrapedProduct {
                    title: t,
                    price,
                    currency,
                });
            }

            // Second fallback: Extract from window.__myx JavaScript object
            if let Some(start_idx) = html_content.find("window.__myx =") {
                let slice = &html_content[start_idx + 14..];
                if let Some(end_idx) = slice.find("</script>") {
                    let mut json_str = slice[..end_idx].trim().to_string();
                    if json_str.ends_with(';') {
                        json_str.pop();
                    }

                    if let Ok(val) = serde_json::from_str::<Value>(&json_str) {
                        let name = val["pdpData"]["name"].as_str().map(|s| s.to_string());
                        let brand = val["pdpData"]["brand"].as_str().unwrap_or("").to_string();
                        let price_val = val["pdpData"]["price"]["discounted"].as_f64();

                        if let (Some(n), Some(p)) = (name, price_val) {
                            let full_title = if brand.is_empty() { n } else { format!("{} - {}", brand, n) };
                            return Ok(ScrapedProduct {
                                title: full_title,
                                price: p,
                                currency: "₹".to_string(),
                            });
                        }
                    }
                }
            }

            Err(PricePulseError::Scrape("Could not parse Myntra page".to_string()))
        })
    }
}
