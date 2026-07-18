use crate::errors::PricePulseError;
use regex::Regex;
use scraper::{Html, Selector};

/// Extracts a price float and a currency string from a text snippet.
/// Supports formats like "₹61,999.00", "$59.99", "Rs. 450", "99,99 €"
pub fn extract_price_and_currency(text: &str) -> Result<(f64, String), PricePulseError> {
    let cleaned = text.trim()
        .replace('\u{a0}', " ")
        .replace('\u{200b}', "")
        .replace('\u{20b9}', "₹"); // normalize Rupee symbol

    // Pattern to look for currency indicator and a number block (ends with a digit to avoid trailing punctuation)
    let re = Regex::new(r"([$₹€£¥]|Rs\.?|USD|INR|EUR)?\s*([\d,.]*\d)")
        .map_err(|e| PricePulseError::Parser(e.to_string()))?;

    if let Some(caps) = re.captures(&cleaned) {
        let mut currency = caps.get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| {
                if cleaned.contains('€') {
                    "€".to_string()
                } else if cleaned.contains('$') {
                    "$".to_string()
                } else if cleaned.contains('£') {
                    "£".to_string()
                } else if cleaned.contains('¥') {
                    "¥".to_string()
                } else {
                    "₹".to_string()
                }
            });

        if currency == "Rs." || currency == "INR" {
            currency = "₹".to_string();
        } else if currency == "USD" {
            currency = "$".to_string();
        } else if currency == "EUR" {
            currency = "€".to_string();
        }

        let price_str = caps.get(2)
            .map(|m| m.as_str())
            .ok_or_else(|| PricePulseError::Parser("No numeric price found".to_string()))?;

        // Normalize decimal/thousands separators
        // Cases:
        // - "1,250.50" (comma thousands, dot decimal) -> strip comma, parse
        // - "1.250,50" (dot thousands, comma decimal) -> strip dot, change comma to dot, parse
        // - "99,99"    (comma decimal) -> change comma to dot
        // - "1,250"    (comma thousands) -> strip comma
        // - "1.250"    (dot thousands) -> strip dot
        let mut normalized_price = price_str.to_string();

        if normalized_price.contains(',') && normalized_price.contains('.') {
            let comma_pos = normalized_price.find(',').unwrap();
            let dot_pos = normalized_price.find('.').unwrap();
            if comma_pos < dot_pos {
                // E.g. 1,250.50
                normalized_price = normalized_price.replace(',', "");
            } else {
                // E.g. 1.250,50
                normalized_price = normalized_price.replace('.', "").replace(',', ".");
            }
        } else if normalized_price.contains(',') {
            let parts: Vec<&str> = normalized_price.split(',').collect();
            if parts.len() == 2 && parts[1].len() == 2 {
                // E.g., 99,99 -> 99.99
                normalized_price = normalized_price.replace(',', ".");
            } else {
                // E.g., 1,250 -> 1250
                normalized_price = normalized_price.replace(',', "");
            }
        } else if normalized_price.contains('.') {
            let parts: Vec<&str> = normalized_price.split('.').collect();
            if parts.len() == 2 && parts[1].len() == 3 {
                // E.g., 1.250 -> 1250 (German/European thousands)
                normalized_price = normalized_price.replace('.', "");
            }
        }

        let price_num: f64 = normalized_price
            .parse()
            .map_err(|_| PricePulseError::Parser(format!("Failed to parse numeric string: {}", price_str)))?;

        Ok((price_num, currency))
    } else {
        Err(PricePulseError::Parser(format!("Could not match price regex in: '{}'", text)))
    }
}

/// Normalizes HTML whitespace and collapses spaces
pub fn clean_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Helper to extract content attribute of a meta tag
pub fn parse_meta_tag(document: &Html, selector_str: &str) -> Option<String> {
    if let Ok(selector) = Selector::parse(selector_str) {
        if let Some(el) = document.select(&selector).next() {
            if let Some(content) = el.value().attr("content") {
                let cleaned = clean_text(content);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
    }
    None
}

/// Parses HTML using list of fallback selectors for title and price
pub fn parse_selectors(
    document: &Html,
    title_selectors: &[&str],
    price_selectors: &[&str],
) -> (Option<String>, Option<String>) {
    let mut title = None;
    for sel in title_selectors {
        if let Ok(selector) = Selector::parse(sel) {
            if let Some(el) = document.select(&selector).next() {
                let t = el.text().collect::<Vec<_>>().join(" ");
                let t_clean = clean_text(&t);
                if !t_clean.is_empty() {
                    title = Some(t_clean);
                    break;
                }
            }
        }
    }

    let mut price = None;
    for sel in price_selectors {
        if let Ok(selector) = Selector::parse(sel) {
            if let Some(el) = document.select(&selector).next() {
                let p = el.text().collect::<Vec<_>>().join(" ");
                let p_clean = clean_text(&p);
                if !p_clean.is_empty() {
                    price = Some(p_clean);
                    break;
                }
            }
        }
    }

    (title, price)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_price_and_currency() {
        let (price, currency) = extract_price_and_currency("₹61,999.00").unwrap();
        assert_eq!(price, 61999.0);
        assert_eq!(currency, "₹");

        let (price, currency) = extract_price_and_currency("  $ 59.99  ").unwrap();
        assert_eq!(price, 59.99);
        assert_eq!(currency, "$");

        let (price, currency) = extract_price_and_currency("Rs. 450").unwrap();
        assert_eq!(price, 450.0);
        assert_eq!(currency, "₹");

        let (price, currency) = extract_price_and_currency("99,99 €").unwrap();
        assert_eq!(price, 99.99);
        assert_eq!(currency, "€");
    }

    #[test]
    fn test_clean_text() {
        assert_eq!(clean_text("  hello   world  "), "hello world");
    }
}
