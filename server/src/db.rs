use chrono::Utc;
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
            status TEXT NOT NULL,
            approval_status TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    if let Err(e) = sqlx::query("ALTER TABLE agents ADD COLUMN approval_status TEXT")
        .execute(&pool)
        .await
    {
        if !is_duplicate_column_error(&e) {
            return Err(e);
        }
    }

    let _ = sqlx::query("UPDATE agents SET approval_status = 'approved' WHERE approval_status IS NULL")
        .execute(&pool)
        .await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS runs (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            scenario_id TEXT,
            test_id TEXT NOT NULL,
            params_json TEXT,
            replay_seed TEXT,
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

    if let Err(e) = sqlx::query("ALTER TABLE runs ADD COLUMN replay_seed TEXT")
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
            scenario_step_id TEXT,
            idx INTEGER NOT NULL,
            name TEXT NOT NULL,
            status TEXT NOT NULL,
            executor_info TEXT,
            started_at TEXT,
            ended_at TEXT
        )",
    )
    .execute(&pool)
    .await?;

    if let Err(e) = sqlx::query("ALTER TABLE steps ADD COLUMN scenario_step_id TEXT")
        .execute(&pool)
        .await
    {
        if !is_duplicate_column_error(&e) {
            return Err(e);
        }
    }

    if let Err(e) = sqlx::query("ALTER TABLE steps ADD COLUMN executor_info TEXT")
        .execute(&pool)
        .await
    {
        if !is_duplicate_column_error(&e) {
            return Err(e);
        }
    }

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

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agent_groups (
            agent_id TEXT NOT NULL,
            group_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            PRIMARY KEY (agent_id, group_id)
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agent_tags (
            agent_id TEXT NOT NULL,
            tag TEXT NOT NULL,
            created_at TEXT NOT NULL,
            PRIMARY KEY (agent_id, tag)
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agent_builds (
            id TEXT PRIMARY KEY,
            guid TEXT NOT NULL UNIQUE,
            target_platform TEXT NOT NULL,
            server_url TEXT NOT NULL,
            sleep_sec INTEGER NOT NULL,
            build_status TEXT NOT NULL,
            binary_path TEXT,
            error_message TEXT,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS achievements (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            category TEXT NOT NULL,
            icon TEXT NOT NULL,
            requirement_type TEXT NOT NULL,
            requirement_value TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_achievements (
            id TEXT PRIMARY KEY,
            achievement_id TEXT NOT NULL UNIQUE,
            unlocked_at TEXT,
            progress INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await?;

    let seeded_at = Utc::now().to_rfc3339();
    let seed_rows = [
        (
            "first_mission",
            "First Mission",
            "Complete your first successful operation.",
            "combat",
            "🎯",
            "scenario_count",
            r#"{"count":1,"depends_on":[]}"#,
        ),
        (
            "field_operator",
            "Field Operator",
            "Secure 5 PASS verdict operations.",
            "combat",
            "🛡️",
            "scenario_count",
            r#"{"count":5,"depends_on":["first_mission"]}"#,
        ),
        (
            "perfect_score",
            "Perfect Score",
            "Execute 10 flawless PASS operations.",
            "mastery",
            "💎",
            "scenario_count",
            r#"{"count":10,"depends_on":["field_operator"]}"#,
        ),
        (
            "combat_initiate",
            "Combat Initiate",
            "Maintain a 2-run PASS streak.",
            "combat",
            "⚔️",
            "verdict_streak",
            r#"{"streak":2,"depends_on":["first_mission"]}"#,
        ),
        (
            "combat_veteran",
            "Combat Veteran",
            "Maintain a 5-run PASS streak.",
            "combat",
            "🔥",
            "verdict_streak",
            r#"{"streak":5,"depends_on":["combat_initiate"]}"#,
        ),
        (
            "stealth_master",
            "Stealth Master",
            "Complete STEALTH-LAB-001 successfully.",
            "stealth",
            "🕶️",
            "specific_scenario",
            r#"{"test_id":"STEALTH-LAB-001","count":1,"depends_on":["first_mission"]}"#,
        ),
        (
            "ghost_protocol",
            "Ghost Protocol",
            "Complete STEALTH-LAB-001 three times.",
            "stealth",
            "👻",
            "specific_scenario",
            r#"{"test_id":"STEALTH-LAB-001","count":3,"depends_on":["stealth_master"]}"#,
        ),
        (
            "recon_expert",
            "Recon Expert",
            "Complete RECON-LAB-001 successfully.",
            "recon",
            "🛰️",
            "specific_scenario",
            r#"{"test_id":"RECON-LAB-001","count":1,"depends_on":["first_mission"]}"#,
        ),
        (
            "signal_hunter",
            "Signal Hunter",
            "Complete RECON-LAB-001 three times.",
            "recon",
            "📡",
            "specific_scenario",
            r#"{"test_id":"RECON-LAB-001","count":3,"depends_on":["recon_expert"]}"#,
        ),
        (
            "mastery_path",
            "Mastery Path",
            "Hold a 3-run PASS streak and 6 total wins.",
            "mastery",
            "🧠",
            "verdict_streak",
            r#"{"streak":3,"depends_on":["field_operator","combat_initiate"]}"#,
        ),
        (
            "scenario_specialist",
            "Scenario Specialist",
            "Clear BAS-DEMO-001 two times.",
            "recon",
            "🧭",
            "specific_scenario",
            r#"{"test_id":"BAS-DEMO-001","count":2,"depends_on":["first_mission"]}"#,
        ),
        (
            "tactical_legend",
            "Tactical Legend",
            "Reach 15 successful operations.",
            "mastery",
            "🏆",
            "scenario_count",
            r#"{"count":15,"depends_on":["perfect_score","mastery_path"]}"#,
        ),
    ];

    for (id, name, description, category, icon, requirement_type, requirement_value) in seed_rows {
        sqlx::query(
            "INSERT OR IGNORE INTO achievements (id, name, description, category, icon, requirement_type, requirement_value, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(category)
        .bind(icon)
        .bind(requirement_type)
        .bind(requirement_value)
        .bind(&seeded_at)
        .execute(&pool)
        .await?;
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_accounts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider TEXT NOT NULL,
            auth_type TEXT NOT NULL,
            api_key TEXT NOT NULL,
            model TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_conversations (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            title TEXT NOT NULL,
            messages_json TEXT NOT NULL DEFAULT '[]',
            scenario_draft TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
