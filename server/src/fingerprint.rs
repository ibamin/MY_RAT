use regex::Regex;
use serde::Deserialize;

use crate::models::FingerprintCandidate;

#[derive(Debug, Deserialize, Clone)]
pub struct FingerprintRule {
    pub service: String,
    pub product: Option<String>,
    pub regex: String,
    pub version_group: Option<usize>,
    pub confidence: Option<f32>,
}

#[derive(Clone)]
pub struct FingerprintMatcher {
    rules: Vec<CompiledRule>,
}

#[derive(Clone)]
struct CompiledRule {
    service: String,
    product: Option<String>,
    regex: Regex,
    version_group: Option<usize>,
    confidence: f32,
}

impl FingerprintMatcher {
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn from_rules(rules: Vec<FingerprintRule>) -> Result<Self, regex::Error> {
        let mut compiled = Vec::with_capacity(rules.len());
        for rule in rules {
            let regex = Regex::new(&rule.regex)?;
            compiled.push(CompiledRule {
                service: rule.service,
                product: rule.product,
                regex,
                version_group: rule.version_group,
                confidence: rule.confidence.unwrap_or(0.7),
            });
        }
        Ok(Self { rules: compiled })
    }

    pub fn load_from_json_path(path: &str) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let rules: Vec<FingerprintRule> =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        Self::from_rules(rules).map_err(|e| e.to_string())
    }

    pub fn match_banner(&self, banner: &str, limit: usize) -> Vec<FingerprintCandidate> {
        let mut out = Vec::new();

        for rule in &self.rules {
            if let Some(caps) = rule.regex.captures(banner) {
                let version = rule
                    .version_group
                    .and_then(|idx| caps.get(idx))
                    .map(|m| m.as_str().to_string());

                out.push(FingerprintCandidate {
                    service: rule.service.clone(),
                    product: rule.product.clone(),
                    version,
                    confidence: rule.confidence,
                });

                if out.len() >= limit {
                    break;
                }
            }
        }

        out
    }
}
