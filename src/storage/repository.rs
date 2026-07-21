use crate::models::{Alert, PriceHistory, Product, User};
use chrono::Utc;
use sqlx::SqlitePool;

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // --- USER METHODS ---

    pub async fn create_user(
        &self,
        email: &str,
        password_hash: Option<&str>,
        google_id: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<User, sqlx::Error> {
        let now = Utc::now();
        let res = sqlx::query(
            "INSERT INTO users (email, password_hash, google_id, avatar_url, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(email)
        .bind(password_hash)
        .bind(google_id)
        .bind(avatar_url)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row_id = res.last_insert_rowid();

        Ok(User {
            id: Some(row_id),
            email: email.to_string(),
            password_hash: password_hash.map(|s| s.to_string()),
            google_id: google_id.map(|s| s.to_string()),
            avatar_url: avatar_url.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, google_id, avatar_url, created_at FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, google_id, avatar_url, created_at FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_user_by_google_id(&self, google_id: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, google_id, avatar_url, created_at FROM users WHERE google_id = ?",
        )
        .bind(google_id)
        .fetch_optional(&self.pool)
        .await
    }

    // --- PRODUCT METHODS ---

    pub async fn add_product(
        &self,
        user_id: Option<i64>,
        title: &str,
        url: &str,
        website: &str,
        target_price: Option<f64>,
    ) -> Result<Product, sqlx::Error> {
        let now = Utc::now();
        let res = sqlx::query(
            "INSERT INTO products (user_id, title, url, website, target_price, active, created_at) VALUES (?, ?, ?, ?, ?, 1, ?)",
        )
        .bind(user_id)
        .bind(title)
        .bind(url)
        .bind(website)
        .bind(target_price)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row_id = res.last_insert_rowid();

        Ok(Product {
            id: Some(row_id),
            user_id,
            title: title.to_string(),
            url: url.to_string(),
            website: website.to_string(),
            target_price,
            active: true,
            created_at: now,
        })
    }

    pub async fn list_products(&self) -> Result<Vec<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "SELECT id, user_id, title, url, website, target_price, active, created_at FROM products",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_products_by_user(&self, user_id: i64) -> Result<Vec<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "SELECT id, user_id, title, url, website, target_price, active, created_at FROM products WHERE user_id = ? OR user_id IS NULL",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn remove_product(&self, id: i64) -> Result<u64, sqlx::Error> {
        let res = sqlx::query("DELETE FROM products WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn remove_user_product(&self, user_id: i64, id: i64) -> Result<u64, sqlx::Error> {
        let res = sqlx::query("DELETE FROM products WHERE id = ? AND (user_id = ? OR user_id IS NULL)")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn set_active_status(&self, id: i64, active: bool) -> Result<u64, sqlx::Error> {
        let active_val = if active { 1 } else { 0 };
        let res = sqlx::query("UPDATE products SET active = ? WHERE id = ?")
            .bind(active_val)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn set_user_product_active_status(&self, user_id: i64, id: i64, active: bool) -> Result<u64, sqlx::Error> {
        let active_val = if active { 1 } else { 0 };
        let res = sqlx::query("UPDATE products SET active = ? WHERE id = ? AND (user_id = ? OR user_id IS NULL)")
            .bind(active_val)
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn set_target_price(&self, id: i64, target_price: Option<f64>) -> Result<u64, sqlx::Error> {
        let res = sqlx::query("UPDATE products SET target_price = ? WHERE id = ?")
            .bind(target_price)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn set_user_product_target_price(&self, user_id: i64, id: i64, target_price: Option<f64>) -> Result<u64, sqlx::Error> {
        let res = sqlx::query("UPDATE products SET target_price = ? WHERE id = ? AND (user_id = ? OR user_id IS NULL)")
            .bind(target_price)
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    #[allow(dead_code)]
    pub async fn get_product_by_id(&self, id: i64) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "SELECT id, user_id, title, url, website, target_price, active, created_at FROM products WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_user_product_by_id(&self, user_id: i64, id: i64) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "SELECT id, user_id, title, url, website, target_price, active, created_at FROM products WHERE id = ? AND (user_id = ? OR user_id IS NULL)",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_product_by_url(&self, url: &str) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            "SELECT id, user_id, title, url, website, target_price, active, created_at FROM products WHERE url = ?",
        )
        .bind(url)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn add_price_history(
        &self,
        product_id: i64,
        price: f64,
        currency: &str,
    ) -> Result<PriceHistory, sqlx::Error> {
        let now = Utc::now();
        let res = sqlx::query(
            "INSERT INTO price_history (product_id, price, currency, timestamp) VALUES (?, ?, ?, ?)",
        )
        .bind(product_id)
        .bind(price)
        .bind(currency)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row_id = res.last_insert_rowid();

        Ok(PriceHistory {
            id: Some(row_id),
            product_id,
            price,
            currency: currency.to_string(),
            timestamp: now,
        })
    }

    pub async fn get_price_history(&self, product_id: i64) -> Result<Vec<PriceHistory>, sqlx::Error> {
        sqlx::query_as::<_, PriceHistory>(
            "SELECT id, product_id, price, currency, timestamp FROM price_history WHERE product_id = ? ORDER BY timestamp DESC",
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn add_alert(&self, product_id: i64, trigger_price: f64) -> Result<Alert, sqlx::Error> {
        let now = Utc::now();
        let res = sqlx::query("INSERT INTO alerts (product_id, trigger_price, sent_at) VALUES (?, ?, ?)")
            .bind(product_id)
            .bind(trigger_price)
            .bind(now)
            .execute(&self.pool)
            .await?;

        let row_id = res.last_insert_rowid();

        Ok(Alert {
            id: Some(row_id),
            product_id,
            trigger_price,
            sent_at: now,
        })
    }

    pub async fn get_latest_alert(&self, product_id: i64) -> Result<Option<Alert>, sqlx::Error> {
        sqlx::query_as::<_, Alert>(
            "SELECT id, product_id, trigger_price, sent_at FROM alerts WHERE product_id = ? ORDER BY sent_at DESC LIMIT 1",
        )
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await
    }
}
