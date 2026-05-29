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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(
        service: &str,
        product: Option<&str>,
        regex: &str,
        version_group: Option<usize>,
        confidence: Option<f32>,
    ) -> FingerprintRule {
        FingerprintRule {
            service: service.into(),
            product: product.map(|p| p.into()),
            regex: regex.into(),
            version_group,
            confidence,
        }
    }

    #[test]
    fn test_empty_matcher() {
        let matcher = FingerprintMatcher::empty();
        let results = matcher.match_banner("anything", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_from_rules_valid() {
        let rules = vec![make_rule(
            "http",
            Some("nginx"),
            r"nginx/([\d.]+)",
            Some(1),
            Some(0.85),
        )];
        let matcher = FingerprintMatcher::from_rules(rules);
        assert!(matcher.is_ok());
    }

    #[test]
    fn test_from_rules_invalid_regex() {
        let rules = vec![make_rule("http", None, r"[invalid(", None, None)];
        let result = FingerprintMatcher::from_rules(rules);
        assert!(result.is_err());
    }

    #[test]
    fn test_match_banner_single_match() {
        let rules = vec![make_rule(
            "http",
            Some("nginx"),
            r"nginx/([\d.]+)",
            Some(1),
            Some(0.85),
        )];
        let matcher = FingerprintMatcher::from_rules(rules).unwrap();
        let results = matcher.match_banner("Server: nginx/1.24.0\r\n", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].service, "http");
        assert_eq!(results[0].product.as_deref(), Some("nginx"));
        assert_eq!(results[0].version.as_deref(), Some("1.24.0"));
        assert!((results[0].confidence - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn test_match_banner_no_match() {
        let rules = vec![make_rule(
            "http",
            Some("nginx"),
            r"nginx/([\d.]+)",
            Some(1),
            Some(0.85),
        )];
        let matcher = FingerprintMatcher::from_rules(rules).unwrap();
        let results = matcher.match_banner("Server: apache/2.4.41", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_match_banner_multiple_matches() {
        let rules = vec![
            make_rule("http", Some("nginx"), r"nginx", None, Some(0.8)),
            make_rule("http", Some("server"), r"Server:", None, Some(0.5)),
        ];
        let matcher = FingerprintMatcher::from_rules(rules).unwrap();
        let results = matcher.match_banner("Server: nginx/1.24.0", 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_match_banner_limit() {
        let rules = vec![
            make_rule("http", Some("nginx"), r"nginx", None, Some(0.8)),
            make_rule("http", Some("server"), r"Server:", None, Some(0.5)),
        ];
        let matcher = FingerprintMatcher::from_rules(rules).unwrap();
        let results = matcher.match_banner("Server: nginx/1.24.0", 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].product.as_deref(), Some("nginx"));
    }

    #[test]
    fn test_match_banner_version_group_extraction() {
        let rules = vec![make_rule(
            "ssh",
            Some("OpenSSH"),
            r"SSH-[\d.]+-OpenSSH_([\d.p]+)",
            Some(1),
            Some(0.9),
        )];
        let matcher = FingerprintMatcher::from_rules(rules).unwrap();
        let results = matcher.match_banner("SSH-2.0-OpenSSH_8.9p1", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].version.as_deref(), Some("8.9p1"));
    }

    #[test]
    fn test_match_banner_no_version_group() {
        let rules = vec![make_rule("http", Some("nginx"), r"nginx", None, Some(0.7))];
        let matcher = FingerprintMatcher::from_rules(rules).unwrap();
        let results = matcher.match_banner("nginx/1.24.0", 10);
        assert_eq!(results.len(), 1);
        assert!(results[0].version.is_none());
    }

    #[test]
    fn test_default_confidence() {
        let rules = vec![make_rule("http", None, r"HTTP", None, None)];
        let matcher = FingerprintMatcher::from_rules(rules).unwrap();
        let results = matcher.match_banner("HTTP/1.1 200 OK", 10);
        assert_eq!(results.len(), 1);
        assert!((results[0].confidence - 0.7).abs() < f32::EPSILON);
    }
}
