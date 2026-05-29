use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub hostname: String,
    pub ip: String,
    pub os: String,
    pub arch: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRun {
    pub id: String,
    pub test_id: String,
    pub scenario_id: Option<String>,
    pub params_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub idx: i64,
    pub name: String,
    pub status: String,
    pub executor_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCompleteRequest {
    pub success: bool,
    pub result_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorAction {
    #[serde(rename = "type")]
    pub type_: String,
    pub choice_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub status: String,
    pub result_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePayload {
    pub run_id: String,
    pub step_id: String,
    pub kind: String,
    pub content_json: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_roundtrip() {
        let req = RegisterRequest {
            id: Some("abc".into()),
            hostname: "host-a".into(),
            ip: "10.0.0.1".into(),
            os: "windows".into(),
            arch: "x86_64".into(),
            user: "admin".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: RegisterRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.hostname, "host-a");
        assert_eq!(decoded.id.as_deref(), Some("abc"));
    }

    #[test]
    fn test_register_request_skip_none_id() {
        let req = RegisterRequest {
            id: None,
            hostname: "h".into(),
            ip: "1.2.3.4".into(),
            os: "linux".into(),
            arch: "aarch64".into(),
            user: "u".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn test_register_request_include_some_id() {
        let req = RegisterRequest {
            id: Some("x".into()),
            hostname: "h".into(),
            ip: "1.2.3.4".into(),
            os: "linux".into(),
            arch: "aarch64".into(),
            user: "u".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"id\""));
    }

    #[test]
    fn test_pending_run_roundtrip() {
        let run = PendingRun {
            id: "run-1".into(),
            test_id: "BAS-001".into(),
            scenario_id: Some("s1".into()),
            params_json: Some(r#"{"k":"v"}"#.into()),
        };
        let json = serde_json::to_string(&run).unwrap();
        let decoded: PendingRun = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, "run-1");
        assert_eq!(decoded.scenario_id.as_deref(), Some("s1"));
    }

    #[test]
    fn test_step_roundtrip() {
        let step = Step {
            id: "s1".into(),
            idx: 0,
            name: "Recon".into(),
            status: "pending".into(),
            executor_info: None,
        };
        let json = serde_json::to_string(&step).unwrap();
        let decoded: Step = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.idx, 0);
        assert_eq!(decoded.name, "Recon");
    }

    #[test]
    fn test_operator_action_type_rename() {
        let action = OperatorAction {
            type_: "approve".into(),
            choice_id: Some("c1".into()),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"type\""));
        assert!(!json.contains("\"type_\""));
        let decoded: OperatorAction = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.type_, "approve");
    }

    #[test]
    fn test_run_result_roundtrip() {
        let result = RunResult {
            status: "completed".into(),
            result_json: Some(r#"{"ok":true}"#.into()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let decoded: RunResult = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.status, "completed");
    }

    #[test]
    fn test_event_payload_skip_none_agent_id() {
        let event = EventPayload {
            run_id: Some("r1".into()),
            agent_id: None,
            level: "info".into(),
            message: "test".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.contains("\"agent_id\""));
    }

    #[test]
    fn test_event_payload_roundtrip() {
        let event = EventPayload {
            run_id: Some("r1".into()),
            agent_id: Some("a1".into()),
            level: "warn".into(),
            message: "something".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: EventPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.level, "warn");
        assert_eq!(decoded.agent_id.as_deref(), Some("a1"));
    }

    #[test]
    fn test_evidence_payload_roundtrip() {
        let evidence = EvidencePayload {
            run_id: "r1".into(),
            step_id: "s1".into(),
            kind: "file".into(),
            content_json: Some(r#"{"path":"/etc/passwd"}"#.into()),
        };
        let json = serde_json::to_string(&evidence).unwrap();
        let decoded: EvidencePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.kind, "file");
        assert!(decoded.content_json.is_some());
    }

    #[test]
    fn test_step_complete_request_roundtrip() {
        let req = StepCompleteRequest {
            success: true,
            result_json: Some(r#"{"ok":true}"#.into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: StepCompleteRequest = serde_json::from_str(&json).unwrap();
        assert!(decoded.success);
        assert_eq!(decoded.result_json.as_deref(), Some(r#"{"ok":true}"#));
    }

    #[test]
    fn test_step_executor_info_roundtrip() {
        let step = Step {
            id: "s1".into(),
            idx: 0,
            name: "Scan".into(),
            status: "READY".into(),
            executor_info: Some(r#"{"executor":"scanner","command":"127.0.0.1"}"#.into()),
        };
        let json = serde_json::to_string(&step).unwrap();
        let decoded: Step = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.status, "READY");
        assert!(decoded.executor_info.is_some());
    }
}
