use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::env;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:red-sim.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            hostname TEXT NOT NULL,
            ip TEXT NOT NULL,
            os TEXT NOT NULL,
            arch TEXT NOT NULL,
            user TEXT NOT NULL,
            last_seen TEXT NOT NULL,
            status TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS runs (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            test_id TEXT NOT NULL,
            params_json TEXT,
            status TEXT NOT NULL,
            result_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY,
            run_id TEXT,
            agent_id TEXT,
            level TEXT NOT NULL,
            message TEXT NOT NULL,
            ts TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
