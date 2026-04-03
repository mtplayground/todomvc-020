use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Initialize the SQLite connection pool and run embedded migrations.
///
/// Reads the database URL from the `DATABASE_URL` environment variable.
/// Defaults to `sqlite://todos.db?mode=rwc` if not set.
pub async fn init_pool() -> Result<SqlitePool, sqlx::Error> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://todos.db?mode=rwc".to_string());

    let connect_options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::info!("database initialized at {}", database_url);
    Ok(pool)
}
