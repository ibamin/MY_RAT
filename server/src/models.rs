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
    pub test_id: String,
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
