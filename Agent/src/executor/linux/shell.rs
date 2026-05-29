use async_trait::async_trait;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::Instant,
};

use crate::executor::{ExecutionResult, Executor};

pub struct ShellExecutor;

#[async_trait]
impl Executor for ShellExecutor {
    fn kind(&self) -> &'static str {
        "shell"
    }

    fn is_available(&self) -> bool {
        Path::new("/bin/sh").exists()
    }

    async fn execute(&self, command: &str, args: &[&str]) -> ExecutionResult {
        let started = Instant::now();
        let script = command.to_string();
        let arg_list: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();

        let result = tokio::task::spawn_blocking(move || run_shell(&script, &arg_list)).await;
        let duration_ms = started.elapsed().as_millis() as u64;

        match result {
            Ok(Ok((stdout, stderr, exit_code))) => ExecutionResult {
                success: exit_code == Some(0),
                stdout,
                stderr,
                exit_code,
                evidence: None,
                duration_ms,
            },
            Ok(Err(err)) => ExecutionResult {
                success: false,
                stdout: Vec::new(),
                stderr: err.into_bytes(),
                exit_code: None,
                evidence: None,
                duration_ms,
            },
            Err(join_err) => ExecutionResult {
                success: false,
                stdout: Vec::new(),
                stderr: join_err.to_string().into_bytes(),
                exit_code: None,
                evidence: None,
                duration_ms,
            },
        }
    }
}

fn run_shell(script: &str, args: &[String]) -> Result<(Vec<u8>, Vec<u8>, Option<i32>), String> {
    let default_shell = "/bin/sh".to_string();
    let shell = args.first().cloned().unwrap_or(default_shell);

    if shell != "/bin/sh" && shell != "/bin/bash" && shell != "/bin/zsh" {
        return Err(format!("unsupported shell '{}'", shell));
    }

    let mut child = Command::new(&shell)
        .arg("-s")
        .args(args.iter().skip(1))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to spawn shell '{}': {}", shell, err))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(script.as_bytes())
            .map_err(|err| format!("failed to write script to shell stdin: {}", err))?;
    } else {
        return Err("shell stdin pipe unavailable".to_string());
    }

    let output = child
        .wait_with_output()
        .map_err(|err| format!("failed waiting for shell output: {}", err))?;

    Ok((output.stdout, output.stderr, output.status.code()))
}
