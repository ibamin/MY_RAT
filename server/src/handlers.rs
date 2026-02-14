use crate::fingerprint::FingerprintMatcher;
use crate::scenarios::{ScenarioCatalog, ScenarioDef, ScenarioMeta};
use crate::models::{
    Agent, Assertion, CreateEvidenceRequest, CreateEventRequest, CreateRunRequest, Evidence, Event,
    FingerprintMatchRequest, FingerprintMatchResponse, OperatorAction, OperatorActionRequest, Run,
    Step, VerdictRow, RegisterRequest, UpdateRunResultRequest,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use sha2::Digest;
use sqlx::SqlitePool;
use serde::Serialize;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub matcher: FingerprintMatcher,
    pub scenarios: ScenarioCatalog,
}

pub async fn list_scenarios(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScenarioMeta>>, (StatusCode, String)> {
    Ok(Json(state.scenarios.metas()))
}

pub async fn get_scenario(
    State(state): State<AppState>,
    Path(scenario_id): Path<String>,
) -> Result<Json<ScenarioDef>, (StatusCode, String)> {
    let scenario = state
        .scenarios
        .get_by_id(&scenario_id)
        .ok_or((StatusCode::NOT_FOUND, "Scenario not found".to_string()))?;
    Ok(Json(scenario))
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

async fn step_exists(state: &AppState, step_id: &str) -> Result<bool, (StatusCode, String)> {
    let row = sqlx::query("SELECT 1 FROM steps WHERE id = ?")
        .bind(step_id)
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

pub async fn list_runs(
    State(state): State<AppState>,
) -> Result<Json<Vec<Run>>, (StatusCode, String)> {
    let runs = sqlx::query_as::<_, Run>("SELECT * FROM runs ORDER BY created_at DESC LIMIT 100")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(runs))
}

pub async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<Run>, (StatusCode, String)> {
    let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = ?")
        .bind(&run_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Run not found".to_string()))?;
    Ok(Json(run))
}

pub async fn list_run_steps(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<Step>>, (StatusCode, String)> {
    if !run_exists(&state, &run_id).await? {
        return Err((StatusCode::NOT_FOUND, "Run not found".to_string()));
    }
    let steps = sqlx::query_as::<_, Step>("SELECT * FROM steps WHERE run_id = ? ORDER BY idx ASC")
        .bind(&run_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(steps))
}

pub async fn list_run_evidence(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<Evidence>>, (StatusCode, String)> {
    if !run_exists(&state, &run_id).await? {
        return Err((StatusCode::NOT_FOUND, "Run not found".to_string()));
    }
    let items =
        sqlx::query_as::<_, Evidence>("SELECT * FROM evidence WHERE run_id = ? ORDER BY created_at ASC")
            .bind(&run_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(items))
}

pub async fn create_evidence(
    State(state): State<AppState>,
    Json(payload): Json<CreateEvidenceRequest>,
) -> Result<Json<Evidence>, (StatusCode, String)> {
    if !run_exists(&state, &payload.run_id).await? {
        return Err((StatusCode::NOT_FOUND, "Run not found".to_string()));
    }
    if !step_exists(&state, &payload.step_id).await? {
        return Err((StatusCode::NOT_FOUND, "Step not found".to_string()));
    }
    if payload.kind.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "kind is required".to_string()));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let locator = format!("db://evidence/{}", id);
    let content = payload.content_json.clone().unwrap_or_default();
    let sha256 = format!("{:x}", sha2::Sha256::digest(content.as_bytes()));

    let ev = Evidence {
        id: id.clone(),
        run_id: payload.run_id,
        step_id: payload.step_id,
        kind: payload.kind,
        locator,
        sha256,
        content_json: payload.content_json,
        created_at,
    };

    sqlx::query(
        "INSERT INTO evidence (id, run_id, step_id, kind, locator, sha256, content_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&ev.id)
    .bind(&ev.run_id)
    .bind(&ev.step_id)
    .bind(&ev.kind)
    .bind(&ev.locator)
    .bind(&ev.sha256)
    .bind(&ev.content_json)
    .bind(&ev.created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ev))
}

pub async fn create_run(
    State(state): State<AppState>,
    Json(payload): Json<CreateRunRequest>,
) -> Result<Json<Run>, (StatusCode, String)> {
    if !agent_exists(&state, &payload.agent_id).await? {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }

    let scenario = match (&payload.scenario_id, &payload.test_id) {
        (Some(scenario_id), _) => state
            .scenarios
            .get_by_id(scenario_id)
            .ok_or((StatusCode::NOT_FOUND, "Scenario not found".to_string()))?,
        (None, Some(test_id)) => state
            .scenarios
            .get_by_test_id(test_id)
            .ok_or((StatusCode::NOT_FOUND, "Scenario not found".to_string()))?,
        (None, None) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "scenario_id or test_id is required".to_string(),
            ));
        }
    };

    let now = Utc::now().to_rfc3339();
    let run = Run {
        id: uuid::Uuid::new_v4().to_string(),
        agent_id: payload.agent_id,
        scenario_id: Some(scenario.scenario_id.clone()),
        test_id: scenario.test_id.clone(),
        params_json: payload.params_json,
        status: "pending".to_string(),
        result_json: None,
        created_at: now.clone(),
        updated_at: now,
    };

    sqlx::query(
        "INSERT INTO runs (id, agent_id, scenario_id, test_id, params_json, status, result_json, created_at, updated_at)\n         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&run.id)
    .bind(&run.agent_id)
    .bind(&run.scenario_id)
    .bind(&run.test_id)
    .bind(&run.params_json)
    .bind(&run.status)
    .bind(&run.result_json)
    .bind(&run.created_at)
    .bind(&run.updated_at)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for (idx, s) in scenario.steps.iter().enumerate() {
        let step_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO steps (id, run_id, idx, name, status, started_at, ended_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&step_id)
        .bind(&run.id)
        .bind(idx as i64)
        .bind(&s.name)
        .bind("PENDING")
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        for a in &s.assertions {
            let assertion_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO assertions (id, run_id, step_id, description, required, rule_type, kind, contains, status, evidence_refs_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&assertion_id)
            .bind(&run.id)
            .bind(&step_id)
            .bind(&a.description)
            .bind(a.required)
            .bind(&a.type_)
            .bind(&a.kind)
            .bind(&a.contains)
            .bind("PENDING")
            .bind(Option::<String>::None)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }

        let verdict_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO verdicts (id, run_id, step_id, verdict, reason_code, summary, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&verdict_id)
        .bind(&run.id)
        .bind(&step_id)
        .bind("IN_PROGRESS")
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(run))
}

pub async fn create_operator_action(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Json(payload): Json<OperatorActionRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = ?")
        .bind(&run_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Run not found".to_string()))?;

    if payload.type_.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "type is required".to_string()));
    }

    let scenario_id = run
        .scenario_id
        .clone()
        .ok_or((StatusCode::BAD_REQUEST, "run missing scenario_id".to_string()))?;
    let scenario = state
        .scenarios
        .get_by_id(&scenario_id)
        .ok_or((StatusCode::NOT_FOUND, "Scenario not found".to_string()))?;

    let mut allowed_action_ids = std::collections::HashSet::new();
    let mut allowed_choice_ids = std::collections::HashSet::new();
    for step in &scenario.steps {
        for a in &step.actions {
            allowed_action_ids.insert(a.action_id.clone());
        }
        for c in &step.choices {
            allowed_choice_ids.insert(c.choice_id.clone());
        }
    }

    if payload.type_ == "approve_action" {
        let action_id = payload
            .action_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string();
        if action_id.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "action_id is required".to_string()));
        }
        if !allowed_action_ids.contains(&action_id) {
            return Err((StatusCode::BAD_REQUEST, "action_id not allowed".to_string()));
        }
    } else if payload.type_ == "select_choice" {
        let choice_id = payload
            .choice_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string();
        if choice_id.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "choice_id is required".to_string()));
        }
        if !allowed_choice_ids.contains(&choice_id) {
            return Err((StatusCode::BAD_REQUEST, "choice_id not allowed".to_string()));
        }
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            "type must be approve_action|select_choice".to_string(),
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let ts = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO operator_actions (id, run_id, type, action_id, choice_id, note, ts) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&run_id)
    .bind(&payload.type_)
    .bind(&payload.action_id)
    .bind(&payload.choice_id)
    .bind(&payload.note)
    .bind(&ts)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

pub async fn list_operator_actions(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<OperatorAction>>, (StatusCode, String)> {
    if !run_exists(&state, &run_id).await? {
        return Err((StatusCode::NOT_FOUND, "Run not found".to_string()));
    }
    let items = sqlx::query_as::<_, OperatorAction>(
        "SELECT * FROM operator_actions WHERE run_id = ? ORDER BY ts ASC",
    )
    .bind(&run_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(items))
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

pub async fn list_run_events(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<Event>>, (StatusCode, String)> {
    if !run_exists(&state, &run_id).await? {
        return Err((StatusCode::NOT_FOUND, "Run not found".to_string()));
    }

    let events = sqlx::query_as::<_, Event>(
        "SELECT * FROM events WHERE run_id = ? ORDER BY ts ASC LIMIT 2000",
    )
    .bind(&run_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(events))
}

#[derive(Debug, Serialize)]
pub struct StepVerdictView {
    pub step: Step,
    pub verdict: VerdictRow,
    pub assertions: Vec<Assertion>,
}

#[derive(Debug, Serialize)]
pub struct RunVerdictView {
    pub run_id: String,
    pub steps: Vec<StepVerdictView>,
}

async fn evaluate_run(state: &AppState, run_id: &str) -> Result<(), (StatusCode, String)> {
    if !run_exists(state, run_id).await? {
        return Err((StatusCode::NOT_FOUND, "Run not found".to_string()));
    }

    let steps = sqlx::query_as::<_, Step>("SELECT * FROM steps WHERE run_id = ? ORDER BY idx ASC")
        .bind(run_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let evidence =
        sqlx::query_as::<_, Evidence>("SELECT * FROM evidence WHERE run_id = ? ORDER BY created_at ASC")
            .bind(run_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let events = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE run_id = ? ORDER BY ts ASC")
        .bind(run_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for step in steps {
        let mut assertions =
            sqlx::query_as::<_, Assertion>("SELECT * FROM assertions WHERE step_id = ?")
                .bind(&step.id)
                .fetch_all(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let mut required_total = 0;
        let mut required_failed = 0;
        let mut first_fail_reason: Option<String> = None;

        for a in &mut assertions {
            let rule_type = a.rule_type.clone().unwrap_or_else(|| "evidence_kind".to_string());

            if a.required {
                required_total += 1;
            }

            let (status, evidence_refs) = if rule_type == "evidence_kind" {
                let kind = a.kind.clone().unwrap_or_default();
                let matches: Vec<_> = evidence
                    .iter()
                    .filter(|e| e.step_id == step.id && e.kind == kind)
                    .map(|e| e.id.clone())
                    .collect();
                if !kind.is_empty() && !matches.is_empty() {
                    ("PASS".to_string(), matches)
                } else {
                    ("FAIL".to_string(), Vec::new())
                }
            } else if rule_type == "event_contains" {
                let needle = a.contains.clone().unwrap_or_default();
                if needle.is_empty() {
                    ("FAIL".to_string(), Vec::new())
                } else if let Some(ev) = events.iter().find(|e| e.message.contains(&needle)) {
                    let id = uuid::Uuid::new_v4().to_string();
                    let created_at = Utc::now().to_rfc3339();
                    let locator = format!("db://evidence/{}", id);
                    let content_json = serde_json::json!({
                        "type": "event_contains",
                        "event_id": ev.id,
                        "ts": ev.ts,
                        "level": ev.level,
                        "message": ev.message
                    })
                    .to_string();
                    let sha256 = format!("{:x}", sha2::Sha256::digest(content_json.as_bytes()));

                    sqlx::query(
                        "INSERT INTO evidence (id, run_id, step_id, kind, locator, sha256, content_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    )
                    .bind(&id)
                    .bind(run_id)
                    .bind(&step.id)
                    .bind("log")
                    .bind(&locator)
                    .bind(&sha256)
                    .bind(&content_json)
                    .bind(&created_at)
                    .execute(&state.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                    ("PASS".to_string(), vec![id])
                } else {
                    ("FAIL".to_string(), Vec::new())
                }
            } else {
                ("FAIL".to_string(), Vec::new())
            };

            let evidence_refs_json = serde_json::to_string(&evidence_refs)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            sqlx::query(
                "UPDATE assertions SET status = ?, evidence_refs_json = ? WHERE id = ?",
            )
            .bind(&status)
            .bind(&evidence_refs_json)
            .bind(&a.id)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            a.status = status.clone();
            a.evidence_refs_json = Some(evidence_refs_json);

            if a.required && status == "FAIL" {
                required_failed += 1;
                if first_fail_reason.is_none() {
                    first_fail_reason = Some(if rule_type == "evidence_kind" {
                        "FAIL_EVIDENCE_MISSING".to_string()
                    } else {
                        "FAIL_ASSERTION_FAILED".to_string()
                    });
                }
            }
        }

        let (step_status, reason_code, summary) = if required_total == 0 {
            (
                "FAIL".to_string(),
                Some("FAIL_EVIDENCE_MISSING".to_string()),
                Some("No required assertions defined".to_string()),
            )
        } else if required_failed > 0 {
            (
                "FAIL".to_string(),
                first_fail_reason,
                Some(format!("{}/{} required assertions failed", required_failed, required_total)),
            )
        } else {
            (
                "PASS".to_string(),
                None,
                Some(format!("All {} required assertions passed", required_total)),
            )
        };

        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE steps SET status = ?, ended_at = ? WHERE id = ?")
            .bind(&step_status)
            .bind(&now)
            .bind(&step.id)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sqlx::query(
            "UPDATE verdicts SET verdict = ?, reason_code = ?, summary = ?, updated_at = ? WHERE run_id = ? AND step_id = ?",
        )
        .bind(&step_status)
        .bind(&reason_code)
        .bind(&summary)
        .bind(&now)
        .bind(run_id)
        .bind(&step.id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(())
}

pub async fn get_run_verdict(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunVerdictView>, (StatusCode, String)> {
    evaluate_run(&state, &run_id).await?;

    let steps = sqlx::query_as::<_, Step>("SELECT * FROM steps WHERE run_id = ? ORDER BY idx ASC")
        .bind(&run_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut out_steps = Vec::with_capacity(steps.len());
    for step in steps {
        let verdict = sqlx::query_as::<_, VerdictRow>(
            "SELECT * FROM verdicts WHERE run_id = ? AND step_id = ? ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(&run_id)
        .bind(&step.id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Verdict not found".to_string()))?;

        let assertions = sqlx::query_as::<_, Assertion>("SELECT * FROM assertions WHERE step_id = ?")
            .bind(&step.id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        out_steps.push(StepVerdictView {
            step,
            verdict,
            assertions,
        });
    }

    Ok(Json(RunVerdictView {
        run_id,
        steps: out_steps,
    }))
}
