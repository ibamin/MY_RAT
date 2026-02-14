use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
