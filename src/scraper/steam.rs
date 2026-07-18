use crate::errors::PricePulseError;
use crate::scraper::r#trait::{PriceScraper, ScrapedProduct};
use crate::parser::product_parser::{extract_price_and_currency, parse_selectors, parse_meta_tag};
use scraper::Html;
use regex::Regex;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub struct SteamScraper;

impl PriceScraper for SteamScraper {
    fn fetch<'a>(
        &'a self,
        client: &'a reqwest::Client,
        url: &'a str,
        user_agent: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ScrapedProduct, PricePulseError>> + Send + 'a>> {
        Box::pin(async move {
            // Attempt to extract App ID and use the official Steam API first (more stable than HTML)
            let app_id_regex = Regex::new(r"/app/(\d+)").unwrap();
            if let Some(caps) = app_id_regex.captures(url) {
                let app_id = caps.get(1).unwrap().as_str();
                let api_url = format!("https://store.steampowered.com/api/appdetails?appids={}&cc=in", app_id);

                if let Ok(response) = client.get(&api_url).header("User-Agent", user_agent).send().await {
                    if response.status().is_success() {
                        if let Ok(json_text) = response.text().await {
                            if let Ok(val) = serde_json::from_str::<Value>(&json_text) {
                                if val[app_id]["success"].as_bool().unwrap_or(false) {
                                    let data = &val[app_id]["data"];
                                    let title = data["name"].as_str().unwrap_or("Unknown Steam Game").to_string();
                                    
                                    if data["is_free"].as_bool().unwrap_or(false) {
                                        return Ok(ScrapedProduct {
                                            title,
                                            price: 0.0,
                                            currency: "₹".to_string(),
                                        });
                                    }

                                    if let Some(price_overview) = data["price_overview"].as_object() {
                                        let price_cents = price_overview.get("final").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let mut currency = price_overview.get("currency").and_then(|v| v.as_str()).unwrap_or("INR").to_string();
                                        
                                        if currency == "INR" {
                                            currency = "₹".to_string();
                                        } else if currency == "USD" {
                                            currency = "$".to_string();
                                        }

                                        return Ok(ScrapedProduct {
                                            title,
                                            price: price_cents / 100.0,
                                            currency,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Fallback to HTML Scraping if API fails or URL is custom
            let response = client.get(url)
                .header("User-Agent", user_agent)
                .header("Cookie", "wants_mature_content=1; birthtime=0; last_subid=0") // Bypass age gate
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(PricePulseError::Scrape(format!("Steam returned status code {}", response.status())));
            }

            let html_content = response.text().await?;
            let document = Html::parse_document(&html_content);

            let title_selectors = &[".apphub_AppName", "h1"];
            let price_selectors = &[
                ".discount_final_price",
                ".game_purchase_price",
                ".purchase_price",
                ".discount_original_price",
            ];

            let (title_opt, price_opt) = parse_selectors(&document, title_selectors, price_selectors);

            let title = title_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='og:title']"))
                .ok_or_else(|| PricePulseError::Scrape("Could not parse Steam game title".to_string()))?;

            let price_raw = price_opt
                .ok_or_else(|| PricePulseError::Scrape("Could not parse Steam price".to_string()))?;

            let (price, currency) = extract_price_and_currency(&price_raw)?;

            Ok(ScrapedProduct {
                title,
                price,
                currency,
            })
        })
    }
}
