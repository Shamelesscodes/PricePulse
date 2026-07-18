mod cli;
mod config;
mod database;
mod errors;
mod models;
mod notifications;
mod parser;
mod scheduler;
mod scraper;
mod storage;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use scheduler::Scheduler;
use scraper::get_scraper_for_url;
use storage::Repository;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // 2. Load Configuration
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Configuration Error: {}", e);
            std::process::exit(1);
        }
    };

    // 3. Connect to Database & Run Migrations
    let pool = match database::establish_connection(&config.database).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Database connection/migration failed: {}", e);
            std::process::exit(1);
        }
    };

    let repo = Repository::new(pool);

    // 4. Parse CLI Arguments
    let args = Cli::parse();

    // 5. Dispatch commands
    match args.command {
        Commands::Add { url, target_price } => {
            // Check if product is already monitored
            match repo.get_product_by_url(&url).await {
                Ok(Some(existing)) => {
                    println!("⚠️  Product is already monitored! ID is: {}", existing.id.unwrap_or(0));
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Database Error: {}", e);
                    std::process::exit(1);
                }
                _ => {}
            }

            // Check URL and resolve scraper
            let (scraper, website) = match get_scraper_for_url(&url) {
                Ok(pair) => pair,
                Err(e) => {
                    eprintln!("❌ URL Error: {}", e);
                    std::process::exit(1);
                }
            };

            println!("🔍 Fetching details from {} to verify URL and get title...", website);
            let client = reqwest::Client::new();
            
            let scraped = match scraper.fetch(&client, &url, &config.user_agent).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("❌ Scraping failed: {}", e);
                    std::process::exit(1);
                }
            };

            println!("✅ Scraped Title: \"{}\"", scraped.title);
            println!("✅ Scraped Price: {}{}", scraped.currency, scraped.price);

            // Save product to database
            let product = match repo.add_product(&scraped.title, &url, &website, target_price).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("❌ Database Error saving product: {}", e);
                    std::process::exit(1);
                }
            };

            // Save initial price history
            let product_id = product.id.unwrap();
            if let Err(e) = repo.add_price_history(product_id, scraped.price, &scraped.currency).await {
                eprintln!("⚠️  Could not save initial price history: {}", e);
            }

            println!(
                "🎉 Product added successfully! [ID: {}]",
                product_id
            );
        }

        Commands::List => {
            let products = match repo.list_products().await {
                Ok(list) => list,
                Err(e) => {
                    eprintln!("❌ Database Error: {}", e);
                    std::process::exit(1);
                }
            };

            if products.is_empty() {
                println!("No products are currently watched.");
                return Ok(());
            }

            println!("{}\nWatched Products:\n{}", "=".repeat(60), "=".repeat(60));
            for product in products {
                let history = repo.get_price_history(product.id.unwrap()).await.unwrap_or_default();
                let last_record = history.first();
                let last_price = last_record.map(|r| r.price);
                let currency = last_record.map(|r| r.currency.as_str());

                notifications::print_product_info(&product, last_price, currency);
            }
        }

        Commands::Remove { id } => {
            match repo.remove_product(id).await {
                Ok(0) => {
                    println!("❌ Product with ID {} not found.", id);
                }
                Ok(_) => {
                    println!("🗑️  Product ID {} removed from watch list.", id);
                }
                Err(e) => {
                    eprintln!("❌ Database Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Pause { id } => {
            match repo.set_active_status(id, false).await {
                Ok(0) => {
                    println!("❌ Product with ID {} not found.", id);
                }
                Ok(_) => {
                    println!("⏸️  Paused price monitoring for product ID {}.", id);
                }
                Err(e) => {
                    eprintln!("❌ Database Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Resume { id } => {
            match repo.set_active_status(id, true).await {
                Ok(0) => {
                    println!("❌ Product with ID {} not found.", id);
                }
                Ok(_) => {
                    println!("▶️  Resumed price monitoring for product ID {}.", id);
                }
                Err(e) => {
                    eprintln!("❌ Database Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::SetTarget { id, price } => {
            match repo.set_target_price(id, price).await {
                Ok(0) => {
                    println!("❌ Product with ID {} not found.", id);
                }
                Ok(_) => {
                    match price {
                        Some(p) => println!("🎯 Set target price alert threshold to {:.2} for product ID {}.", p, id),
                        None => println!("🎯 Cleared target price alert threshold for product ID {}.", id),
                    }
                }
                Err(e) => {
                    eprintln!("❌ Database Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Monitor { once } => {
            let scheduler = Scheduler::new(config, repo);
            if once {
                scheduler.run_once().await;
            } else {
                scheduler.start_loop().await;
            }
        }
    }

    Ok(())
}
