use crate::config::Config;
use crate::storage::Repository;
use crate::scraper::get_scraper_for_url;
use crate::notifications::{notify_price_drop, notify_price_increase, notify_target_reached};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};
use tracing::{info, error};

pub struct Scheduler {
    config: Config,
    repository: Arc<Repository>,
    client: reqwest::Client,
}

impl Scheduler {
    pub fn new(config: Config, repository: Repository) -> Self {
        Self {
            config,
            repository: Arc::new(repository),
            client: reqwest::Client::new(),
        }
    }

    pub async fn run_once(&self) {
        info!("Starting scheduled price checks...");

        let products = match self.repository.list_products().await {
            Ok(list) => list,
            Err(e) => {
                error!("Failed to fetch products from database: {}", e);
                return;
            }
        };

        // Filter active products
        let active_products: Vec<_> = products.into_iter().filter(|p| p.active).collect();
        if active_products.is_empty() {
            info!("No active products to monitor.");
            return;
        }

        info!("Found {} active products to check.", active_products.len());

        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_requests));
        let mut tasks = vec![];

        for product in active_products {
            let repo = Arc::clone(&self.repository);
            let sem = Arc::clone(&semaphore);
            let client = self.client.clone();
            let user_agent = self.config.user_agent.clone();

            let task = tokio::spawn(async move {
                // Acquire concurrency permit
                let _permit = sem.acquire().await.unwrap();
                info!("Checking price for product: {} ({})", product.title, product.website);

                match get_scraper_for_url(&product.url) {
                    Ok((scraper, _)) => {
                        match scraper.fetch(&client, &product.url, &user_agent).await {
                            Ok(scraped) => {
                                info!("Scraped price for '{}': {} {}", product.title, scraped.currency, scraped.price);

                                // Fetch latest recorded price from history
                                let history = repo.get_price_history(product.id.unwrap()).await.unwrap_or_default();
                                let last_record = history.first();

                                // Save price history
                                if let Err(e) = repo.add_price_history(product.id.unwrap(), scraped.price, &scraped.currency).await {
                                    error!("Failed to save price history for '{}': {}", product.title, e);
                                }

                                if let Some(last) = last_record {
                                    if scraped.price < last.price {
                                        notify_price_drop(&product, last.price, scraped.price, &scraped.currency);
                                    } else if scraped.price > last.price {
                                        notify_price_increase(&product, last.price, scraped.price, &scraped.currency);
                                    }
                                } else {
                                    info!("Initial price recorded for '{}': {}{}", product.title, scraped.currency, scraped.price);
                                }

                                // Check target price alerts
                                if let Some(target) = product.target_price {
                                    if scraped.price <= target {
                                        let latest_alert = repo.get_latest_alert(product.id.unwrap()).await.unwrap_or_default();
                                        let should_alert = match latest_alert {
                                            Some(alert) => scraped.price < alert.trigger_price, // alert again if price falls further
                                            None => true,
                                        };

                                        if should_alert {
                                            notify_target_reached(&product, scraped.price, target, &scraped.currency);
                                            if let Err(e) = repo.add_alert(product.id.unwrap(), scraped.price).await {
                                                error!("Failed to save sent alert history: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Scraper failed for '{}' (URL: {}): {}", product.title, product.url, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Could not dispatch scraper for '{}': {}", product.title, e);
                    }
                }
            });

            tasks.push(task);
        }

        // Await all spawned checks
        for t in tasks {
            let _ = t.await;
        }

        info!("All price checks completed.");
    }

    pub async fn start_loop(&self) {
        info!("Starting background monitoring loop (interval: {}m).", self.config.check_interval);

        loop {
            self.run_once().await;

            info!("Sleeping for {} minutes before next check...", self.config.check_interval);
            sleep(Duration::from_secs(self.config.check_interval * 60)).await;
        }
    }
}
