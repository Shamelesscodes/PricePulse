use crate::models::Product;
use colored::Colorize;

pub fn notify_price_drop(product: &Product, old_price: f64, new_price: f64, currency: &str) {
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
}

pub fn notify_price_increase(product: &Product, old_price: f64, new_price: f64, currency: &str) {
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

pub fn notify_target_reached(product: &Product, current_price: f64, target_price: f64, currency: &str) {
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
