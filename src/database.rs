use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub async fn establish_connection(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = Path::new(db_path).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }

    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    let pool = SqlitePool::connect_with(opts).await?;

    // Programmatically run migrations located in /migrations
    sqlx::migrate!()
        .run(&pool)
        .await?;

    Ok(pool)
}
