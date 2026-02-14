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
    test_id: String,
    params_json: Option<String>,
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
            let _ = client
                .post(format!("{}/api/events", server_url))
                .json(&CreateEventRequest {
                    run_id: Some(run_id.clone()),
                    agent_id: Some(agent.id.clone()),
                    level: "info".to_string(),
                    message: format!("run_start test_id={}", run.test_id),
                })
                .send()
                .await;

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
