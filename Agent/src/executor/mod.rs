use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Executor: Send + Sync {
    fn kind(&self) -> &'static str;
    fn is_available(&self) -> bool;
    async fn execute(&self, command: &str, args: &[&str]) -> ExecutionResult;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: Option<i32>,
    pub evidence: Option<serde_json::Value>,
    pub duration_ms: u64,
}

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_serde() {
        let result = ExecutionResult {
            success: true,
            stdout: b"hello".to_vec(),
            stderr: vec![],
            exit_code: Some(0),
            evidence: Some(serde_json::json!({"key": "value"})),
            duration_ms: 150,
        };
        let json = serde_json::to_string(&result).unwrap();
        let decoded: ExecutionResult = serde_json::from_str(&json).unwrap();
        assert!(decoded.success);
        assert_eq!(decoded.exit_code, Some(0));
        assert_eq!(decoded.duration_ms, 150);
        assert!(decoded.evidence.is_some());
    }

    #[test]
    fn test_execution_result_serde_none_evidence() {
        let result = ExecutionResult {
            success: false,
            stdout: vec![],
            stderr: b"error".to_vec(),
            exit_code: None,
            evidence: None,
            duration_ms: 0,
        };
        let json = serde_json::to_string(&result).unwrap();
        let decoded: ExecutionResult = serde_json::from_str(&json).unwrap();
        assert!(!decoded.success);
        assert!(decoded.evidence.is_none());
        assert!(decoded.exit_code.is_none());
    }
}
