use std::collections::HashMap;

use agent::{
    config,
    executor::{ExecutionResult, Executor},
    scanner::{self, port::TOP_PORTS},
    transport::{http::HttpTransport, protocol::{EvidencePayload, RunResult}},
};

#[cfg(windows)]
use agent::executor::windows::{
    com::ComExecutor, fileless::FilelessExecutor, powershell::PowerShellExecutor,
    process::ProcessExecutor, registry::RegistryExecutor,
};

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "agent-host".to_string())
}

fn current_user() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn local_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|sock| {
            sock.connect("8.8.8.8:53")?;
            sock.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

fn jitter_duration(base_secs: u64) -> std::time::Duration {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    let seed = nanos ^ (std::process::id() as u64).wrapping_mul(2654435761);
    let factor = (seed % 601) as f64 / 1000.0 - 0.3; // range: -0.3 to +0.3
    let jittered = base_secs as f64 * (1.0 + factor);
    std::time::Duration::from_secs_f64(jittered.max(1.0))
}

fn build_executor_registry() -> HashMap<&'static str, Box<dyn Executor>> {
    let mut registry: HashMap<&'static str, Box<dyn Executor>> = HashMap::new();

    #[cfg(windows)]
    {
        let executors: Vec<Box<dyn Executor>> = vec![
            Box::new(ComExecutor),
            Box::new(ProcessExecutor),
            Box::new(PowerShellExecutor),
            Box::new(RegistryExecutor),
            Box::new(FilelessExecutor),
        ];
        for executor in executors {
            if executor.is_available() {
                registry.insert(executor.kind(), executor);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use agent::executor::linux::{MemfdExecutor, RawSyscallExecutor, ShellExecutor};

        let executors: Vec<Box<dyn Executor>> = vec![
            Box::new(MemfdExecutor),
            Box::new(RawSyscallExecutor),
            Box::new(ShellExecutor),
        ];
        for executor in executors {
            if executor.is_available() {
                registry.insert(executor.kind(), executor);
            }
        }
    }

    registry
}

async fn dispatch_scanner(params: &serde_json::Value) -> ExecutionResult {
    let started = std::time::Instant::now();

    let target = params
        .get("target")
        .or_else(|| params.get("command"))
        .and_then(|v| v.as_str())
        .unwrap_or("127.0.0.1");

    let ports: Vec<u16> = params
        .get("ports")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|p| p as u16)).collect())
        .unwrap_or_else(|| TOP_PORTS.to_vec());

    let banners = scanner::run_scan(target, &ports).await;
    let duration_ms = started.elapsed().as_millis() as u64;
    let evidence = serde_json::to_value(&banners).ok();
    let summary = format!("scanned {} ports on {}, found {} banners", ports.len(), target, banners.len());

    ExecutionResult {
        success: true,
        stdout: summary.into_bytes(),
        stderr: Vec::new(),
        exit_code: Some(0),
        evidence,
        duration_ms,
    }
}

async fn dispatch_step(
    executors: &HashMap<&str, Box<dyn Executor>>,
    params: &serde_json::Value,
) -> ExecutionResult {
    let executor_kind = params
        .get("executor")
        .and_then(|v| v.as_str())
        .unwrap_or("powershell");

    if executor_kind == "scanner" {
        return dispatch_scanner(params).await;
    }

    let command = params
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let args: Vec<&str> = params
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if let Some(executor) = executors.get(executor_kind) {
        if executor.is_available() {
            executor.execute(command, &args).await
        } else {
            ExecutionResult {
                success: false,
                stdout: Vec::new(),
                stderr: format!("Executor '{}' not available on this platform", executor_kind)
                    .into_bytes(),
                exit_code: None,
                evidence: None,
                duration_ms: 0,
            }
        }
    } else {
        let available: Vec<&str> = executors.keys().copied().collect();
        ExecutionResult {
            success: false,
            stdout: Vec::new(),
            stderr: format!(
                "Unknown executor '{}'. Available: {:?}",
                executor_kind, available
            )
            .into_bytes(),
            exit_code: None,
            evidence: None,
            duration_ms: 0,
        }
    }
}

async fn execute_run(
    transport: &HttpTransport,
    executors: &HashMap<&str, Box<dyn Executor>>,
    run_id: &str,
    test_id: &str,
) {
    let _ = transport
        .post_event(run_id, "info", &format!("run_start test_id={}", test_id))
        .await;

    let step_poll_interval = std::time::Duration::from_millis(500);
    let mut all_success = true;
    let mut step_results: Vec<serde_json::Value> = Vec::new();
    let mut consecutive_empty = 0u32;

    loop {
        let steps = match transport.get_steps(run_id).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[agent] failed to get steps for run {}: {}", run_id, e);
                break;
            }
        };

        let ready_steps: Vec<_> = steps.iter().filter(|s| s.status == "READY").collect();
        let locked_count = steps.iter().filter(|s| s.status == "LOCKED").count();

        if ready_steps.is_empty() {
            if locked_count == 0 {
                break;
            }
            consecutive_empty += 1;
            if consecutive_empty > 60 {
                eprintln!("[agent] run {} timed out waiting for READY steps", run_id);
                all_success = false;
                break;
            }
            tokio::time::sleep(step_poll_interval).await;
            continue;
        }

        consecutive_empty = 0;

        for step in &ready_steps {
            let _ = transport
                .post_event(
                    run_id,
                    "info",
                    &format!("step_start idx={} name={}", step.idx, step.name),
                )
                .await;

            let params: serde_json::Value = step
                .executor_info
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(serde_json::json!({}));

            let result = dispatch_step(executors, &params).await;

            if let Some(ref evidence_val) = result.evidence {
                if let Some(banners) = evidence_val.as_array() {
                    for banner in banners {
                        let _ = transport
                            .post_evidence(EvidencePayload {
                                run_id: run_id.to_string(),
                                step_id: step.id.clone(),
                                kind: "banner".to_string(),
                                content_json: serde_json::to_string(banner).ok(),
                            })
                            .await;
                    }
                } else {
                    let _ = transport
                        .post_evidence(EvidencePayload {
                            run_id: run_id.to_string(),
                            step_id: step.id.clone(),
                            kind: "evidence".to_string(),
                            content_json: serde_json::to_string(evidence_val).ok(),
                        })
                        .await;
                }
            }

            let step_success = result.success;
            if !step_success {
                all_success = false;
            }

            let result_json_str = serde_json::json!({
                "idx": step.idx,
                "name": step.name,
                "success": result.success,
                "exit_code": result.exit_code,
                "stdout_len": result.stdout.len(),
                "stderr_len": result.stderr.len(),
                "duration_ms": result.duration_ms,
            })
            .to_string();

            let _ = transport
                .complete_step(run_id, &step.id, step_success, Some(result_json_str.clone()))
                .await;

            step_results.push(serde_json::from_str(&result_json_str).unwrap_or_default());

            let _ = transport
                .post_event(
                    run_id,
                    if step_success { "info" } else { "warn" },
                    &format!(
                        "step_done idx={} success={} duration_ms={}",
                        step.idx, step_success, result.duration_ms
                    ),
                )
                .await;
        }
    }

    let result_json = serde_json::json!({
        "ok": all_success,
        "test_id": test_id,
        "steps": step_results,
    })
    .to_string();

    let _ = transport
        .post_result(
            run_id,
            RunResult {
                status: if all_success {
                    "completed".to_string()
                } else {
                    "failed".to_string()
                },
                result_json: Some(result_json),
            },
        )
        .await;

    let _ = transport.post_event(run_id, "info", "run_done").await;
}

#[tokio::main]
async fn main() {
    let guid = config::guid();
    let server_url = config::server_url();
    let sleep_sec = config::sleep_sec();

    let transport = HttpTransport::new(server_url, guid).with_auth(guid);

    let agent_id = {
        let mut attempts = 0u32;
        loop {
            match transport
                .register(
                    &hostname(),
                    &local_ip(),
                    std::env::consts::OS,
                    std::env::consts::ARCH,
                    &current_user(),
                )
                .await
            {
                Ok(id) => break id,
                Err(e) => {
                    attempts += 1;
                    let backoff = std::cmp::min(2u64.pow(attempts), 60);
                    eprintln!(
                        "[agent] registration failed (attempt {}): {e} — retrying in {backoff}s",
                        attempts
                    );
                    tokio::time::sleep(jitter_duration(backoff)).await;
                }
            }
        }
    };

    let transport = HttpTransport::new(server_url, &agent_id).with_auth(guid);

    let executors = build_executor_registry();
    let available: Vec<&str> = executors.keys().copied().collect();
    eprintln!(
        "[agent] registered id={} executors={:?}",
        agent_id, available
    );

    let hb_transport = transport.clone();
    tokio::spawn(async move {
        loop {
            let _ = hb_transport.heartbeat().await;
            tokio::time::sleep(jitter_duration(sleep_sec)).await;
        }
    });

    loop {
        match transport.poll_pending().await {
            Ok(runs) => {
                for run in runs {
                    execute_run(&transport, &executors, &run.id, &run.test_id).await;
                }
            }
            Err(e) => {
                eprintln!("[agent] poll error: {e}");
            }
        }

        tokio::time::sleep(jitter_duration(sleep_sec)).await;
    }
}
