use smith_config::SmithConfig;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub server: String,
    pub risk_level: RiskLevel,
    pub findings: Vec<Finding>,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Caution,
    Dangerous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub tool: Option<String>,
}

pub fn audit(config: &SmithConfig, server_name: &str) -> Result<SecurityReport, String> {
    let entry = config.get_server(server_name)
        .ok_or(format!("server not found: {}", server_name))?;

    let mut findings = Vec::new();

    let env_concerns: Vec<&str> = entry.env.keys()
        .filter(|k| {
            let lower = k.to_lowercase();
            lower.contains("key") || lower.contains("secret") || lower.contains("token") || lower.contains("password")
        })
        .map(|k| k.as_str())
        .collect();

    for key in &env_concerns {
        findings.push(Finding {
            severity: "info".to_string(),
            category: "env".to_string(),
            message: format!("env var {} contains sensitive credential pattern", key),
            tool: None,
        });
    }

    if entry.command == "npx" || entry.command == "npm" {
        let pkg = entry.args.iter().find(|a| !a.starts_with('-')).cloned().unwrap_or_default();
        if pkg.is_empty() {
            findings.push(Finding {
                severity: "warn".to_string(),
                category: "install".to_string(),
                message: "npx command without explicit package name".to_string(),
                tool: None,
            });
        }
    }

    let risk = if findings.iter().any(|f| f.severity == "critical") {
        RiskLevel::Dangerous
    } else if findings.iter().any(|f| f.severity == "warn") {
        RiskLevel::Caution
    } else {
        RiskLevel::Safe
    };

    let passed = !matches!(risk, RiskLevel::Dangerous);

    Ok(SecurityReport {
        server: server_name.to_string(),
        risk_level: risk,
        findings,
        passed,
    })
}

pub fn audit_all(config: &SmithConfig) -> Vec<SecurityReport> {
    config.servers.iter()
        .filter(|s| s.enabled)
        .filter_map(|s| audit(config, &s.name).ok())
        .collect()
}
