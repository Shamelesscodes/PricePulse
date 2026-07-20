use crate::errors::PricePulseError;
use crate::scraper::r#trait::{PriceScraper, ScrapedProduct};
use crate::parser::product_parser::{extract_price_and_currency, parse_selectors, parse_meta_tag};
use scraper::Html;
use regex::Regex;
use std::future::Future;
use std::pin::Pin;

pub struct AmazonScraper;

fn normalize_amazon_url(url: &str) -> String {
    if let Ok(re) = Regex::new(r"/(?:dp|gp/product)/([A-Z0-9]{10})") {
        if let Some(caps) = re.captures(url) {
            if let Some(asin) = caps.get(1) {
                let domain = if url.contains("amazon.in") {
                    "www.amazon.in"
                } else if url.contains("amazon.co.uk") {
                    "www.amazon.co.uk"
                } else if url.contains("amazon.de") {
                    "www.amazon.de"
                } else {
                    "www.amazon.com"
                };
                return format!("https://{}/dp/{}", domain, asin.as_str());
            }
        }
    }
    url.to_string()
}

async fn fetch_html_with_curl(url: &str, user_agent: &str) -> Option<String> {
    let output = tokio::process::Command::new("curl.exe")
        .arg("-s")
        .arg("-L")
        .arg("-H")
        .arg(format!("User-Agent: {}", user_agent))
        .arg("-H")
        .arg("Accept-Language: en-US,en;q=0.9")
        .arg("-H")
        .arg("Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
        .arg(url)
        .output()
        .await
        .ok()?;

    if output.status.success() && !output.stdout.is_empty() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

impl PriceScraper for AmazonScraper {
    fn fetch<'a>(
        &'a self,
        client: &'a reqwest::Client,
        url: &'a str,
        user_agent: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ScrapedProduct, PricePulseError>> + Send + 'a>> {
        Box::pin(async move {
            let target_url = normalize_amazon_url(url);
            let effective_user_agent = if user_agent.contains("PricePulse") || user_agent.is_empty() {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"
            } else {
                user_agent
            };

            let mut html_content = match client.get(&target_url)
                .header("User-Agent", effective_user_agent)
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
                .header("Accept-Language", "en-US,en;q=0.9")
                .header("Sec-Ch-Ua", "\"Not/A)Brand\";v=\"8\", \"Chromium\";v=\"126\", \"Google Chrome\";v=\"126\"")
                .header("Sec-Ch-Ua-Mobile", "?0")
                .header("Sec-Ch-Ua-Platform", "\"Windows\"")
                .header("Sec-Fetch-Dest", "document")
                .header("Sec-Fetch-Mode", "navigate")
                .header("Sec-Fetch-Site", "none")
                .header("Sec-Fetch-User", "?1")
                .header("Upgrade-Insecure-Requests", "1")
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
                _ => String::new(),
            };

            // Try standard Amazon CSS selectors
            let title_selectors = &[
                "#productTitle",
                "#title",
                "h1 span#productTitle",
                "#titleSection",
                "#productTitle_feature_div",
                ".qa-title-text",
            ];
            let price_selectors = &[
                "#corePriceDisplay_desktop_feature_div .a-price .a-offscreen",
                "#corePrice_feature_div .a-price .a-offscreen",
                ".priceToPay .a-offscreen",
                ".apexPriceToPay .a-offscreen",
                ".a-price .a-offscreen",
                ".a-price-whole",
                "#priceblock_ourprice",
                "#priceblock_dealprice",
                "#price_inside_buybox",
                "#buyNewSection .a-color-price",
            ];

            let (has_title, has_price) = {
                let doc = Html::parse_document(&html_content);
                let (t, p) = parse_selectors(&doc, title_selectors, price_selectors);
                (t.is_some(), p.is_some())
            };

            // If reqwest got blocked (e.g., small payload or no title), fallback to curl.exe
            if !has_title || !has_price || html_content.len() < 10000 {
                if let Some(curl_html) = fetch_html_with_curl(&target_url, effective_user_agent).await {
                    if curl_html.len() > html_content.len() {
                        html_content = curl_html;
                    }
                }
            }

            let document = Html::parse_document(&html_content);
            let (title_opt, price_opt) = parse_selectors(&document, title_selectors, price_selectors);

            let title = title_opt
                .or_else(|| parse_meta_tag(&document, "meta[property='og:title']"))
                .or_else(|| parse_meta_tag(&document, "meta[name='title']"))
                .or_else(|| {
                    parse_selectors(&document, &["title"], &[]).0.and_then(|t| {
                        let candidate = t.split(':').next().unwrap_or(&t).split('|').next().unwrap_or(&t).trim().to_string();
                        if candidate != "Amazon.in" && candidate != "Amazon.com" && !candidate.contains("Robot Check") && !candidate.is_empty() {
                            Some(candidate)
                        } else {
                            None
                        }
                    })
                })
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
