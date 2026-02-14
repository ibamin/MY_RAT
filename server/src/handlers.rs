use crate::fingerprint::FingerprintMatcher;
use crate::models::{
    Agent, CreateEventRequest, CreateRunRequest, Event, FingerprintMatchRequest,
    FingerprintMatchResponse, RegisterRequest, Run, UpdateRunResultRequest,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub matcher: FingerprintMatcher,
}

async fn agent_exists(state: &AppState, agent_id: &str) -> Result<bool, (StatusCode, String)> {
    let row = sqlx::query("SELECT 1 FROM agents WHERE id = ?")
        .bind(agent_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(row.is_some())
}

async fn run_exists(state: &AppState, run_id: &str) -> Result<bool, (StatusCode, String)> {
    let row = sqlx::query("SELECT 1 FROM runs WHERE id = ?")
        .bind(run_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(row.is_some())
}

pub async fn register_agent(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let now = Utc::now();

    let id = uuid::Uuid::new_v4().to_string();
    let last_seen = now.to_rfc3339();

    let agent = Agent {
        id,
        hostname: payload.hostname,
        ip: payload.ip,
        os: payload.os,
        arch: payload.arch,
        user: payload.user,
        last_seen,
        status: "online".to_string(),
    };

    sqlx::query(
        "INSERT INTO agents (id, hostname, ip, os, arch, user, last_seen, status) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&agent.id)
    .bind(&agent.hostname)
    .bind(&agent.ip)
    .bind(&agent.os)
    .bind(&agent.arch)
    .bind(&agent.user)
    .bind(&agent.last_seen)
    .bind(&agent.status)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agent))
}

pub async fn heartbeat(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let now = Utc::now().to_rfc3339();
    let res = sqlx::query("UPDATE agents SET last_seen = ?, status = 'online' WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }

    Ok(StatusCode::OK)
}

pub async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<Vec<Agent>>, (StatusCode, String)> {
    let agents = sqlx::query_as::<_, Agent>("SELECT * FROM agents")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agents))
}

pub async fn fingerprint_match(
    State(state): State<AppState>,
    Json(payload): Json<FingerprintMatchRequest>,
) -> Result<Json<FingerprintMatchResponse>, (StatusCode, String)> {
    let limit = payload.limit.unwrap_or(10);
    let candidates = state.matcher.match_banner(&payload.banner, limit);
    Ok(Json(FingerprintMatchResponse { candidates }))
}

pub async fn create_run(
    State(state): State<AppState>,
    Json(payload): Json<CreateRunRequest>,
) -> Result<Json<Run>, (StatusCode, String)> {
    if !agent_exists(&state, &payload.agent_id).await? {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }
    if payload.test_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "test_id is required".to_string()));
    }

    let now = Utc::now().to_rfc3339();
    let run = Run {
        id: uuid::Uuid::new_v4().to_string(),
        agent_id: payload.agent_id,
        test_id: payload.test_id,
        params_json: payload.params_json,
        status: "pending".to_string(),
        result_json: None,
        created_at: now.clone(),
        updated_at: now,
    };

    sqlx::query(
        "INSERT INTO runs (id, agent_id, test_id, params_json, status, result_json, created_at, updated_at)\n         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&run.id)
    .bind(&run.agent_id)
    .bind(&run.test_id)
    .bind(&run.params_json)
    .bind(&run.status)
    .bind(&run.result_json)
    .bind(&run.created_at)
    .bind(&run.updated_at)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(run))
}

pub async fn get_pending_runs(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<Vec<Run>>, (StatusCode, String)> {
    if !agent_exists(&state, &agent_id).await? {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut runs = sqlx::query_as::<_, Run>(
        "SELECT * FROM runs WHERE agent_id = ? AND status = 'pending' ORDER BY created_at ASC",
    )
    .bind(&agent_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !runs.is_empty() {
        let now = Utc::now().to_rfc3339();
        for run in &mut runs {
            let _ = sqlx::query(
                "UPDATE runs SET status = 'dispatched', updated_at = ? WHERE id = ? AND status = 'pending'",
            )
            .bind(&now)
            .bind(&run.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            run.status = "dispatched".to_string();
            run.updated_at = now.clone();
        }
    }

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(runs))
}

pub async fn update_run_result(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Json(payload): Json<UpdateRunResultRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let now = Utc::now().to_rfc3339();
    let res =
        sqlx::query("UPDATE runs SET status = ?, result_json = ?, updated_at = ? WHERE id = ?")
            .bind(&payload.status)
            .bind(&payload.result_json)
            .bind(&now)
            .bind(&run_id)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Run not found".to_string()));
    }

    Ok(StatusCode::OK)
}

pub async fn create_event(
    State(state): State<AppState>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<Json<Event>, (StatusCode, String)> {
    let level = payload.level.trim().to_lowercase();
    if !matches!(level.as_str(), "info" | "warn" | "error" | "debug") {
        return Err((
            StatusCode::BAD_REQUEST,
            "level must be one of: info|warn|error|debug".to_string(),
        ));
    }

    if let Some(ref run_id) = payload.run_id {
        if !run_exists(&state, run_id).await? {
            return Err((StatusCode::NOT_FOUND, "Run not found".to_string()));
        }
    }

    if let Some(ref agent_id) = payload.agent_id {
        if !agent_exists(&state, agent_id).await? {
            return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
        }
    }

    let ts = Utc::now().to_rfc3339();
    let event = Event {
        id: uuid::Uuid::new_v4().to_string(),
        run_id: payload.run_id,
        agent_id: payload.agent_id,
        level,
        message: payload.message,
        ts,
    };

    sqlx::query(
        "INSERT INTO events (id, run_id, agent_id, level, message, ts) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&event.id)
    .bind(&event.run_id)
    .bind(&event.agent_id)
    .bind(&event.level)
    .bind(&event.message)
    .bind(&event.ts)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(event))
}

pub async fn list_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, (StatusCode, String)> {
    let events = sqlx::query_as::<_, Event>("SELECT * FROM events ORDER BY ts DESC LIMIT 200")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(events))
}
