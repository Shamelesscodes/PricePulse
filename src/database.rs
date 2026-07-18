use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

pub async fn establish_connection(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
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
