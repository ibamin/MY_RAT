use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::handlers::AppState;
use crate::models::{
    AiAccount, AiAccountInfo, AiChatRequest, AiConversation, AiMessage,
    CreateAiAccountRequest, CreateConversationRequest,
};

const SCENARIO_SYSTEM_PROMPT: &str = r#"You are a specialized assistant for creating BAS (Breach & Attack Simulation) test scenarios for the Shadow Protocol red team training platform.

When asked to generate a scenario, output a valid JSON block in this exact structure:

```json
{
  "scenario_id": "unique-kebab-case-id",
  "test_id": "BAS-CATEGORY-NNN",
  "title": "Scenario Title",
  "difficulty": 1,
  "version": "1.0.0",
  "estimated_time_sec": 60,
  "steps": [
    {
      "step_id": "s1",
      "name": "Step Name",
      "requires_choice_id": null,
      "executor": "powershell",
      "command": "whoami",
      "args": [],
      "actions": [
        { "action_id": "a1", "title": "Emit telemetry", "kind": "emit_events" }
      ],
      "choices": [
        { "choice_id": "c1", "title": "Option A" },
        { "choice_id": "c2", "title": "Option B" }
      ],
      "assertions": [
        {
          "assertion_id": "as1",
          "description": "Evidence must be present",
          "required": true,
          "type": "evidence_kind",
          "kind": "telemetry",
          "contains": null
        }
      ]
    }
  ]
}
```

Field rules:
- scenario_id: lowercase kebab-case, globally unique (e.g., "recon-ad-enum-001")
- test_id: BAS-CATEGORY-NNN format (e.g., BAS-RECON-001, BAS-EXEC-002, BAS-PRIV-003)
- difficulty: integer 1–5 (1=trivial, 5=expert)
- executor values: "powershell", "com", "syscall", "shell", "scanner", null
- assertion type "evidence_kind": checks that evidence of the given kind exists
- assertion type "event_contains": checks that an event message contains the given string
- First step always has requires_choice_id: null
- Branching steps set requires_choice_id to a choice_id from a prior step
- actions with kind "emit_events" auto-generate telemetry evidence

You can discuss attack techniques, explain red team concepts, and iteratively refine scenarios.
When producing a final scenario, always wrap the JSON in ```json ``` fences so the platform can auto-detect and preview it."#;

fn default_model(provider: &str) -> &'static str {
    match provider {
        "claude" => "claude-sonnet-4-6",
        "openai" => "gpt-4o",
        "gemini" => "gemini-2.0-flash",
        _ => "gpt-4o",
    }
}

async fn call_claude(api_key: &str, model: &str, messages: &[AiMessage]) -> Result<String, String> {
    let client = Client::new();
    let msgs: Vec<Value> = messages
        .iter()
        .map(|m| json!({ "role": m.role, "content": m.content }))
        .collect();

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&json!({
            "model": model,
            "max_tokens": 4096,
            "system": SCENARIO_SYSTEM_PROMPT,
            "messages": msgs
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(err) = body.get("error") {
        return Err(err["message"].as_str().unwrap_or("Unknown error").to_string());
    }

    body["content"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected Claude response: {}", body))
}

async fn call_openai(api_key: &str, model: &str, messages: &[AiMessage]) -> Result<String, String> {
    let client = Client::new();
    let mut msgs = vec![json!({ "role": "system", "content": SCENARIO_SYSTEM_PROMPT })];
    for m in messages {
        msgs.push(json!({ "role": m.role, "content": m.content }));
    }

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("content-type", "application/json")
        .json(&json!({ "model": model, "messages": msgs }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(err) = body.get("error") {
        return Err(err["message"].as_str().unwrap_or("Unknown error").to_string());
    }

    body["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected OpenAI response: {}", body))
}

async fn call_gemini(api_key: &str, model: &str, messages: &[AiMessage]) -> Result<String, String> {
    let client = Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    let contents: Vec<Value> = messages
        .iter()
        .map(|m| {
            let role = if m.role == "assistant" { "model" } else { "user" };
            json!({ "role": role, "parts": [{ "text": m.content }] })
        })
        .collect();

    let resp = client
        .post(&url)
        .header("content-type", "application/json")
        .header("x-goog-api-key", api_key)
        .json(&json!({
            "system_instruction": { "parts": [{ "text": SCENARIO_SYSTEM_PROMPT }] },
            "contents": contents
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(err) = body.get("error") {
        return Err(err["message"].as_str().unwrap_or("Unknown error").to_string());
    }

    body["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected Gemini response: {}", body))
}

async fn call_provider(account: &AiAccount, messages: &[AiMessage]) -> Result<String, String> {
    let api_key = crate::crypto::decrypt_api_key(&account.api_key);
    match account.provider.as_str() {
        "claude" => call_claude(&api_key, &account.model, messages).await,
        "openai" => call_openai(&api_key, &account.model, messages).await,
        "gemini" => call_gemini(&api_key, &account.model, messages).await,
        p => Err(format!("Unknown provider: {}", p)),
    }
}

fn extract_scenario_draft(text: &str) -> Option<Value> {
    let marker = "```json";
    let start_pos = text.find(marker)?;
    let after_marker = start_pos + marker.len();
    let content_start = if text.as_bytes().get(after_marker) == Some(&b'\n') {
        after_marker + 1
    } else {
        after_marker
    };
    let end_pos = text[content_start..].find("```")? + content_start;
    let json_str = text[content_start..end_pos].trim();
    serde_json::from_str(json_str).ok()
}

// --- Handlers ---

pub async fn list_ai_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AiAccountInfo>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AiAccount>(
        "SELECT * FROM ai_accounts ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows.into_iter().map(AiAccountInfo::from).collect()))
}

pub async fn create_ai_account(
    State(state): State<AppState>,
    Json(req): Json<CreateAiAccountRequest>,
) -> Result<Json<AiAccountInfo>, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    let model = req
        .model
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| default_model(&req.provider).to_string());
    let now = Utc::now().to_rfc3339();

    let stored_key = crate::crypto::encrypt_api_key(&req.api_key)
        .unwrap_or_else(|_| req.api_key.clone());

    sqlx::query(
        "INSERT INTO ai_accounts (id, name, provider, auth_type, api_key, model, is_active, created_at)
         VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.provider)
    .bind(&req.auth_type)
    .bind(&stored_key)
    .bind(&model)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AiAccountInfo {
        id,
        name: req.name,
        provider: req.provider,
        auth_type: req.auth_type,
        model,
        is_active: true,
        created_at: now,
    }))
}

pub async fn remove_ai_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    sqlx::query("DELETE FROM ai_accounts WHERE id = ?")
        .bind(&account_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "removed": true })))
}

pub async fn list_ai_conversations(
    State(state): State<AppState>,
) -> Result<Json<Vec<AiConversation>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AiConversation>(
        "SELECT * FROM ai_conversations ORDER BY updated_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

pub async fn create_ai_conversation(
    State(state): State<AppState>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<AiConversation>, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    let title = req.title.unwrap_or_else(|| "New Conversation".to_string());
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO ai_conversations (id, account_id, title, messages_json, scenario_draft, created_at, updated_at)
         VALUES (?, ?, ?, '[]', NULL, ?, ?)",
    )
    .bind(&id)
    .bind(&req.account_id)
    .bind(&title)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AiConversation {
        id,
        account_id: req.account_id,
        title,
        messages_json: "[]".to_string(),
        scenario_draft: None,
        created_at: now.clone(),
        updated_at: now,
    }))
}

pub async fn get_ai_conversation(
    State(state): State<AppState>,
    Path(conv_id): Path<String>,
) -> Result<Json<AiConversation>, (StatusCode, String)> {
    let conv = sqlx::query_as::<_, AiConversation>(
        "SELECT * FROM ai_conversations WHERE id = ?",
    )
    .bind(&conv_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;

    Ok(Json(conv))
}

pub async fn remove_ai_conversation(
    State(state): State<AppState>,
    Path(conv_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    sqlx::query("DELETE FROM ai_conversations WHERE id = ?")
        .bind(&conv_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "removed": true })))
}

pub async fn ai_chat(
    State(state): State<AppState>,
    Path(conv_id): Path<String>,
    Json(req): Json<AiChatRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Load conversation
    let conv = sqlx::query_as::<_, AiConversation>(
        "SELECT * FROM ai_conversations WHERE id = ?",
    )
    .bind(&conv_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;

    // Load account
    let account = sqlx::query_as::<_, AiAccount>(
        "SELECT * FROM ai_accounts WHERE id = ?",
    )
    .bind(&conv.account_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "AI account not found".to_string()))?;

    // Deserialize existing messages
    let mut messages: Vec<AiMessage> =
        serde_json::from_str(&conv.messages_json).unwrap_or_default();

    // Add user message
    messages.push(AiMessage {
        role: "user".to_string(),
        content: req.message.clone(),
    });

    // Call AI provider
    let reply = call_provider(&account, &messages)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("AI error: {}", e)))?;

    // Add assistant message
    messages.push(AiMessage {
        role: "assistant".to_string(),
        content: reply.clone(),
    });

    // Extract scenario draft if JSON block present
    let draft = extract_scenario_draft(&reply);
    let draft_json = draft
        .as_ref()
        .and_then(|v| serde_json::to_string_pretty(v).ok());

    // Auto-title from first user message if still default
    let new_title = if conv.title == "New Conversation" {
        let snippet: String = req.message.chars().take(50).collect();
        snippet
    } else {
        conv.title.clone()
    };

    let messages_json = serde_json::to_string(&messages)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE ai_conversations
         SET messages_json = ?, scenario_draft = COALESCE(?, scenario_draft), title = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&messages_json)
    .bind(&draft_json)
    .bind(&new_title)
    .bind(&now)
    .bind(&conv_id)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "reply": reply,
        "scenario_draft": draft,
        "messages": messages,
    })))
}

pub async fn save_scenario(
    State(state): State<AppState>,
    Path(conv_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conv = sqlx::query_as::<_, AiConversation>(
        "SELECT * FROM ai_conversations WHERE id = ?",
    )
    .bind(&conv_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;

    let draft = conv
        .scenario_draft
        .ok_or((StatusCode::BAD_REQUEST, "No scenario draft in this conversation".to_string()))?;

    let scenario: Value = serde_json::from_str(&draft)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid scenario JSON: {}", e)))?;

    let scenario_id = scenario["scenario_id"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "Scenario JSON missing scenario_id".to_string()))?;

    if !scenario_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') || scenario_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Invalid scenario_id: only alphanumeric, '-', and '_' characters allowed".to_string()));
    }

    let scenarios_dir =
        std::env::var("SCENARIOS_PATH").unwrap_or_else(|_| "data/scenarios".to_string());

    std::fs::create_dir_all(&scenarios_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Cannot create scenarios dir: {}", e)))?;

    let path = std::path::Path::new(&scenarios_dir).join(format!("{}.json", scenario_id));
    let canonical_dir = std::fs::canonicalize(&scenarios_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Cannot resolve scenarios dir: {}", e)))?;
    let canonical_path = {
        // File doesn't exist yet; canonicalize the parent and reconstruct
        let parent = path.parent().and_then(|p| std::fs::canonicalize(p).ok())
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Cannot resolve path parent".to_string()))?;
        if !parent.starts_with(&canonical_dir) {
            return Err((StatusCode::BAD_REQUEST, "Path traversal detected".to_string()));
        }
        parent.join(format!("{}.json", scenario_id))
    };

    let pretty = serde_json::to_string_pretty(&scenario)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    std::fs::write(&canonical_path, pretty)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Write failed: {}", e)))?;

    Ok(Json(json!({
        "saved": true,
        "scenario_id": scenario_id,
        "path": canonical_path.to_string_lossy(),
        "note": "Restart server to load the new scenario into the catalog."
    })))
}
