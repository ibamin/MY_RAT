use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioActionDef {
    pub action_id: String,
    pub title: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioChoiceDef {
    pub choice_id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioAssertionDef {
    pub assertion_id: String,
    pub description: String,
    pub required: bool,
    #[serde(rename = "type")]
    pub type_: String,
    pub kind: Option<String>,
    pub contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioStepDef {
    pub step_id: String,
    pub name: String,
    #[serde(default)]
    pub requires_choice_id: Option<String>,
    #[serde(default)]
    pub executor: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub actions: Vec<ScenarioActionDef>,
    #[serde(default)]
    pub choices: Vec<ScenarioChoiceDef>,
    #[serde(default)]
    pub assertions: Vec<ScenarioAssertionDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDef {
    pub scenario_id: String,
    pub test_id: String,
    pub title: String,
    pub difficulty: u8,
    pub version: String,
    pub estimated_time_sec: u32,
    #[serde(default)]
    pub steps: Vec<ScenarioStepDef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioMeta {
    pub scenario_id: String,
    pub test_id: String,
    pub title: String,
    pub difficulty: u8,
    pub version: String,
    pub estimated_time_sec: u32,
}

#[derive(Clone, Default)]
pub struct ScenarioCatalog {
    by_id: HashMap<String, ScenarioDef>,
    id_by_test_id: HashMap<String, String>,
}

impl ScenarioCatalog {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn metas(&self) -> Vec<ScenarioMeta> {
        let mut out: Vec<_> = self
            .by_id
            .values()
            .map(|s| ScenarioMeta {
                scenario_id: s.scenario_id.clone(),
                test_id: s.test_id.clone(),
                title: s.title.clone(),
                difficulty: s.difficulty,
                version: s.version.clone(),
                estimated_time_sec: s.estimated_time_sec,
            })
            .collect();
        out.sort_by(|a, b| a.test_id.cmp(&b.test_id));
        out
    }

    pub fn get_by_id(&self, scenario_id: &str) -> Option<ScenarioDef> {
        self.by_id.get(scenario_id).cloned()
    }

    pub fn get_by_test_id(&self, test_id: &str) -> Option<ScenarioDef> {
        let id = self.id_by_test_id.get(test_id)?;
        self.get_by_id(id)
    }

    pub fn load_from_dir(dir: &str) -> Result<Self, String> {
        let mut catalog = Self::empty();
        let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let is_json = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("json"))
                .unwrap_or(false);
            if !is_json {
                continue;
            }

            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            let scenario: ScenarioDef =
                serde_json::from_slice(&bytes).map_err(|e| format!("{}: {}", path.display(), e))?;

            let validation = validate_scenario(&scenario);
            for w in &validation.warnings {
                eprintln!(
                    "[scenario-warn] {}: {} — {}",
                    path.display(),
                    w.path,
                    w.message
                );
            }
            for e in &validation.errors {
                eprintln!(
                    "[scenario-error] {}: {} — {}",
                    path.display(),
                    e.path,
                    e.message
                );
            }

            if scenario.scenario_id.trim().is_empty() {
                return Err(format!("{}: scenario_id is required", path.display()));
            }
            if scenario.test_id.trim().is_empty() {
                return Err(format!("{}: test_id is required", path.display()));
            }

            catalog
                .id_by_test_id
                .insert(scenario.test_id.clone(), scenario.scenario_id.clone());
            catalog.by_id.insert(scenario.scenario_id.clone(), scenario);
        }

        Ok(catalog)
    }
}

// ---------------------------------------------------------------------------
// Scenario Validator
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
    pub severity: String, // "error" | "warning"
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

const VALID_ASSERTION_TYPES: &[&str] = &["evidence_kind", "event_contains"];
const VALID_ACTION_KINDS: &[&str] = &["emit_events", "execute", "scan", "recon", "exfiltrate"];

pub fn validate_scenario(scenario: &ScenarioDef) -> ValidationResult {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationError> = Vec::new();

    if scenario.scenario_id.trim().is_empty() {
        errors.push(ValidationError {
            path: "scenario_id".into(),
            message: "scenario_id is required and cannot be empty".into(),
            severity: "error".into(),
        });
    }
    if scenario.test_id.trim().is_empty() {
        errors.push(ValidationError {
            path: "test_id".into(),
            message: "test_id is required and cannot be empty".into(),
            severity: "error".into(),
        });
    }
    if scenario.title.trim().is_empty() {
        errors.push(ValidationError {
            path: "title".into(),
            message: "title is required and cannot be empty".into(),
            severity: "error".into(),
        });
    }
    if scenario.difficulty > 10 {
        warnings.push(ValidationError {
            path: "difficulty".into(),
            message: format!(
                "difficulty {} exceeds recommended max of 10",
                scenario.difficulty
            ),
            severity: "warning".into(),
        });
    }
    if scenario.version.trim().is_empty() {
        errors.push(ValidationError {
            path: "version".into(),
            message: "version is required".into(),
            severity: "error".into(),
        });
    }
    if scenario.estimated_time_sec == 0 {
        warnings.push(ValidationError {
            path: "estimated_time_sec".into(),
            message: "estimated_time_sec is 0, consider setting an estimate".into(),
            severity: "warning".into(),
        });
    }

    // Steps presence
    if scenario.steps.is_empty() {
        errors.push(ValidationError {
            path: "steps".into(),
            message: "scenario must have at least one step".into(),
            severity: "error".into(),
        });
        return ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        };
    }

    // Collect all IDs for cross-reference validation
    let mut step_ids = HashSet::new();
    let mut all_choice_ids = HashSet::new();
    let mut all_action_ids = HashSet::new();
    let mut all_assertion_ids = HashSet::new();

    for (si, step) in scenario.steps.iter().enumerate() {
        let step_path = format!("steps[{}]", si);

        // Step ID uniqueness
        if step.step_id.trim().is_empty() {
            errors.push(ValidationError {
                path: format!("{}.step_id", step_path),
                message: "step_id is required".into(),
                severity: "error".into(),
            });
        } else if !step_ids.insert(step.step_id.clone()) {
            errors.push(ValidationError {
                path: format!("{}.step_id", step_path),
                message: format!("duplicate step_id '{}'", step.step_id),
                severity: "error".into(),
            });
        }

        // Step name
        if step.name.trim().is_empty() {
            errors.push(ValidationError {
                path: format!("{}.name", step_path),
                message: "step name is required".into(),
                severity: "error".into(),
            });
        }

        // Validate actions
        for (ai, action) in step.actions.iter().enumerate() {
            let action_path = format!("{}.actions[{}]", step_path, ai);
            if action.action_id.trim().is_empty() {
                errors.push(ValidationError {
                    path: format!("{}.action_id", action_path),
                    message: "action_id is required".into(),
                    severity: "error".into(),
                });
            } else if !all_action_ids.insert(action.action_id.clone()) {
                errors.push(ValidationError {
                    path: format!("{}.action_id", action_path),
                    message: format!("duplicate action_id '{}'", action.action_id),
                    severity: "error".into(),
                });
            }
            if action.title.trim().is_empty() {
                warnings.push(ValidationError {
                    path: format!("{}.title", action_path),
                    message: "action title is empty".into(),
                    severity: "warning".into(),
                });
            }
            if !VALID_ACTION_KINDS.contains(&action.kind.as_str()) {
                warnings.push(ValidationError {
                    path: format!("{}.kind", action_path),
                    message: format!(
                        "unknown action kind '{}', expected one of: {}",
                        action.kind,
                        VALID_ACTION_KINDS.join(", ")
                    ),
                    severity: "warning".into(),
                });
            }
        }

        // Validate choices
        for (ci, choice) in step.choices.iter().enumerate() {
            let choice_path = format!("{}.choices[{}]", step_path, ci);
            if choice.choice_id.trim().is_empty() {
                errors.push(ValidationError {
                    path: format!("{}.choice_id", choice_path),
                    message: "choice_id is required".into(),
                    severity: "error".into(),
                });
            } else if !all_choice_ids.insert(choice.choice_id.clone()) {
                errors.push(ValidationError {
                    path: format!("{}.choice_id", choice_path),
                    message: format!("duplicate choice_id '{}'", choice.choice_id),
                    severity: "error".into(),
                });
            }
            if choice.title.trim().is_empty() {
                warnings.push(ValidationError {
                    path: format!("{}.title", choice_path),
                    message: "choice title is empty".into(),
                    severity: "warning".into(),
                });
            }
        }

        // Validate assertions
        for (ai, assertion) in step.assertions.iter().enumerate() {
            let assertion_path = format!("{}.assertions[{}]", step_path, ai);
            if assertion.assertion_id.trim().is_empty() {
                errors.push(ValidationError {
                    path: format!("{}.assertion_id", assertion_path),
                    message: "assertion_id is required".into(),
                    severity: "error".into(),
                });
            } else if !all_assertion_ids.insert(assertion.assertion_id.clone()) {
                errors.push(ValidationError {
                    path: format!("{}.assertion_id", assertion_path),
                    message: format!("duplicate assertion_id '{}'", assertion.assertion_id),
                    severity: "error".into(),
                });
            }
            if assertion.description.trim().is_empty() {
                warnings.push(ValidationError {
                    path: format!("{}.description", assertion_path),
                    message: "assertion description is empty".into(),
                    severity: "warning".into(),
                });
            }
            if !VALID_ASSERTION_TYPES.contains(&assertion.type_.as_str()) {
                errors.push(ValidationError {
                    path: format!("{}.type", assertion_path),
                    message: format!(
                        "invalid assertion type '{}', expected one of: {}",
                        assertion.type_,
                        VALID_ASSERTION_TYPES.join(", ")
                    ),
                    severity: "error".into(),
                });
            }
            // Type-specific validation
            if assertion.type_ == "evidence_kind"
                && assertion.kind.as_ref().is_none_or(|k| k.trim().is_empty())
            {
                errors.push(ValidationError {
                    path: format!("{}.kind", assertion_path),
                    message: "evidence_kind assertion requires a non-empty 'kind' field".into(),
                    severity: "error".into(),
                });
            }
            if assertion.type_ == "event_contains"
                && assertion
                    .contains
                    .as_ref()
                    .is_none_or(|c| c.trim().is_empty())
            {
                errors.push(ValidationError {
                    path: format!("{}.contains", assertion_path),
                    message: "event_contains assertion requires a non-empty 'contains' field"
                        .into(),
                    severity: "error".into(),
                });
            }
        }
    }

    // Cross-reference: requires_choice_id must reference a defined choice_id
    for (si, step) in scenario.steps.iter().enumerate() {
        if let Some(ref choice_id) = step.requires_choice_id {
            if !choice_id.trim().is_empty() && !all_choice_ids.contains(choice_id) {
                errors.push(ValidationError {
                    path: format!("steps[{}].requires_choice_id", si),
                    message: format!(
                        "requires_choice_id '{}' does not match any defined choice_id",
                        choice_id
                    ),
                    severity: "error".into(),
                });
            }
        }
    }

    // Warn if no assertions exist at all
    if all_assertion_ids.is_empty() {
        warnings.push(ValidationError {
            path: "steps".into(),
            message: "no assertions defined in any step — verdicts will have no criteria".into(),
            severity: "warning".into(),
        });
    }

    // Check that at least one step has no requires_choice_id (entry point)
    let entry_steps = scenario
        .steps
        .iter()
        .filter(|s| {
            s.requires_choice_id.is_none()
                || s.requires_choice_id
                    .as_ref()
                    .is_none_or(|c| c.trim().is_empty())
        })
        .count();
    if entry_steps == 0 {
        errors.push(ValidationError {
            path: "steps".into(),
            message: "all steps have requires_choice_id — no entry point step exists".into(),
            severity: "error".into(),
        });
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Validate raw JSON bytes as a scenario, returning parse errors or validation results.
pub fn validate_scenario_json(json_bytes: &[u8]) -> ValidationResult {
    match serde_json::from_slice::<ScenarioDef>(json_bytes) {
        Ok(scenario) => validate_scenario(&scenario),
        Err(e) => ValidationResult {
            valid: false,
            errors: vec![ValidationError {
                path: "(root)".into(),
                message: format!("JSON parse error: {}", e),
                severity: "error".into(),
            }],
            warnings: vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_scenario() -> ScenarioDef {
        ScenarioDef {
            scenario_id: "s1".into(),
            test_id: "BAS-001".into(),
            title: "Test Scenario".into(),
            difficulty: 3,
            version: "1.0.0".into(),
            estimated_time_sec: 60,
            steps: vec![ScenarioStepDef {
                step_id: "step1".into(),
                name: "Step 1".into(),
                requires_choice_id: None,
                executor: None,
                command: None,
                args: None,
                actions: vec![ScenarioActionDef {
                    action_id: "act1".into(),
                    title: "Do something".into(),
                    kind: "execute".into(),
                }],
                choices: vec![],
                assertions: vec![ScenarioAssertionDef {
                    assertion_id: "assert1".into(),
                    description: "Check evidence".into(),
                    required: true,
                    type_: "evidence_kind".into(),
                    kind: Some("telemetry".into()),
                    contains: None,
                }],
            }],
        }
    }

    #[test]
    fn test_catalog_empty() {
        let catalog = ScenarioCatalog::empty();
        assert!(catalog.metas().is_empty());
        assert!(catalog.get_by_id("any").is_none());
        assert!(catalog.get_by_test_id("any").is_none());
    }

    #[test]
    fn test_validate_valid_scenario() {
        let scenario = make_valid_scenario();
        let result = validate_scenario(&scenario);
        assert!(result.valid, "errors: {:?}", result.errors);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_empty_scenario_id() {
        let mut s = make_valid_scenario();
        s.scenario_id = "".into();
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.path == "scenario_id"));
    }

    #[test]
    fn test_validate_empty_test_id() {
        let mut s = make_valid_scenario();
        s.test_id = "".into();
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.path == "test_id"));
    }

    #[test]
    fn test_validate_empty_title() {
        let mut s = make_valid_scenario();
        s.title = "".into();
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.path == "title"));
    }

    #[test]
    fn test_validate_empty_version() {
        let mut s = make_valid_scenario();
        s.version = "".into();
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.path == "version"));
    }

    #[test]
    fn test_validate_no_steps() {
        let mut s = make_valid_scenario();
        s.steps = vec![];
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.path == "steps" && e.message.contains("at least one step")));
    }

    #[test]
    fn test_validate_high_difficulty() {
        let mut s = make_valid_scenario();
        s.difficulty = 15;
        let result = validate_scenario(&s);
        assert!(result.valid); // warning, not error
        assert!(result.warnings.iter().any(|w| w.path == "difficulty"));
    }

    #[test]
    fn test_validate_zero_estimated_time() {
        let mut s = make_valid_scenario();
        s.estimated_time_sec = 0;
        let result = validate_scenario(&s);
        assert!(result.valid); // warning, not error
        assert!(result
            .warnings
            .iter()
            .any(|w| w.path == "estimated_time_sec"));
    }

    #[test]
    fn test_validate_duplicate_step_id() {
        let mut s = make_valid_scenario();
        let step2 = ScenarioStepDef {
            step_id: "step1".into(),
            name: "Step 2".into(),
            requires_choice_id: None,
            executor: None,
            command: None,
            args: None,
            actions: vec![],
            choices: vec![],
            assertions: vec![],
        };
        s.steps.push(step2);
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("duplicate step_id")));
    }

    #[test]
    fn test_validate_duplicate_action_id() {
        let mut s = make_valid_scenario();
        s.steps[0].actions.push(ScenarioActionDef {
            action_id: "act1".into(),
            title: "Another".into(),
            kind: "execute".into(),
        });
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("duplicate action_id")));
    }

    #[test]
    fn test_validate_duplicate_choice_id() {
        let mut s = make_valid_scenario();
        s.steps[0].choices = vec![
            ScenarioChoiceDef {
                choice_id: "c1".into(),
                title: "A".into(),
            },
            ScenarioChoiceDef {
                choice_id: "c1".into(),
                title: "B".into(),
            },
        ];
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("duplicate choice_id")));
    }

    #[test]
    fn test_validate_duplicate_assertion_id() {
        let mut s = make_valid_scenario();
        s.steps[0].assertions.push(ScenarioAssertionDef {
            assertion_id: "assert1".into(),
            description: "Another".into(),
            required: false,
            type_: "evidence_kind".into(),
            kind: Some("file".into()),
            contains: None,
        });
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("duplicate assertion_id")));
    }

    #[test]
    fn test_validate_invalid_action_kind() {
        let mut s = make_valid_scenario();
        s.steps[0].actions[0].kind = "invalid_kind".into();
        let result = validate_scenario(&s);
        assert!(result.valid); // warning, not error
        assert!(result
            .warnings
            .iter()
            .any(|w| w.message.contains("unknown action kind")));
    }

    #[test]
    fn test_validate_invalid_assertion_type() {
        let mut s = make_valid_scenario();
        s.steps[0].assertions[0].type_ = "bad_type".into();
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("invalid assertion type")));
    }

    #[test]
    fn test_validate_evidence_kind_missing_kind() {
        let mut s = make_valid_scenario();
        s.steps[0].assertions[0].type_ = "evidence_kind".into();
        s.steps[0].assertions[0].kind = None;
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("requires a non-empty 'kind'")));
    }

    #[test]
    fn test_validate_event_contains_missing_contains() {
        let mut s = make_valid_scenario();
        s.steps[0].assertions[0].type_ = "event_contains".into();
        s.steps[0].assertions[0].kind = None;
        s.steps[0].assertions[0].contains = None;
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("requires a non-empty 'contains'")));
    }

    #[test]
    fn test_validate_requires_choice_id_invalid() {
        let mut s = make_valid_scenario();
        s.steps[0].requires_choice_id = Some("nonexistent_choice".into());
        s.steps.push(ScenarioStepDef {
            step_id: "step2".into(),
            name: "Entry".into(),
            requires_choice_id: None,
            executor: None,
            command: None,
            args: None,
            actions: vec![],
            choices: vec![],
            assertions: vec![],
        });
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("does not match any defined choice_id")));
    }

    #[test]
    fn test_validate_no_entry_point() {
        let mut s = make_valid_scenario();
        s.steps[0].choices = vec![ScenarioChoiceDef {
            choice_id: "c1".into(),
            title: "A".into(),
        }];
        s.steps[0].requires_choice_id = Some("c1".into());
        let result = validate_scenario(&s);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("no entry point")));
    }

    #[test]
    fn test_validate_no_assertions_warning() {
        let mut s = make_valid_scenario();
        s.steps[0].assertions = vec![];
        let result = validate_scenario(&s);
        assert!(result.valid); // warning only
        assert!(result
            .warnings
            .iter()
            .any(|w| w.message.contains("no assertions")));
    }

    #[test]
    fn test_validate_scenario_json_invalid() {
        let result = validate_scenario_json(b"not valid json{{{");
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("JSON parse error")));
    }

    #[test]
    fn test_validate_scenario_json_valid() {
        let scenario = make_valid_scenario();
        let json = serde_json::to_vec(&scenario).unwrap();
        let result = validate_scenario_json(&json);
        assert!(result.valid, "errors: {:?}", result.errors);
    }
}
