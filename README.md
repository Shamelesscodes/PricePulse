# ⚡ PricePulse

> A High-Performance, Concurrent Product Price Tracker and Discovery System built in Rust.

PricePulse is a production-grade CLI application that monitors product prices across multiple e-commerce platforms (Amazon, Flipkart, Steam, Myntra, Nykaa, and Savana) and notifies you when they drop or reach your target threshold. It uses asynchronous Rust to perform concurrent checks, persists data using SQLite (via SQLx), and supports modular scrapers using Rust traits.

---

## 🏗️ Architecture

```
                    ┌────────────────────────┐
                    │          CLI           │
                    │   (Clap Command Line)  │
                    └───────────┬────────────┘
                                │
                                ▼
                    ┌────────────────────────┐
                    │     Command Parser     │
                    └─────┬────────────┬─────┘
                          │            │
             ┌────────────┘            └────────────┐
             ▼                                      ▼
   ┌───────────────────┐                  ┌───────────────────┐
   │  Product Manager  │                  │     Scheduler     │
   │ (Add/List/Remove) │                  │  (Interval Loop)  │
   └─────────┬─────────┘                  └─────────┬─────────┘
             │                                      │
             └──────────────────┬───────────────────┘
                                │
                                ▼
                    ┌────────────────────────┐
                    │   Async Task Manager   │
                    │   (Tokio Semaphore)    │
                    └───────────┬────────────┘
                                │
                                ▼
                    ┌────────────────────────┐
                    │      PriceScraper      │
                    │    (Scraper Trait)     │
                    └─┬──────┬──────┬──────┬─┘
                      │      │      │      │
         ┌────────────┘      │      │      └────────────┐
         ▼                   ▼      ▼                   ▼
   ┌───────────┐       ┌──────────┐ ┌──────────┐  ┌───────────┐
   │  Amazon   │       │ Flipkart │ │  Steam   │  │  Myntra/  │
   │  Scraper  │       │ Scraper  │ │ Scraper  │  │  Nykaa    │
   └───────────┘       └──────────┘ └──────────┘  └───────────┘
         │                   │      │                   │
         └───────────────────┼──────┴───────────────────┘
                             │
                             ▼
                    ┌────────────────────────┐
                    │     Product Parser     │
                    └───────────┬────────────┘
                                │
                                ▼
                    ┌────────────────────────┐
                    │    SQLite Database     │
                    │  (SQLx WAL Persistence)│
                    └───────────┬────────────┘
                                │
                                ▼
                    ┌────────────────────────┐
                    │   Notification Layer   │
                    │   (Terminal Alerts)    │
                    └────────────────────────┘
```

---

## ✨ Features

- **Concurrent Async Scraping**: Leverages Tokio and Reqwest to perform high-performance concurrent scraping across multiple websites with rate limiting (semaphore controlled).
- **SQLite Persistence**: Stores products, price histories, and alert records locally using SQLite. SQLx is used for compile-time verified queries and automatic database migrations.
- **Robust Extensible Scrapers**: Clean trait-based design makes it simple to add scrapers for new e-commerce sites.
- **Configurable Daemon Mode**: Runs persistently as a daemon, checking prices at customizable intervals.
- **SMTP Email Alerts**: Receive an email listing your active products upon starting the monitor command, plus instant email updates when price drops or target price thresholds are met.
- **Comprehensive Logging**: Detailed diagnostics and step-by-step tracing utilizing the `tracing` framework.
- **Aesthetic Terminal UX**: Rich CLI visuals, colors, and progress indicators powered by `colored` and `indicatif`.

---

## 🛠️ Tech Stack

- **Language**: Rust (Edition 2021)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **HTTP Client**: [Reqwest](https://github.com/seanmonstar/reqwest) (with Rustls TLS)
- **HTML Parsing**: [Scraper](https://github.com/causal-agent/scraper)
- **Database / ORM**: [SQLx](https://github.com/launchbadge/sqlx) (SQLite driver with WAL logging)
- **Command Line Parser**: [Clap](https://clap.rs/) (v4 with derive macros)
- **Logging & Diagnostics**: [Tracing](https://github.com/tokio-rs/tracing) & `tracing-subscriber`
- **Configuration**: [TOML](https://github.com/toml-rs/toml)
- **Date/Time**: [Chrono](https://github.com/chronotope/chrono)

---

## 🚀 Getting Started

### 1. Prerequisites

Ensure you have the Rust toolchain installed:

```bash
# Verify Rust installation
rustc --version
cargo --version
```

### 2. Configuration

Create or modify `config/config.toml` inside the `pricepulse` directory:

```toml
check_interval = 30           # Minutes between background checks
database = "pricepulse.db"    # SQLite database file path
max_concurrent_requests = 20  # Max concurrent HTTP requests
user_agent = "PricePulse/1.0" # Custom User-Agent header

[email]
enabled = false                              # Toggle email notifications
smtp_host = "smtp.gmail.com"                 # SMTP server address
smtp_port = 587                              # SMTP port (587 for STARTTLS, 465 for implicit TLS)
smtp_username = "your-email@gmail.com"       # Outgoing email username
smtp_password = "your-google-app-password"   # Outgoing email app password
from_email = "your-email@gmail.com"          # Sender address
to_email = "destination-email@gmail.com"     # Recipient address
```

### 3. Build & Setup

Run cargo to compile the codebase. On the first run, SQLite database will be automatically created and migrations executed programmatically.

```bash
cargo build --release
```

---

## 💻 Usage

PricePulse provides a rich set of subcommands. You can view them using `--help`:

```bash
cargo run -- --help
```

### Subcommands

#### Add a Product
Add a product URL to track. You can optionally supply a target price threshold (e.g., alert when price falls to or below this target).
```bash
cargo run -- add "https://www.amazon.in/dp/B0CXM7S4NZ" --target-price 59999.00
```

#### List Watched Products
View details of all monitored products, their status, initial price, current price, target threshold, and latest scraped status.
```bash
cargo run -- list
```

#### Pause Monitoring
Temporarily stop tracking a product by its ID:
```bash
cargo run -- pause 1
```

#### Resume Monitoring
Resume tracking a paused product:
```bash
cargo run -- resume 1
```

#### Set/Update Target Price
Set a new target price limit or clear it:
```bash
# Update target threshold
cargo run -- set-target 1 --price 58000.00

# Clear target threshold
cargo run -- set-target 1
```

#### Remove a Product
Delete a product and all of its associated price history from the database:
```bash
cargo run -- remove 1
```

#### Start Monitoring Loop
Start the persistent background scraping loop:
```bash
# Run background checking daemon
cargo run -- monitor

# Run all scrapers exactly once and exit immediately
cargo run -- monitor --once
```

---

## 📊 Database Schema

```sql
-- Products Table
CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    website TEXT NOT NULL,
    target_price REAL,
    active INTEGER NOT NULL DEFAULT 1, -- 0 = inactive, 1 = active
    created_at TEXT NOT NULL
);

-- Price History Table
CREATE TABLE price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER NOT NULL,
    price REAL NOT NULL,
    currency TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE
);

-- Alerts Table
CREATE TABLE alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER NOT NULL,
    trigger_price REAL NOT NULL,
    sent_at TEXT NOT NULL,
    FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE
);
```

---

## 🧩 Adding a New Scraper

Adding a new e-commerce scraper is clean and modular. Every scraper implements the `PriceScraper` trait:

```rust
#[async_trait]
pub trait PriceScraper: Send + Sync {
    async fn fetch(
        &self,
        client: &reqwest::Client,
        url: &str,
        user_agent: &str,
    ) -> Result<ScrapedProduct, ScraperError>;
}
```

### Steps to Add a New Site (e.g. `MyStore`):

1. **Create the Scraper Implementation**: Add `src/scraper/mystore.rs` and implement the `PriceScraper` trait, including HTML parsing logic for price and title.
2. **Register the Module**: Add the module declaration in `src/scraper/mod.rs`:
   ```rust
   pub mod mystore;
   ```
3. **Bind to URL Parser**: Map the new scraper in the URL dispatcher (`get_scraper_for_url` function in `src/scraper/mod.rs`):
   ```rust
   if url.contains("mystore.com") {
       return Ok((Arc::new(mystore::MyStoreScraper), "MyStore".to_string()));
   }
   ```

---

## 🪵 Logging & Diagnostics

You can configure diagnostic verbosity by setting the `RUST_LOG` environment variable:

```bash
# Enable trace-level logging
$env:RUST_LOG="trace" ; cargo run -- monitor
```