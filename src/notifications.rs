use crate::models::Product;
use crate::config::EmailConfig;
use colored::Colorize;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use tracing::{info, error};

pub async fn send_email_notification(
    email_config: &EmailConfig,
    subject: &str,
    body: &str,
) {
    let creds = Credentials::new(email_config.smtp_username.clone(), email_config.smtp_password.clone());

    let mailer_builder = if email_config.smtp_port == 587 {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&email_config.smtp_host)
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&email_config.smtp_host)
    };

    let mailer = match mailer_builder {
        Ok(builder) => builder
            .port(email_config.smtp_port)
            .credentials(creds)
            .build(),
        Err(e) => {
            error!("Failed to create SMTP mailer transport: {}", e);
            return;
        }
    };

    let email = Message::builder()
        .from(match email_config.from_email.parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("Invalid SMTP From address: {}", e);
                return;
            }
        })
        .to(match email_config.to_email.parse() {
            Ok(addr) => addr,
            Err(e) => {
                error!("Invalid SMTP To address: {}", e);
                return;
            }
        })
        .subject(subject)
        .body(body.to_string());

    let email = match email {
        Ok(msg) => msg,
        Err(e) => {
            error!("Failed to build email message: {}", e);
            return;
        }
    };

    match mailer.send(email).await {
        Ok(_) => info!("Email notification sent successfully to {}", email_config.to_email),
        Err(e) => error!("Failed to send email: {}", e),
    }
}

pub async fn notify_price_drop(
    product: &Product,
    old_price: f64,
    new_price: f64,
    currency: &str,
    email_config: Option<&EmailConfig>,
) {
    let diff = old_price - new_price;
    let percent = (diff / old_price) * 100.0;
    
    println!(
        "\n🔔 {} {}{} (↓ {:.2}%)",
        "PRICE DROP DETECTED:".bold().yellow(),
        product.title.bold().cyan(),
        format!(" on {}", product.website).dimmed(),
        percent
    );
    println!(
        "   {} {}{}  ➡️  {}{} ({:.2} saved!)",
        "Change:".dimmed(),
        currency,
        old_price,
        currency,
        new_price.to_string().bold().green(),
        diff
    );

    if let Some(cfg) = email_config {
        if cfg.enabled {
            let subject = format!("Price Drop Alert: {} on {}", product.title, product.website);
            let body = format!(
                "Hello,\n\nA price drop has been detected for a product you are tracking:\n\n\
                Product: {}\n\
                Website: {}\n\
                Original Price: {}{:.2}\n\
                New Price: {}{:.2} (Saved {}{:.2}, down {:.2}%)\n\
                URL: {}\n\n\
                Best regards,\nPricePulse Team",
                product.title, product.website, currency, old_price, currency, new_price, currency, diff, percent, product.url
            );
            send_email_notification(cfg, &subject, &body).await;
        }
    }
}

pub async fn notify_price_increase(product: &Product, old_price: f64, new_price: f64, currency: &str) {
    let diff = new_price - old_price;
    let percent = (diff / old_price) * 100.0;
    
    println!(
        "\n📈 {} {}{} (↑ {:.2}%)",
        "PRICE INCREASED:".bold().red(),
        product.title.bold().cyan(),
        format!(" on {}", product.website).dimmed(),
        percent
    );
    println!(
        "   {} {}{}  ➡️  {}{}",
        "Change:".dimmed(),
        currency,
        old_price,
        currency,
        new_price.to_string().bold().red(),
    );
}

pub async fn notify_target_reached(
    product: &Product,
    current_price: f64,
    target_price: f64,
    currency: &str,
    email_config: Option<&EmailConfig>,
) {
    println!(
        "\n🔥 {} Target price of {}{} was reached/beaten for {}!",
        "TARGET MET!".bold().green(),
        currency,
        target_price,
        product.title.bold().cyan()
    );
    println!(
        "   Current Price: {}{}",
        currency,
        current_price.to_string().bold().green()
    );
    println!("   Product URL:   {}", product.url.underline().blue());

    if let Some(cfg) = email_config {
        if cfg.enabled {
            let subject = format!("Price Alert (Target Met): {} on {}", product.title, product.website);
            let body = format!(
                "Hello,\n\nYour target price has been met/beaten for a product you are tracking:\n\n\
                Product: {}\n\
                Website: {}\n\
                Current Price: {}{:.2}\n\
                Target Price: {}{:.2}\n\
                URL: {}\n\n\
                Best regards,\nPricePulse Team",
                product.title, product.website, currency, current_price, currency, target_price, product.url
            );
            send_email_notification(cfg, &subject, &body).await;
        }
    }
}

pub fn print_product_info(product: &Product, last_price: Option<f64>, currency: Option<&str>) {
    let id_str = format!("[#{}]", product.id.unwrap_or(0)).bold().white();
    
    let price_str = match last_price {
        Some(p) => format!("{}{}", currency.unwrap_or("₹"), p).bold().green().to_string(),
        None => "No Price Scraped Yet".dimmed().to_string(),
    };
    
    let target_str = match product.target_price {
        Some(t) => format!("{}{}", currency.unwrap_or("₹"), t).yellow().to_string(),
        None => "None".dimmed().to_string(),
    };

    let status_str = if product.active {
        "MONITORING".green().bold()
    } else {
        "PAUSED".yellow().bold()
    };

    println!(
        "{} {} ({})\n   Price: {} | Target: {} | Status: {}\n   URL:   {}",
        id_str,
        product.title.bold(),
        product.website.cyan(),
        price_str,
        target_str,
        status_str,
        product.url.dimmed()
    );
    println!("{}", "-".repeat(60).dimmed());
}

