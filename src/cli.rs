use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pricepulse")]
#[command(about = "PricePulse: High-Performance Product Price Tracker in Rust", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a product URL to monitor
    Add {
        /// URL of the product page (Amazon, Flipkart, Steam, Myntra, Nykaa)
        url: String,

        /// Optional target price alert threshold
        #[arg(short, long)]
        target_price: Option<f64>,
    },

    /// List all watched products
    List,

    /// Remove a product from tracking by ID
    Remove {
        /// ID of the product to delete
        id: i64,
    },

    /// Pause monitoring for a product by ID
    Pause {
        /// ID of the product to pause
        id: i64,
    },

    /// Resume monitoring for a product by ID
    Resume {
        /// ID of the product to resume
        id: i64,
    },

    /// Set or update the target price alert for a product
    SetTarget {
        /// ID of the product to modify
        id: i64,

        /// Target price limit. If omitted, clears the target price alert.
        price: Option<f64>,
    },

    /// Start checking prices (loop runs persistently at check_interval)
    Monitor {
        /// Run all price scrapers once and exit immediately without looping
        #[arg(long)]
        once: bool,
    },

    /// Start the PricePulse REST API web server
    Serve {
        /// Port to bind the HTTP server to (overrides config)
        #[arg(short, long)]
        port: Option<u16>,

        /// Disable auto-starting background price monitoring scheduler
        #[arg(long)]
        no_scheduler: bool,
    },
}
