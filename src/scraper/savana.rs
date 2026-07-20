use crate::errors::PricePulseError;
use crate::scraper::r#trait::{PriceScraper, ScrapedProduct};
use crate::parser::product_parser::{extract_price_and_currency, parse_selectors, parse_meta_tag};
use scraper::Html;
use std::future::Future;
use std::pin::Pin;

pub struct SavanaScraper;

impl PriceScraper for SavanaScraper {
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
                return Err(PricePulseError::Scrape(format!("Savana returned status code {}", response.status())));
            }

            let html_content = response.text().await?;

            // 1. Try parsing window.$preData JavaScript object
            if let Some(start_idx) = html_content.find("window.$preData = ") {
                let slice = &html_content[start_idx + 18..];
                let mut brace_count = 0;
                let mut end_idx = None;
                for (i, c) in slice.char_indices() {
                    if c == '{' {
                        brace_count += 1;
                    } else if c == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            end_idx = Some(i + 1);
                            break;
                        }
                    }
                }

                if let Some(end) = end_idx {
                    let raw_json = &slice[..end];
                    let cleaned_json = raw_json
                        .replace(":undefined", ":null")
                        .replace(": undefined", ": null");

                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cleaned_json) {
                        let detail_key = "/n/api/intention/item/v4/detail";
                        let trade_key = "/n/api/trade/intention/item/detail";
                        
                        let detail = val.get(detail_key).or_else(|| val.get(trade_key));
                        
                        if let Some(detail_val) = detail {
                            let goods_name = detail_val["goodsName"].as_str()
                                .or_else(|| detail_val["shortGoodsName"].as_str());

                            let price_val = detail_val["promotePrice"].as_f64()
                                .or_else(|| detail_val["promotePrice"].as_str().and_then(|s| s.parse::<f64>().ok()))
                                .or_else(|| detail_val["salesPrice"].as_f64())
                                .or_else(|| detail_val["salesPrice"].as_str().and_then(|s| s.parse::<f64>().ok()));

                            let currency_symbol = if let Some(txt) = detail_val["promotePriceText"].as_str() {
                                if txt.contains('₹') || txt.contains("\u{20b9}") {
                                    "₹".to_string()
                                } else if txt.contains('$') {
                                    "$".to_string()
                                } else if txt.contains('€') {
                                    "€".to_string()
                                } else {
                                    "₹".to_string()
                                }
                            } else {
                                "₹".to_string()
                            };

                            if let (Some(name), Some(price)) = (goods_name, price_val) {
                                return Ok(ScrapedProduct {
                                    title: name.to_string(),
                                    price,
                                    currency: currency_symbol,
                                });
                            }
                        }
                    }
                }
            }

            // 2. Fallback to parsing meta tags and standard HTML selectors
            let document = Html::parse_document(&html_content);

            let title_selectors = &[
                ".product-name",
                ".product-title",
                ".pdp-name",
                ".pdp-title",
                "h1"
            ];
            let price_selectors = &[
                ".product-price",
                ".price",
                ".pdp-price",
                ".discount-price"
            ];

            let (title_opt, price_opt) = parse_selectors(&document, title_selectors, price_selectors);

            let title = title_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='og:title']"))
                .or_else(|| parse_meta_tag(&document, "meta[name='twitter:title']"))
                .ok_or_else(|| PricePulseError::Scrape("Could not parse Savana product title".to_string()))?;

            let price_raw = price_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='product:price:amount']"))
                .or_else(|| parse_meta_tag(&document, "meta[property='og:price:amount']"))
                .or_else(|| parse_meta_tag(&document, "meta[property='product:sale_price:amount']"))
                .ok_or_else(|| PricePulseError::Scrape("Could not parse Savana price".to_string()))?;

            let (price, currency) = extract_price_and_currency(&price_raw)?;

            Ok(ScrapedProduct {
                title,
                price,
                currency,
            })
        })
    }
}
