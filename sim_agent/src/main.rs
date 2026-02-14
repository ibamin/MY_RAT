use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct RegisterRequest {
    hostname: String,
    ip: String,
    os: String,
    arch: String,
    user: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Agent {
    id: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Run {
    id: String,
    scenario_id: Option<String>,
    test_id: String,
    params_json: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct OperatorAction {
    #[serde(rename = "type")]
    type_: String,
    choice_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Step {
    id: String,
    idx: i64,
    name: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct UpdateRunResultRequest {
    status: String,
    result_json: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateEventRequest {
    run_id: Option<String>,
    agent_id: Option<String>,
    level: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct CreateEvidenceRequest {
    run_id: String,
    step_id: String,
    kind: String,
    content_json: Option<String>,
}

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "sim-agent".to_string())
}

fn user() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_url = env_or("SERVER_URL", "http://127.0.0.1:3000");
    let client = reqwest::Client::new();

    let reg = RegisterRequest {
        hostname: env_or("AGENT_HOSTNAME", &hostname()),
        ip: env_or("AGENT_IP", "127.0.0.1"),
        os: env_or("AGENT_OS", std::env::consts::OS),
        arch: env_or("AGENT_ARCH", std::env::consts::ARCH),
        user: env_or("AGENT_USER", &user()),
    };

    let agent: Agent = client
        .post(format!("{}/api/agents/register", server_url))
        .json(&reg)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    loop {
        let _ = client
            .post(format!("{}/api/agents/{}/heartbeat", server_url, agent.id))
            .send()
            .await;

        let pending: Vec<Run> = client
            .get(format!("{}/api/runs/pending/{}", server_url, agent.id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        for run in pending {
            let run_id = run.id.clone();
            let scenario_id = run.scenario_id.clone().unwrap_or_else(|| "unknown".to_string());
            let _ = client
                .post(format!("{}/api/events", server_url))
                .json(&CreateEventRequest {
                    run_id: Some(run_id.clone()),
                    agent_id: Some(agent.id.clone()),
                    level: "info".to_string(),
                    message: format!("run_start scenario_id={} test_id={}", scenario_id, run.test_id),
                })
                .send()
                .await;

            if run.scenario_id.is_some() {
                for _ in 0..20 {
                    let ops: Vec<OperatorAction> = client
                        .get(format!("{}/api/runs/{}/operator-actions", server_url, run_id))
                        .send()
                        .await?
                        .error_for_status()?
                        .json()
                        .await?;
                    if ops
                        .iter()
                        .any(|o| o.type_ == "select_choice" && o.choice_id.as_deref().unwrap_or("") != "")
                    {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }

            let mut done_steps: std::collections::HashSet<String> = std::collections::HashSet::new();
            for _ in 0..10 {
                let steps: Vec<Step> = client
                    .get(format!("{}/api/runs/{}/steps", server_url, run_id))
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;

                let mut progressed = false;
                for step in steps {
                    if done_steps.contains(&step.id) {
                        continue;
                    }
                    progressed = true;
                    done_steps.insert(step.id.clone());

                    let step_id = step.id.clone();
                    let step_idx = step.idx;
                    let step_name = step.name.clone();

                let _ = client
                    .post(format!("{}/api/events", server_url))
                    .json(&CreateEventRequest {
                        run_id: Some(run_id.clone()),
                        agent_id: Some(agent.id.clone()),
                        level: "info".to_string(),
                        message: format!("step_start idx={} name={}", step_idx, step_name),
                    })
                    .send()
                    .await;

                let content_json = serde_json::json!({
                    "scenario_id": scenario_id.clone(),
                    "test_id": run.test_id.clone(),
                    "step": { "id": step_id.clone(), "idx": step_idx, "name": step_name },
                    "note": "simulated evidence"
                })
                .to_string();

                let _ = client
                    .post(format!("{}/api/evidence", server_url))
                    .json(&CreateEvidenceRequest {
                        run_id: run_id.clone(),
                        step_id,
                        kind: "telemetry".to_string(),
                        content_json: Some(content_json),
                    })
                    .send()
                    .await;

                    let _ = client
                        .post(format!("{}/api/events", server_url))
                        .json(&CreateEventRequest {
                            run_id: Some(run_id.clone()),
                            agent_id: Some(agent.id.clone()),
                            level: "info".to_string(),
                            message: format!("step_done idx={}", step_idx),
                        })
                        .send()
                        .await;
                }

                if !progressed {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }

            tokio::time::sleep(std::time::Duration::from_millis(800)).await;

            let result_json = serde_json::json!({
                "ok": true,
                "test_id": run.test_id,
                "note": "simulated execution",
                "params_json": run.params_json,
            })
            .to_string();

            let _ = client
                .post(format!("{}/api/runs/{}/result", server_url, run_id))
                .json(&UpdateRunResultRequest {
                    status: "completed".to_string(),
                    result_json: Some(result_json),
                })
                .send()
                .await;

            let _ = client
                .post(format!("{}/api/events", server_url))
                .json(&CreateEventRequest {
                    run_id: Some(run_id),
                    agent_id: Some(agent.id.clone()),
                    level: "info".to_string(),
                    message: "run_done".to_string(),
                })
                .send()
                .await;
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
