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
    pub approval_status: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub id: Option<String>,
    pub hostname: String,
    pub ip: String,
    pub os: String,
    pub arch: String,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AgentGroup {
    pub agent_id: String,
    pub group_id: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SetAgentGroupRequest {
    pub agent_id: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AgentTag {
    pub agent_id: String,
    pub tag: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SetAgentTagRequest {
    pub tag: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRunsRequest {
    pub scenario_id: Option<String>,
    pub test_id: Option<String>,
    pub params_json: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateGroupRunsResponse {
    pub runs: Vec<Run>,
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
    pub replay_seed: Option<String>,
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
    pub scenario_step_id: Option<String>,
    pub idx: i64,
    pub name: String,
    pub status: String,
    pub executor_info: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StepCompleteRequest {
    pub success: bool,
    pub result_json: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplayData {
    pub run: Run,
    pub steps: Vec<Step>,
    pub events: Vec<Event>,
    pub evidence: Vec<Evidence>,
    pub verdicts: Vec<VerdictRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplayDataResponse {
    pub replay_data: ReplayData,
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

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AgentBuild {
    pub id: String,
    pub guid: String,
    pub target_platform: String,
    pub server_url: String,
    pub sleep_sec: i64,
    pub build_status: String,
    pub binary_path: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBuildRequest {
    pub target_platform: Option<String>,
    pub server_url: Option<String>,
    pub sleep_sec: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BuildResponse {
    pub build: AgentBuild,
    pub download_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon: String,
    pub requirement_type: String,
    pub requirement_value: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct UserAchievement {
    pub id: String,
    pub achievement_id: String,
    pub unlocked_at: Option<String>,
    pub progress: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AchievementStatus {
    pub achievement: Achievement,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
    pub progress: i64,
}

#[derive(Debug, Serialize)]
pub struct AchievementCheckResponse {
    pub checked_runs: i64,
    pub unlocked_achievement_ids: Vec<String>,
}

// --- AI Script Generator ---

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AiAccount {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub auth_type: String,
    pub api_key: String,
    pub model: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AiAccountInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub auth_type: String,
    pub model: String,
    pub is_active: bool,
    pub created_at: String,
}

impl From<AiAccount> for AiAccountInfo {
    fn from(a: AiAccount) -> Self {
        AiAccountInfo {
            id: a.id,
            name: a.name,
            provider: a.provider,
            auth_type: a.auth_type,
            model: a.model,
            is_active: a.is_active,
            created_at: a.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAiAccountRequest {
    pub name: String,
    pub provider: String,
    pub auth_type: String,
    pub api_key: String,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AiConversation {
    pub id: String,
    pub account_id: String,
    pub title: String,
    pub messages_json: String,
    pub scenario_draft: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub account_id: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AiChatRequest {
    pub message: String,
}
