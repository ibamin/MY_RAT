use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Agent {
    pub id: String,
    pub hostname: String,
    pub ip: String,
    pub os: String,
    pub arch: String,
    pub user: String,
    pub last_seen: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub hostname: String,
    pub ip: String,
    pub os: String,
    pub arch: String,
    pub user: String,
}

#[derive(Debug, Deserialize)]
pub struct FingerprintMatchRequest {
    pub banner: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct FingerprintCandidate {
    pub service: String,
    pub product: Option<String>,
    pub version: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Serialize)]
pub struct FingerprintMatchResponse {
    pub candidates: Vec<FingerprintCandidate>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Run {
    pub id: String,
    pub agent_id: String,
    pub scenario_id: Option<String>,
    pub test_id: String,
    pub params_json: Option<String>,
    pub status: String,
    pub result_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub agent_id: String,
    pub scenario_id: Option<String>,
    pub test_id: Option<String>,
    pub params_json: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRunResultRequest {
    pub status: String,
    pub result_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Event {
    pub id: String,
    pub run_id: Option<String>,
    pub agent_id: Option<String>,
    pub level: String,
    pub message: String,
    pub ts: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub run_id: Option<String>,
    pub agent_id: Option<String>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Step {
    pub id: String,
    pub run_id: String,
    pub idx: i64,
    pub name: String,
    pub status: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Evidence {
    pub id: String,
    pub run_id: String,
    pub step_id: String,
    pub kind: String,
    pub locator: String,
    pub sha256: String,
    pub content_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEvidenceRequest {
    pub run_id: String,
    pub step_id: String,
    pub kind: String,
    pub content_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Assertion {
    pub id: String,
    pub run_id: String,
    pub step_id: String,
    pub description: String,
    pub required: bool,
    pub rule_type: Option<String>,
    pub kind: Option<String>,
    pub contains: Option<String>,
    pub status: String,
    pub evidence_refs_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct VerdictRow {
    pub id: String,
    pub run_id: String,
    pub step_id: String,
    pub verdict: String,
    pub reason_code: Option<String>,
    pub summary: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct OperatorActionRequest {
    #[serde(rename = "type")]
    pub type_: String,
    pub action_id: Option<String>,
    pub choice_id: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct OperatorAction {
    pub id: String,
    pub run_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub action_id: Option<String>,
    pub choice_id: Option<String>,
    pub note: Option<String>,
    pub ts: String,
}
