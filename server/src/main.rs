mod db;
mod fingerprint;
mod handlers;
mod models;
mod scenarios;

use axum::{
    Router,
    routing::{get, post},
};
use handlers::AppState;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method};
use std::net::SocketAddr;
use std::process::Command;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use scenarios::ScenarioCatalog;
use sqlx::SqlitePool;

fn env_true(key: &str) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"),
        Err(_) => false,
    }
}

fn launch_ui(server_url: &str) {
    let ui_dir = std::env::var("UI_DIR").unwrap_or_else(|_| "../ui".to_string());
    let shell = if cfg!(windows) { "cmd" } else { "sh" };
    let shell_flag = if cfg!(windows) { "/C" } else { "-lc" };

    let mut cmd = Command::new(shell);
    cmd.arg(shell_flag)
        .arg("npm run dev")
        .current_dir(ui_dir)
        .env("VITE_SERVER_URL", server_url)
        .env("ELECTRON_RENDERER_URL", "http://127.0.0.1:5173");

    let _ = cmd.spawn();
}

async fn spawn_status_ticker(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;

            let agents_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
                .fetch_one(&pool)
                .await
                .unwrap_or(0);
            let agents_online: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents WHERE status = 'online'")
                .fetch_one(&pool)
                .await
                .unwrap_or(0);

            let runs_pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM runs WHERE status = 'pending'")
                .fetch_one(&pool)
                .await
                .unwrap_or(0);
            let runs_dispatched: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM runs WHERE status = 'dispatched'")
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);
            let runs_completed: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM runs WHERE status = 'completed'")
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);

            println!(
                "[console] agents {}/{} online | runs pending={} dispatched={} completed={}",
                agents_online, agents_total, runs_pending, runs_dispatched, runs_completed
            );
        }
    });
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = db::init_db().await.expect("Failed to initialize database");

    let matcher = std::env::var("FINGERPRINT_RULES_PATH")
        .ok()
        .and_then(|p| fingerprint::FingerprintMatcher::load_from_json_path(&p).ok())
        .unwrap_or_else(fingerprint::FingerprintMatcher::empty);

    let scenarios_dir = std::env::var("SCENARIOS_PATH").unwrap_or_else(|_| "data/scenarios".to_string());
    let scenarios = ScenarioCatalog::load_from_dir(&scenarios_dir)
        .map_err(|e| {
            eprintln!("Failed to load scenarios from {}: {}", scenarios_dir, e);
            e
        })
        .ok()
        .unwrap_or_else(ScenarioCatalog::empty);

    let state = AppState {
        pool,
        matcher,
        scenarios,
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server_url = format!("http://{}", addr);
    if env_true("LAUNCH_UI") {
        launch_ui(&server_url);
    }
    spawn_status_ticker(state.pool.clone()).await;

    let cors = if env_true("CORS_PERMISSIVE") {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin([
                HeaderValue::from_static("http://127.0.0.1:5173"),
                HeaderValue::from_static("http://localhost:5173"),
                HeaderValue::from_static("http://127.0.0.1:4173"),
                HeaderValue::from_static("http://localhost:4173"),
                HeaderValue::from_static("null"),
            ])
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([CONTENT_TYPE])
    };

    let app = Router::new()
        .route("/api/scenarios", get(handlers::list_scenarios))
        .route(
            "/api/scenarios/:scenario_id",
            get(handlers::get_scenario),
        )
        .route("/api/agents/register", post(handlers::register_agent))
        .route("/api/agents/list", get(handlers::list_agents))
        .route("/api/agents/:id/heartbeat", post(handlers::heartbeat))
        .route("/api/fingerprint/match", post(handlers::fingerprint_match))
        .route("/api/runs", post(handlers::create_run).get(handlers::list_runs))
        .route("/api/runs/:run_id", get(handlers::get_run))
        .route(
            "/api/runs/:run_id/operator-actions",
            post(handlers::create_operator_action).get(handlers::list_operator_actions),
        )
        .route("/api/runs/:run_id/events", get(handlers::list_run_events))
        .route("/api/runs/:run_id/steps", get(handlers::list_run_steps))
        .route(
            "/api/runs/:run_id/evidence",
            get(handlers::list_run_evidence),
        )
        .route("/api/runs/:run_id/verdict", get(handlers::get_run_verdict))
        .route("/api/evidence", post(handlers::create_evidence))
        .route(
            "/api/runs/pending/:agent_id",
            get(handlers::get_pending_runs),
        )
        .route(
            "/api/runs/:run_id/result",
            post(handlers::update_run_result),
        )
        .route("/api/events", post(handlers::create_event))
        .route("/api/events", get(handlers::list_events))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    println!("C2 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
