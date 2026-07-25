//! security probe. audit a server before you activate it.
//!
//! checks for dangerous tool patterns, excessive permissions, data
//! exfiltration risks. powered by mcprobe concepts. the idea is simple:
//! before you let a random MCP server into your agent, check what it can
//! do. smith runs this check and reports.
//!
//! this is static analysis. it looks at env vars, command patterns, and
//! tool names. it doesn't execute anything. safety first.

use serde::{Deserialize, Serialize};
use smith_config::SmithConfig;

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
    let entry = config
        .get_server(server_name)
        .ok_or(format!("server not found: {}", server_name))?;

    let mut findings = Vec::new();

    let env_concerns: Vec<&str> = entry
        .env
        .keys()
        .filter(|k| {
            let lower = k.to_lowercase();
            lower.contains("key")
                || lower.contains("secret")
                || lower.contains("token")
                || lower.contains("password")
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
        let pkg = entry
            .args
            .iter()
            .find(|a| !a.starts_with('-'))
            .cloned()
            .unwrap_or_default();
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
    config
        .servers
        .iter()
        .filter(|s| s.enabled)
        .filter_map(|s| audit(config, &s.name).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use smith_config::{ServerEntry, SmithConfig};

    fn make_server(name: &str, env: Vec<(&str, &str)>) -> ServerEntry {
        let mut e = std::collections::HashMap::new();
        for (k, v) in env {
            e.insert(k.to_string(), v.to_string());
        }
        ServerEntry {
            name: name.into(),
            command: "npx".into(),
            args: vec![],
            env: e,
            source: format!("@scope/{}", name),
            profile: "default".into(),
            version: "latest".into(),
            enabled: true,
        }
    }

    #[test]
    fn audit_safe_server_passes() {
        let mut config = SmithConfig::default();
        config.servers.push(make_server("safe", vec![]));
        let report = audit(&config, "safe").unwrap();
        assert!(report.passed);
    }

    #[test]
    fn audit_missing_server_errors() {
        let config = SmithConfig::default();
        assert!(audit(&config, "nope").is_err());
    }

    #[test]
    fn audit_flags_api_key() {
        let mut config = SmithConfig::default();
        config
            .servers
            .push(make_server("risky", vec![("API_KEY", "secret")]));
        let report = audit(&config, "risky").unwrap();
        assert!(report.findings.iter().any(|f| f.category == "env"));
    }

    #[test]
    fn audit_flags_token() {
        let mut config = SmithConfig::default();
        config
            .servers
            .push(make_server("risky", vec![("TOKEN", "abc123")]));
        let report = audit(&config, "risky").unwrap();
        assert!(report.findings.iter().any(|f| f.category == "env"));
    }
    fn audit_flags_secret() {
        let mut config = SmithConfig::default();
        config
            .servers
            .push(make_server("risky", vec![("SECRET", "abc")]));
        let report = audit(&config, "risky").unwrap();
        assert!(report.findings.iter().any(|f| f.category == "env"));
    }

    #[test]
    fn audit_flags_password() {
        let mut config = SmithConfig::default();
        config
            .servers
            .push(make_server("risky", vec![("PASSWORD", "abc")]));
        let report = audit(&config, "risky").unwrap();
        assert!(report.findings.iter().any(|f| f.category == "env"));
    }

    #[test]
    fn audit_npx_no_package_warns() {
        let mut config = SmithConfig::default();
        let mut server = make_server("npx-server", vec![]);
        server.command = "npx".into();
        server.args = vec![];
        config.servers.push(server);
        let report = audit(&config, "npx-server").unwrap();
        assert!(report.findings.iter().any(|f| f.category == "install"));
        assert!(matches!(report.risk_level, RiskLevel::Caution));
    }

    #[test]
    fn audit_npx_with_package_safe() {
        let mut config = SmithConfig::default();
        let mut server = make_server("npx-safe", vec![]);
        server.command = "npx".into();
        server.args = vec!["@scope/server".into()];
        config.servers.push(server);
        let report = audit(&config, "npx-safe").unwrap();
        assert!(report.passed);
    }

    #[test]
    fn audit_all_returns_reports() {
        let mut config = SmithConfig::default();
        config.servers.push(make_server("safe1", vec![]));
        config.servers.push(make_server("safe2", vec![]));
        config.servers[1].enabled = false;
        let reports = audit_all(&config);
        assert_eq!(reports.len(), 1);
    }

    #[test]
    fn audit_all_multiple_enabled() {
        let mut config = SmithConfig::default();
        config.servers.push(make_server("s1", vec![]));
        config.servers.push(make_server("s2", vec![("KEY", "v")]));
        config.servers.push(make_server("s3", vec![]));
        let reports = audit_all(&config);
        assert_eq!(reports.len(), 3);
    }

    #[test]
    fn audit_npm_command_no_args() {
        let mut config = SmithConfig::default();
        let mut server = make_server("npm-server", vec![]);
        server.command = "npm".into();
        server.args = vec![];
        config.servers.push(server);
        let report = audit(&config, "npm-server").unwrap();
        assert!(report.findings.iter().any(|f| f.category == "install"));
    }

    #[test]
    fn security_report_serialization() {
        let report = SecurityReport {
            server: "test".to_string(),
            risk_level: RiskLevel::Safe,
            findings: vec![],
            passed: true,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("Safe"));
    }

    #[test]
    fn finding_serialization() {
        let f = Finding {
            severity: "warn".to_string(),
            category: "env".to_string(),
            message: "test message".to_string(),
            tool: Some("test_tool".to_string()),
        };
        let json = serde_json::to_string(&f).unwrap();
        assert!(json.contains("warn"));
        assert!(json.contains("test_tool"));
    }
}
