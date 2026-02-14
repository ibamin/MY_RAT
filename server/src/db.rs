use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::env;

fn is_duplicate_column_error(e: &sqlx::Error) -> bool {
    match e {
        sqlx::Error::Database(db) => db.message().contains("duplicate column name"),
        _ => false,
    }
}

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
            scenario_id TEXT,
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

    // Best-effort migration for existing DBs.
    if let Err(e) = sqlx::query("ALTER TABLE runs ADD COLUMN scenario_id TEXT")
        .execute(&pool)
        .await
    {
        if !is_duplicate_column_error(&e) {
            return Err(e);
        }
    }

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

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS steps (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            idx INTEGER NOT NULL,
            name TEXT NOT NULL,
            status TEXT NOT NULL,
            started_at TEXT,
            ended_at TEXT
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS evidence (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            step_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            locator TEXT NOT NULL,
            sha256 TEXT NOT NULL,
            content_json TEXT,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS assertions (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            step_id TEXT NOT NULL,
            description TEXT NOT NULL,
            required INTEGER NOT NULL,
            rule_type TEXT,
            kind TEXT,
            contains TEXT,
            status TEXT NOT NULL,
            evidence_refs_json TEXT
        )",
    )
    .execute(&pool)
    .await?;

    for stmt in [
        "ALTER TABLE assertions ADD COLUMN rule_type TEXT",
        "ALTER TABLE assertions ADD COLUMN kind TEXT",
        "ALTER TABLE assertions ADD COLUMN contains TEXT",
    ] {
        if let Err(e) = sqlx::query(stmt).execute(&pool).await {
            if !is_duplicate_column_error(&e) {
                return Err(e);
            }
        }
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS verdicts (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            step_id TEXT NOT NULL,
            verdict TEXT NOT NULL,
            reason_code TEXT,
            summary TEXT,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS operator_actions (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            type TEXT NOT NULL,
            action_id TEXT,
            choice_id TEXT,
            note TEXT,
            ts TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
