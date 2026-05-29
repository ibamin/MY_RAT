use crate::fingerprint::FingerprintMatcher;
use crate::scenarios::{ScenarioCatalog, ScenarioDef, ScenarioMeta};
use crate::models::{
    Achievement, AchievementCheckResponse, AchievementStatus, Agent, AgentBuild, AgentTag,
    Assertion, BuildResponse, CreateBuildRequest,
    CreateEvidenceRequest, CreateEventRequest,
    CreateGroupRequest, CreateGroupRunsRequest, CreateGroupRunsResponse, CreateRunRequest, Evidence,
    Event, FingerprintMatchRequest, FingerprintMatchResponse, Group, OperatorAction,
    OperatorActionRequest, RegisterRequest, ReplayData, ReplayDataResponse, Run,
    SetAgentGroupRequest, SetAgentTagRequest, Step, StepCompleteRequest, UpdateRunResultRequest,
    VerdictRow,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use chrono::Utc;
use sha2::Digest;
use sqlx::{FromRow, Row, SqlitePool};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub matcher: FingerprintMatcher,
    pub scenarios: ScenarioCatalog,
}

#[derive(Debug, Deserialize)]
struct ScenarioCountRequirement {
    count: Option<i64>,
    depends_on: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct VerdictStreakRequirement {
    streak: Option<i64>,
    depends_on: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct SpecificScenarioRequirement {
    scenario_id: Option<String>,
    test_id: Option<String>,
    count: Option<i64>,
    depends_on: Option<Vec<String>>,
}

#[derive(Debug, FromRow)]
struct AchievementStatusRow {
    id: String,
    name: String,
    description: String,
    category: String,
    icon: String,
    requirement_type: String,
    requirement_value: String,
    created_at: String,
    unlocked_at: Option<String>,
    progress: i64,
}

pub(crate) fn parse_dependencies(requirement_type: &str, requirement_value: &str) -> Vec<String> {
    if requirement_type == "scenario_count" {
        if let Ok(parsed) = serde_json::from_str::<ScenarioCountRequirement>(requirement_value) {
            return parsed.depends_on.unwrap_or_default();
        }
    }

    if requirement_type == "verdict_streak" {
        if let Ok(parsed) = serde_json::from_str::<VerdictStreakRequirement>(requirement_value) {
            return parsed.depends_on.unwrap_or_default();
        }
    }

    if requirement_type == "specific_scenario" {
        if let Ok(parsed) = serde_json::from_str::<SpecificScenarioRequirement>(requirement_value) {
            return parsed.depends_on.unwrap_or_default();
        }
    }

    Vec::new()
}

async fn dependencies_unlocked(
    state: &AppState,
    dependency_ids: &[String],
) -> Result<bool, (StatusCode, String)> {
    if dependency_ids.is_empty() {
        return Ok(true);
    }

    for dep_id in dependency_ids {
        let unlocked_at = sqlx::query_scalar::<_, Option<String>>(
            "SELECT unlocked_at FROM user_achievements WHERE achievement_id = ?",
        )
        .bind(dep_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .flatten();

        if unlocked_at.is_none() {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn list_passed_runs(state: &AppState) -> Result<Vec<Run>, (StatusCode, String)> {
    let runs = sqlx::query_as::<_, Run>(
        "SELECT r.*
         FROM runs r
         JOIN verdicts v ON v.run_id = r.id
         WHERE r.status = 'completed'
         GROUP BY r.id
         HAVING COUNT(v.id) > 0 AND SUM(CASE WHEN v.verdict = 'PASS' THEN 1 ELSE 0 END) = COUNT(v.id)
         ORDER BY r.created_at ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(runs)
}

async fn calculate_pass_streak(state: &AppState) -> Result<i64, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT r.id,
                CASE
                    WHEN COUNT(v.id) > 0
                         AND SUM(CASE WHEN v.verdict = 'PASS' THEN 1 ELSE 0 END) = COUNT(v.id)
                    THEN 1
                    ELSE 0
                END AS is_pass
         FROM runs r
         LEFT JOIN verdicts v ON v.run_id = r.id
         WHERE r.status = 'completed'
         GROUP BY r.id
         ORDER BY r.created_at ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut streak = 0_i64;
    for row in rows {
        let is_pass: i64 = row.get("is_pass");
        if is_pass == 1 {
            streak += 1;
        } else {
            streak = 0;
        }
    }

    Ok(streak)
}

pub(crate) fn calc_percent(current: i64, target: i64) -> i64 {
    let safe_target = target.max(1);
    let pct = current.saturating_mul(100) / safe_target;
    pct.clamp(0, 100)
}

pub(crate) fn evaluate_achievement_progress(achievement: &Achievement, passed_runs: &[Run], streak: i64) -> i64 {
    if achievement.requirement_type == "scenario_count" {
        if let Ok(parsed) = serde_json::from_str::<ScenarioCountRequirement>(&achievement.requirement_value)
        {
            return calc_percent(passed_runs.len() as i64, parsed.count.unwrap_or(1));
        }
        return 0;
    }

    if achievement.requirement_type == "verdict_streak" {
        if let Ok(parsed) = serde_json::from_str::<VerdictStreakRequirement>(&achievement.requirement_value)
        {
            return calc_percent(streak, parsed.streak.unwrap_or(1));
        }
        return 0;
    }

    if achievement.requirement_type == "specific_scenario" {
        if let Ok(parsed) = serde_json::from_str::<SpecificScenarioRequirement>(&achievement.requirement_value)
        {
            let target_count = parsed.count.unwrap_or(1).max(1);
            let matched = passed_runs
                .iter()
                .filter(|run| {
                    parsed
                        .scenario_id
                        .as_deref()
                        .map(|scenario_id| run.scenario_id.as_deref() == Some(scenario_id))
                        .unwrap_or(false)
                        || parsed
                            .test_id
                            .as_deref()
                            .map(|test_id| run.test_id == test_id)
                            .unwrap_or(false)
                })
                .count() as i64;
            return calc_percent(matched, target_count);
        }
        return 0;
    }

    0
}

async fn upsert_user_achievement(
    state: &AppState,
    achievement_id: &str,
    progress: i64,
    unlock: bool,
) -> Result<bool, (StatusCode, String)> {
    let existing = sqlx::query_as::<_, crate::models::UserAchievement>(
        "SELECT * FROM user_achievements WHERE achievement_id = ?",
    )
    .bind(achievement_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let clamped_progress = progress.clamp(0, 100);
    let now = Utc::now().to_rfc3339();

    if let Some(row) = existing {
        let should_unlock_now = unlock && row.unlocked_at.is_none();
        let unlocked_at = if row.unlocked_at.is_some() || unlock {
            row.unlocked_at.or(Some(now.clone()))
        } else {
            None
        };

        sqlx::query("UPDATE user_achievements SET progress = ?, unlocked_at = ? WHERE id = ?")
            .bind(clamped_progress)
            .bind(&unlocked_at)
            .bind(&row.id)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        return Ok(should_unlock_now);
    }

    let unlocked_at = if unlock { Some(now) } else { None };
    sqlx::query(
        "INSERT INTO user_achievements (id, achievement_id, unlocked_at, progress) VALUES (?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(achievement_id)
    .bind(&unlocked_at)
    .bind(clamped_progress)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(unlock)
}

async fn check_achievements_internal(
    state: &AppState,
) -> Result<AchievementCheckResponse, (StatusCode, String)> {
    let achievements = sqlx::query_as::<_, Achievement>(
        "SELECT * FROM achievements ORDER BY category ASC, created_at ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let passed_runs = list_passed_runs(state).await?;
    let streak = calculate_pass_streak(state).await?;
    let mut unlocked_achievement_ids = Vec::new();

    for achievement in achievements {
        let dependencies = parse_dependencies(&achievement.requirement_type, &achievement.requirement_value);
        let dependencies_met = dependencies_unlocked(state, &dependencies).await?;
        let progress = evaluate_achievement_progress(&achievement, &passed_runs, streak);
        let unlock = dependencies_met && progress >= 100;

        let did_unlock = upsert_user_achievement(state, &achievement.id, progress, unlock).await?;
        if did_unlock {
            unlocked_achievement_ids.push(achievement.id);
        }
    }

    Ok(AchievementCheckResponse {
        checked_runs: passed_runs.len() as i64,
        unlocked_achievement_ids,
    })
}

pub async fn validate_scenario(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<crate::scenarios::ValidationResult>, (StatusCode, String)> {
    let bytes = serde_json::to_vec(&payload)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(crate::scenarios::validate_scenario_json(&bytes)))
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

async fn get_agent(state: &AppState, agent_id: &str) -> Result<Option<Agent>, (StatusCode, String)> {
    let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
        .bind(agent_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(agent)
}

async fn agent_is_approved(state: &AppState, agent_id: &str) -> Result<bool, (StatusCode, String)> {
    let row: Option<(String,)> = sqlx::query_as("SELECT approval_status FROM agents WHERE id = ?")
        .bind(agent_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(row.map(|(s,)| s == "approved").unwrap_or(false))
}

async fn agent_is_blocked(state: &AppState, agent_id: &str) -> Result<bool, (StatusCode, String)> {
    let row: Option<(String,)> = sqlx::query_as("SELECT approval_status FROM agents WHERE id = ?")
        .bind(agent_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(row.map(|(s,)| s == "blocked").unwrap_or(false))
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

async fn step_exists_by_scenario_step_id(
    state: &AppState,
    run_id: &str,
    scenario_step_id: &str,
) -> Result<bool, (StatusCode, String)> {
    let row = sqlx::query("SELECT 1 FROM steps WHERE run_id = ? AND scenario_step_id = ?")
        .bind(run_id)
        .bind(scenario_step_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(row.is_some())
}

#[allow(clippy::too_many_arguments)]
async fn insert_step_plan(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    run_id: &str,
    idx: i64,
    scenario_step_id: &str,
    name: &str,
    status: &str,
    executor_info: Option<&str>,
    assertions: &[crate::scenarios::ScenarioAssertionDef],
) -> Result<(), (StatusCode, String)> {
    let step_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO steps (id, run_id, scenario_step_id, idx, name, status, executor_info, started_at, ended_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&step_id)
    .bind(run_id)
    .bind(scenario_step_id)
    .bind(idx)
    .bind(name)
    .bind(status)
    .bind(executor_info)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .execute(&mut **tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for a in assertions {
        let assertion_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO assertions (id, run_id, step_id, description, required, rule_type, kind, contains, status, evidence_refs_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&assertion_id)
        .bind(run_id)
        .bind(&step_id)
        .bind(&a.description)
        .bind(a.required)
        .bind(&a.type_)
        .bind(&a.kind)
        .bind(&a.contains)
        .bind("PENDING")
        .bind(Option::<String>::None)
        .execute(&mut **tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let verdict_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO verdicts (id, run_id, step_id, verdict, reason_code, summary, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&verdict_id)
    .bind(run_id)
    .bind(&step_id)
    .bind("IN_PROGRESS")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(Utc::now().to_rfc3339())
    .execute(&mut **tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}

fn build_executor_info(step: &crate::scenarios::ScenarioStepDef) -> Option<String> {
    if step.executor.is_none() && step.command.is_none() && step.args.is_none() {
        return None;
    }
    let mut info = serde_json::Map::new();
    if let Some(ref executor) = step.executor {
        info.insert("executor".into(), serde_json::Value::String(executor.clone()));
    }
    if let Some(ref command) = step.command {
        info.insert("command".into(), serde_json::Value::String(command.clone()));
    }
    if let Some(ref args) = step.args {
        info.insert(
            "args".into(),
            serde_json::Value::Array(args.iter().map(|a| serde_json::Value::String(a.clone())).collect()),
        );
    }
    Some(serde_json::to_string(&info).unwrap_or_default())
}

async fn ensure_unlocked_steps(
    state: &AppState,
    run: &Run,
    scenario: &ScenarioDef,
) -> Result<(), (StatusCode, String)> {
    let selected_choices: Vec<String> = sqlx::query(
        "SELECT choice_id FROM operator_actions WHERE run_id = ? AND type = 'select_choice' AND choice_id IS NOT NULL ORDER BY ts ASC",
    )
    .bind(&run.id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .into_iter()
    .filter_map(|r| r.get::<Option<String>, _>(0))
    .collect();

    let mut tx = state.pool.begin().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for (idx, s) in scenario.steps.iter().enumerate() {
        let req = s.requires_choice_id.as_deref().unwrap_or("");
        if !req.is_empty() && !selected_choices.iter().any(|c| c == req) {
            continue;
        }
        if step_exists_by_scenario_step_id(state, &run.id, &s.step_id).await? {
            continue;
        }
        let exec_info = build_executor_info(s);
        insert_step_plan(&mut tx, &run.id, idx as i64, &s.step_id, &s.name, "READY", exec_info.as_deref(), &s.assertions).await?;
    }

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}

pub async fn register_agent(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let now = Utc::now();

    let last_seen = now.to_rfc3339();
    let requested_id = payload.id.as_deref().unwrap_or("").trim();

    if !requested_id.is_empty() {
        if agent_exists(&state, requested_id).await? {
            sqlx::query(
                "UPDATE agents SET hostname = ?, ip = ?, os = ?, arch = ?, user = ?, last_seen = ?, status = 'online' WHERE id = ?",
            )
            .bind(&payload.hostname)
            .bind(&payload.ip)
            .bind(&payload.os)
            .bind(&payload.arch)
            .bind(&payload.user)
            .bind(&last_seen)
            .bind(requested_id)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let agent = get_agent(&state, requested_id)
                .await?
                .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Agent missing after update".to_string()))?;
            return Ok(Json(agent));
        }

        let agent = Agent {
            id: requested_id.to_string(),
            hostname: payload.hostname,
            ip: payload.ip,
            os: payload.os,
            arch: payload.arch,
            user: payload.user,
            last_seen,
            status: "online".to_string(),
            approval_status: "pending".to_string(),
        };

        sqlx::query(
            "INSERT INTO agents (id, hostname, ip, os, arch, user, last_seen, status, approval_status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&agent.id)
        .bind(&agent.hostname)
        .bind(&agent.ip)
        .bind(&agent.os)
        .bind(&agent.arch)
        .bind(&agent.user)
        .bind(&agent.last_seen)
        .bind(&agent.status)
        .bind(&agent.approval_status)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        return Ok(Json(agent));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let agent = Agent {
        id,
        hostname: payload.hostname,
        ip: payload.ip,
        os: payload.os,
        arch: payload.arch,
        user: payload.user,
        last_seen,
        status: "online".to_string(),
        approval_status: "pending".to_string(),
    };

    sqlx::query(
        "INSERT INTO agents (id, hostname, ip, os, arch, user, last_seen, status, approval_status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&agent.id)
    .bind(&agent.hostname)
    .bind(&agent.ip)
    .bind(&agent.os)
    .bind(&agent.arch)
    .bind(&agent.user)
    .bind(&agent.last_seen)
    .bind(&agent.status)
    .bind(&agent.approval_status)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agent))
}

pub async fn heartbeat(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    if agent_is_blocked(&state, &id).await? {
        return Err((StatusCode::FORBIDDEN, "Agent is blocked".to_string()));
    }
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
    let agents = sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY last_seen DESC LIMIT 200")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agents))
}

pub async fn list_pending_agents(
    State(state): State<AppState>,
) -> Result<Json<Vec<Agent>>, (StatusCode, String)> {
    let agents = sqlx::query_as::<_, Agent>(
        "SELECT * FROM agents WHERE approval_status = 'pending' ORDER BY last_seen DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agents))
}

pub async fn approve_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query("UPDATE agents SET approval_status = 'approved' WHERE id = ?")
        .bind(&agent_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }
    Ok(StatusCode::OK)
}

pub async fn block_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let res = sqlx::query(
        "UPDATE agents SET approval_status = 'blocked', status = 'blocked' WHERE id = ?",
    )
    .bind(&agent_id)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }
    Ok(StatusCode::OK)
}

pub async fn list_agent_runs(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<Vec<Run>>, (StatusCode, String)> {
    if !agent_exists(&state, &agent_id).await? {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }
    let runs = sqlx::query_as::<_, Run>(
        "SELECT * FROM runs WHERE agent_id = ? ORDER BY created_at DESC LIMIT 200",
    )
    .bind(&agent_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(runs))
}

pub async fn list_groups(
    State(state): State<AppState>,
) -> Result<Json<Vec<Group>>, (StatusCode, String)> {
    let items = sqlx::query_as::<_, Group>("SELECT * FROM groups ORDER BY created_at DESC LIMIT 200")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(items))
}

pub async fn create_group(
    State(state): State<AppState>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<Json<Group>, (StatusCode, String)> {
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required".to_string()));
    }

    let group = Group {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.name.trim().to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    sqlx::query("INSERT INTO groups (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&group.id)
        .bind(&group.name)
        .bind(&group.created_at)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(group))
}

pub async fn assign_agent_to_group(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Json(payload): Json<SetAgentGroupRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if !agent_exists(&state, &payload.agent_id).await? {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM groups WHERE id = ?")
        .bind(&group_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if exists == 0 {
        return Err((StatusCode::NOT_FOUND, "Group not found".to_string()));
    }

    let created_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO agent_groups (agent_id, group_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(&payload.agent_id)
    .bind(&group_id)
    .bind(&created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

pub async fn unassign_agent_from_group(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Json(payload): Json<SetAgentGroupRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM agent_groups WHERE agent_id = ? AND group_id = ?")
        .bind(&payload.agent_id)
        .bind(&group_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn list_group_agents(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
) -> Result<Json<Vec<Agent>>, (StatusCode, String)> {
    let agents = sqlx::query_as::<_, Agent>(
        "SELECT a.* FROM agents a JOIN agent_groups ag ON ag.agent_id = a.id WHERE ag.group_id = ? ORDER BY a.hostname ASC",
    )
    .bind(&group_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(agents))
}

pub async fn create_group_runs(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Json(payload): Json<CreateGroupRunsRequest>,
) -> Result<Json<CreateGroupRunsResponse>, (StatusCode, String)> {
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

    let group_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM groups WHERE id = ?")
        .bind(&group_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if group_exists == 0 {
        return Err((StatusCode::NOT_FOUND, "Group not found".to_string()));
    }

    let agent_ids: Vec<String> = sqlx::query_scalar(
        "SELECT a.id FROM agents a JOIN agent_groups ag ON ag.agent_id = a.id WHERE ag.group_id = ? AND a.approval_status = 'approved' ORDER BY a.hostname ASC",
    )
    .bind(&group_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if agent_ids.is_empty() {
        return Ok(Json(CreateGroupRunsResponse { runs: Vec::new() }));
    }

    let mut out = Vec::with_capacity(agent_ids.len());
    for agent_id in agent_ids {
        let now = Utc::now().to_rfc3339();
        let run_id = uuid::Uuid::new_v4().to_string();
        let replay_seed = {
            let digest = sha2::Sha256::digest(run_id.as_bytes());
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(&digest[..8]);
            u64::from_le_bytes(bytes).to_string()
        };
        let run = Run {
            id: run_id,
            agent_id,
            scenario_id: Some(scenario.scenario_id.clone()),
            test_id: scenario.test_id.clone(),
            params_json: payload.params_json.clone(),
            replay_seed: Some(replay_seed),
            status: "pending".to_string(),
            result_json: None,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut tx = state.pool.begin().await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sqlx::query(
            "INSERT INTO runs (id, agent_id, scenario_id, test_id, params_json, replay_seed, status, result_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&run.id)
        .bind(&run.agent_id)
        .bind(&run.scenario_id)
        .bind(&run.test_id)
        .bind(&run.params_json)
        .bind(&run.replay_seed)
        .bind(&run.status)
        .bind(&run.result_json)
        .bind(&run.created_at)
        .bind(&run.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let mut first_step_inserted = false;
        for (idx, s) in scenario.steps.iter().enumerate() {
            if s.requires_choice_id.is_some() {
                continue;
            }
            let status = if !first_step_inserted { "READY" } else { "LOCKED" };
            first_step_inserted = true;
            let exec_info = build_executor_info(s);
            insert_step_plan(&mut tx, &run.id, idx as i64, &s.step_id, &s.name, status, exec_info.as_deref(), &s.assertions)
                .await?;
        }

        tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        out.push(run);
    }

    Ok(Json(CreateGroupRunsResponse { runs: out }))
}

pub async fn list_agent_groups(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<Vec<Group>>, (StatusCode, String)> {
    if !agent_exists(&state, &agent_id).await? {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }
    let groups = sqlx::query_as::<_, Group>(
        "SELECT g.* FROM groups g JOIN agent_groups ag ON ag.group_id = g.id WHERE ag.agent_id = ? ORDER BY g.name ASC",
    )
    .bind(&agent_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(groups))
}

pub async fn list_agent_tags(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<Vec<AgentTag>>, (StatusCode, String)> {
    if !agent_exists(&state, &agent_id).await? {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }
    let tags = sqlx::query_as::<_, AgentTag>(
        "SELECT * FROM agent_tags WHERE agent_id = ? ORDER BY tag ASC",
    )
    .bind(&agent_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(tags))
}

pub async fn add_agent_tag(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(payload): Json<SetAgentTagRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if !agent_exists(&state, &agent_id).await? {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }
    if payload.tag.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "tag is required".to_string()));
    }
    let created_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO agent_tags (agent_id, tag, created_at) VALUES (?, ?, ?)",
    )
    .bind(&agent_id)
    .bind(payload.tag.trim())
    .bind(&created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn remove_agent_tag(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(payload): Json<SetAgentTagRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if payload.tag.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "tag is required".to_string()));
    }
    sqlx::query("DELETE FROM agent_tags WHERE agent_id = ? AND tag = ?")
        .bind(&agent_id)
        .bind(payload.tag.trim())
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn fingerprint_match(
    State(state): State<AppState>,
    Json(payload): Json<FingerprintMatchRequest>,
) -> Result<Json<FingerprintMatchResponse>, (StatusCode, String)> {
    let limit = payload.limit.unwrap_or(10);
    let candidates = state.matcher.match_banner(&payload.banner, limit);
    Ok(Json(FingerprintMatchResponse { candidates }))
}

pub async fn list_achievements(
    State(state): State<AppState>,
) -> Result<Json<Vec<AchievementStatus>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AchievementStatusRow>(
        "SELECT
            a.id,
            a.name,
            a.description,
            a.category,
            a.icon,
            a.requirement_type,
            a.requirement_value,
            a.created_at,
            ua.unlocked_at,
            COALESCE(ua.progress, 0) AS progress
         FROM achievements a
         LEFT JOIN user_achievements ua ON ua.achievement_id = a.id
         ORDER BY a.category ASC, a.created_at ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let items = rows
        .into_iter()
        .map(|row| AchievementStatus {
            achievement: Achievement {
                id: row.id,
                name: row.name,
                description: row.description,
                category: row.category,
                icon: row.icon,
                requirement_type: row.requirement_type,
                requirement_value: row.requirement_value,
                created_at: row.created_at,
            },
            unlocked: row.unlocked_at.is_some(),
            unlocked_at: row.unlocked_at,
            progress: row.progress.clamp(0, 100),
        })
        .collect();

    Ok(Json(items))
}

pub async fn get_achievement_progress(
    State(state): State<AppState>,
) -> Result<Json<Vec<AchievementStatus>>, (StatusCode, String)> {
    list_achievements(State(state)).await
}

pub async fn check_achievements(
    State(state): State<AppState>,
) -> Result<Json<AchievementCheckResponse>, (StatusCode, String)> {
    let response = check_achievements_internal(&state).await?;
    Ok(Json(response))
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
    let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = ?")
        .bind(&run_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Run not found".to_string()))?;

    let scenario_id = run
        .scenario_id
        .clone()
        .ok_or((StatusCode::BAD_REQUEST, "run missing scenario_id".to_string()))?;
    let scenario = state
        .scenarios
        .get_by_id(&scenario_id)
        .ok_or((StatusCode::NOT_FOUND, "Scenario not found".to_string()))?;

    ensure_unlocked_steps(&state, &run, &scenario).await?;
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
    if !agent_is_approved(&state, &payload.agent_id).await? {
        return Err((
            StatusCode::FORBIDDEN,
            "Agent not approved".to_string(),
        ));
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
    let run_id = uuid::Uuid::new_v4().to_string();
    let replay_seed = {
        let digest = sha2::Sha256::digest(run_id.as_bytes());
        let mut bytes = [0_u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        u64::from_le_bytes(bytes).to_string()
    };
    let run = Run {
        id: run_id,
        agent_id: payload.agent_id,
        scenario_id: Some(scenario.scenario_id.clone()),
        test_id: scenario.test_id.clone(),
        params_json: payload.params_json,
        replay_seed: Some(replay_seed),
        status: "pending".to_string(),
        result_json: None,
        created_at: now.clone(),
        updated_at: now,
    };

    let mut tx = state.pool.begin().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "INSERT INTO runs (id, agent_id, scenario_id, test_id, params_json, replay_seed, status, result_json, created_at, updated_at)\n         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&run.id)
    .bind(&run.agent_id)
    .bind(&run.scenario_id)
    .bind(&run.test_id)
    .bind(&run.params_json)
    .bind(&run.replay_seed)
    .bind(&run.status)
    .bind(&run.result_json)
    .bind(&run.created_at)
    .bind(&run.updated_at)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut first_step_inserted = false;
    for (idx, s) in scenario.steps.iter().enumerate() {
        if s.requires_choice_id.is_some() {
            continue;
        }
        let status = if !first_step_inserted { "READY" } else { "LOCKED" };
        first_step_inserted = true;
        let exec_info = build_executor_info(s);
        insert_step_plan(&mut tx, &run.id, idx as i64, &s.step_id, &s.name, status, exec_info.as_deref(), &s.assertions).await?;
    }

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(run))
}

pub async fn replay_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<Run>, (StatusCode, String)> {
    let source_run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = ?")
        .bind(&run_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Run not found".to_string()))?;

    if source_run.status != "completed" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Only completed runs can be replayed".to_string(),
        ));
    }

    let scenario_id = source_run
        .scenario_id
        .clone()
        .ok_or((StatusCode::BAD_REQUEST, "run missing scenario_id".to_string()))?;
    let scenario = state
        .scenarios
        .get_by_id(&scenario_id)
        .ok_or((StatusCode::NOT_FOUND, "Scenario not found".to_string()))?;

    let now = Utc::now().to_rfc3339();
    let replayed_run = Run {
        id: uuid::Uuid::new_v4().to_string(),
        agent_id: source_run.agent_id,
        scenario_id: source_run.scenario_id,
        test_id: source_run.test_id,
        params_json: source_run.params_json,
        replay_seed: source_run.replay_seed,
        status: "pending".to_string(),
        result_json: None,
        created_at: now.clone(),
        updated_at: now,
    };

    let mut tx = state.pool.begin().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "INSERT INTO runs (id, agent_id, scenario_id, test_id, params_json, replay_seed, status, result_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&replayed_run.id)
    .bind(&replayed_run.agent_id)
    .bind(&replayed_run.scenario_id)
    .bind(&replayed_run.test_id)
    .bind(&replayed_run.params_json)
    .bind(&replayed_run.replay_seed)
    .bind(&replayed_run.status)
    .bind(&replayed_run.result_json)
    .bind(&replayed_run.created_at)
    .bind(&replayed_run.updated_at)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut first_step_inserted = false;
    for (idx, s) in scenario.steps.iter().enumerate() {
        if s.requires_choice_id.is_some() {
            continue;
        }
        let status = if !first_step_inserted { "READY" } else { "LOCKED" };
        first_step_inserted = true;
        let exec_info = build_executor_info(s);
        insert_step_plan(
            &mut tx,
            &replayed_run.id,
            idx as i64,
            &s.step_id,
            &s.name,
            status,
            exec_info.as_deref(),
            &s.assertions,
        )
        .await?;
    }

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(replayed_run))
}

pub async fn get_replay_data(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<ReplayDataResponse>, (StatusCode, String)> {
    let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = ?")
        .bind(&run_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Run not found".to_string()))?;

    let steps = sqlx::query_as::<_, Step>("SELECT * FROM steps WHERE run_id = ? ORDER BY idx ASC")
        .bind(&run_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let events = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE run_id = ? ORDER BY ts ASC")
        .bind(&run_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let evidence =
        sqlx::query_as::<_, Evidence>("SELECT * FROM evidence WHERE run_id = ? ORDER BY created_at ASC")
            .bind(&run_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let verdicts = sqlx::query_as::<_, VerdictRow>(
        "SELECT * FROM verdicts WHERE run_id = ? ORDER BY updated_at ASC",
    )
    .bind(&run_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ReplayDataResponse {
        replay_data: ReplayData {
            run,
            steps,
            events,
            evidence,
            verdicts,
        },
    }))
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

    ensure_unlocked_steps(&state, &run, &scenario).await?;
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
    if !agent_is_approved(&state, &agent_id).await? {
        return Err((
            StatusCode::FORBIDDEN,
            "Agent not approved".to_string(),
        ));
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
    const VALID_STATUSES: &[&str] = &["completed", "failed", "error"];
    if !VALID_STATUSES.contains(&payload.status.as_str()) {
        return Err((StatusCode::BAD_REQUEST, format!("Invalid status '{}'. Must be one of: completed, failed, error", payload.status)));
    }

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

pub async fn complete_step(
    State(state): State<AppState>,
    Path((run_id, step_id)): Path<(String, String)>,
    Json(payload): Json<StepCompleteRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let now = Utc::now().to_rfc3339();

    let step = sqlx::query_as::<_, Step>("SELECT * FROM steps WHERE id = ? AND run_id = ?")
        .bind(&step_id)
        .bind(&run_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Step not found".to_string()))?;

    if step.status != "READY" && step.status != "IN_PROGRESS" {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Step status is '{}', expected READY or IN_PROGRESS", step.status),
        ));
    }

    let new_status = if payload.success { "COMPLETED" } else { "FAILED" };
    sqlx::query("UPDATE steps SET status = ?, ended_at = ? WHERE id = ?")
        .bind(new_status)
        .bind(&now)
        .bind(&step_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(ref result_json) = payload.result_json {
        let run = sqlx::query_as::<_, Run>("SELECT * FROM runs WHERE id = ?")
            .bind(&run_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if let Some(run) = run {
            let event_id = uuid::Uuid::new_v4().to_string();
            let msg = format!("step_result step_id={} status={} result={}", step_id, new_status, result_json);
            sqlx::query("INSERT INTO events (id, run_id, agent_id, level, message, ts) VALUES (?,?,?,?,?,?)")
                .bind(&event_id)
                .bind(&run_id)
                .bind(&run.agent_id)
                .bind("info")
                .bind(&msg)
                .bind(&now)
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }

    if payload.success {
        let next_locked = sqlx::query_as::<_, Step>(
            "SELECT * FROM steps WHERE run_id = ? AND status = 'LOCKED' ORDER BY idx ASC LIMIT 1",
        )
        .bind(&run_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if let Some(next_step) = next_locked {
            sqlx::query("UPDATE steps SET status = 'READY', started_at = ? WHERE id = ?")
                .bind(&now)
                .bind(&next_step.id)
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }

    evaluate_run(&state, &run_id).await?;

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

    let total_verdicts = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM verdicts WHERE run_id = ?")
        .bind(run_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let non_pass_verdicts =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM verdicts WHERE run_id = ? AND verdict != 'PASS'")
            .bind(run_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if total_verdicts > 0 && non_pass_verdicts == 0 {
        let _ = check_achievements_internal(state).await;
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

static BUILD_SEMAPHORE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

pub async fn build_agent(
    State(state): State<AppState>,
    Json(payload): Json<CreateBuildRequest>,
) -> Result<Json<BuildResponse>, (StatusCode, String)> {
    let _permit = BUILD_SEMAPHORE
        .try_acquire()
        .map_err(|_| (StatusCode::TOO_MANY_REQUESTS, "Build already in progress".to_string()))?;
    let target_platform = payload
        .target_platform
        .unwrap_or_else(|| "windows-x86_64".to_string());
    let server_url = payload
        .server_url
        .unwrap_or_else(|| "http://127.0.0.1:3000".to_string());
    let sleep_sec = payload.sleep_sec.unwrap_or(5);

    if !server_url.starts_with("http://") && !server_url.starts_with("https://") {
        return Err((
            StatusCode::BAD_REQUEST,
            "server_url must start with http:// or https://".to_string(),
        ));
    }
    if !(1..=3600).contains(&sleep_sec) {
        return Err((
            StatusCode::BAD_REQUEST,
            "sleep_sec must be between 1 and 3600".to_string(),
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let guid = uuid::Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO agent_builds (id, guid, target_platform, server_url, sleep_sec, build_status, binary_path, error_message, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&guid)
    .bind(&target_platform)
    .bind(&server_url)
    .bind(sleep_sec)
    .bind("building")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(&created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let server_dir = std::env::current_dir()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get current dir: {}", e)))?;
    let agent_dir = server_dir.join("..").join("agent");
    let guid_clone = guid.clone();
    let server_url_clone = server_url.clone();

    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new("cargo")
            .current_dir(&agent_dir)
            .env("AGENT_GUID", &guid_clone)
            .env("AGENT_SERVER_URL", &server_url_clone)
            .env("AGENT_SLEEP_SEC", sleep_sec.to_string())
            .args(["build", "--release"])
            .output()
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Build task failed: {}", e),
        )
    })?
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Build process failed: {}", e),
        )
    })?;

    if output.status.success() {
        let build_dir = server_dir.join("builds").join(&guid);
        tokio::fs::create_dir_all(&build_dir)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create build dir: {}", e)))?;

        let source_binary = server_dir
            .join("..")
            .join("agent")
            .join("target")
            .join("release")
            .join("agent.exe");
        let dest_binary = build_dir.join("agent.exe");

        tokio::fs::copy(&source_binary, &dest_binary)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to copy binary: {}", e)))?;

        let binary_path = format!("builds/{}/agent.exe", guid);
        sqlx::query("UPDATE agent_builds SET build_status = 'completed', binary_path = ?, error_message = NULL WHERE guid = ?")
            .bind(&binary_path)
            .bind(&guid)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let error_message = if stderr.is_empty() {
            "Unknown build error".to_string()
        } else {
            stderr
        };

        sqlx::query(
            "UPDATE agent_builds SET build_status = 'failed', binary_path = NULL, error_message = ? WHERE guid = ?",
        )
        .bind(&error_message)
        .bind(&guid)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let build = sqlx::query_as::<_, AgentBuild>("SELECT * FROM agent_builds WHERE guid = ?")
        .bind(&guid)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Build not found after update".to_string(),
        ))?;

    let download_url = if build.build_status == "completed" {
        Some(format!("/api/agents/builds/{}/download", build.guid))
    } else {
        None
    };

    Ok(Json(BuildResponse {
        build,
        download_url,
    }))
}

pub async fn list_builds(
    State(state): State<AppState>,
) -> Result<Json<Vec<AgentBuild>>, (StatusCode, String)> {
    let builds = sqlx::query_as::<_, AgentBuild>(
        "SELECT * FROM agent_builds ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(builds))
}

pub async fn get_build(
    State(state): State<AppState>,
    Path(guid): Path<String>,
) -> Result<Json<BuildResponse>, (StatusCode, String)> {
    let build = sqlx::query_as::<_, AgentBuild>("SELECT * FROM agent_builds WHERE guid = ?")
        .bind(&guid)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Build not found".to_string()))?;

    let download_url = if build.build_status == "completed" {
        Some(format!("/api/agents/builds/{}/download", build.guid))
    } else {
        None
    };

    Ok(Json(BuildResponse {
        build,
        download_url,
    }))
}

pub async fn download_build(
    State(state): State<AppState>,
    Path(guid): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let build = sqlx::query_as::<_, AgentBuild>("SELECT * FROM agent_builds WHERE guid = ?")
        .bind(&guid)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Build not found".to_string()))?;

    if build.build_status != "completed" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Build is not completed".to_string(),
        ));
    }

    let binary_path = build.binary_path.ok_or((
        StatusCode::BAD_REQUEST,
        "Build binary path is missing".to_string(),
    ))?;

    let bytes = tokio::fs::read(&binary_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read binary: {}", e)))?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"agent.exe\"",
            ),
        ],
        bytes,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Achievement, Run};

    fn make_achievement(req_type: &str, req_value: &str) -> Achievement {
        Achievement {
            id: "ach-1".into(),
            name: "Test".into(),
            description: "".into(),
            category: "general".into(),
            icon: "star".into(),
            requirement_type: req_type.into(),
            requirement_value: req_value.into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    fn make_run(test_id: &str, scenario_id: Option<&str>) -> Run {
        Run {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: "agent-1".into(),
            scenario_id: scenario_id.map(|s| s.into()),
            test_id: test_id.into(),
            params_json: None,
            replay_seed: None,
            status: "completed".into(),
            result_json: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_calc_percent_basic() {
        assert_eq!(calc_percent(50, 100), 50);
    }

    #[test]
    fn test_calc_percent_zero_target() {
        assert_eq!(calc_percent(5, 0), 100);
    }

    #[test]
    fn test_calc_percent_overflow_clamp() {
        assert_eq!(calc_percent(200, 100), 100);
    }

    #[test]
    fn test_calc_percent_zero_current() {
        assert_eq!(calc_percent(0, 100), 0);
    }

    #[test]
    fn test_calc_percent_exact() {
        assert_eq!(calc_percent(1, 1), 100);
    }

    #[test]
    fn test_parse_dependencies_scenario_count() {
        let val = r#"{"count":5,"depends_on":["ach-a","ach-b"]}"#;
        let deps = parse_dependencies("scenario_count", val);
        assert_eq!(deps, vec!["ach-a", "ach-b"]);
    }

    #[test]
    fn test_parse_dependencies_scenario_count_no_deps() {
        let val = r#"{"count":5}"#;
        let deps = parse_dependencies("scenario_count", val);
        assert!(deps.is_empty());
    }

    #[test]
    fn test_parse_dependencies_verdict_streak() {
        let val = r#"{"streak":3,"depends_on":["dep1"]}"#;
        let deps = parse_dependencies("verdict_streak", val);
        assert_eq!(deps, vec!["dep1"]);
    }

    #[test]
    fn test_parse_dependencies_specific_scenario() {
        let val = r#"{"scenario_id":"s1","count":1,"depends_on":["dep2"]}"#;
        let deps = parse_dependencies("specific_scenario", val);
        assert_eq!(deps, vec!["dep2"]);
    }

    #[test]
    fn test_parse_dependencies_unknown_type() {
        let deps = parse_dependencies("unknown_type", r#"{"depends_on":["x"]}"#);
        assert!(deps.is_empty());
    }

    #[test]
    fn test_parse_dependencies_invalid_json() {
        let deps = parse_dependencies("scenario_count", "not json");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_evaluate_scenario_count() {
        let ach = make_achievement("scenario_count", r#"{"count":5}"#);
        let runs: Vec<Run> = (0..3).map(|i| make_run(&format!("t{i}"), None)).collect();
        assert_eq!(evaluate_achievement_progress(&ach, &runs, 0), 60);
    }

    #[test]
    fn test_evaluate_verdict_streak() {
        let ach = make_achievement("verdict_streak", r#"{"streak":5}"#);
        let runs: Vec<Run> = vec![];
        assert_eq!(evaluate_achievement_progress(&ach, &runs, 3), 60);
    }

    #[test]
    fn test_evaluate_specific_scenario_by_test_id() {
        let ach = make_achievement("specific_scenario", r#"{"test_id":"BAS-001","count":2}"#);
        let runs = vec![
            make_run("BAS-001", Some("s1")),
            make_run("BAS-002", Some("s2")),
            make_run("BAS-001", Some("s1")),
        ];
        assert_eq!(evaluate_achievement_progress(&ach, &runs, 0), 100);
    }

    #[test]
    fn test_evaluate_unknown_type() {
        let ach = make_achievement("unknown_type", r#"{}"#);
        let runs: Vec<Run> = vec![];
        assert_eq!(evaluate_achievement_progress(&ach, &runs, 0), 0);
    }
}