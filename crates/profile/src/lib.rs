//! profiles. named server sets for different contexts.
//!
//! `smith profile work` switches your active server set. `smith profile
//! personal` switches back. one command. your work tools don't leak into
//! your personal agent and vice versa.

use smith_config::SmithConfig;

pub fn switch(config: &mut SmithConfig, profile: &str) -> Result<String, String> {
    config.set_profile(profile);
    Ok(format!("switched to profile: {}", profile))
}

pub fn list(config: &SmithConfig) -> Vec<String> {
    config.list_profiles()
}

pub fn current(config: &SmithConfig) -> &str {
    &config.active_profile
}

pub fn assign(config: &mut SmithConfig, server_name: &str, profile: &str) -> Result<(), String> {
    let server = config
        .servers
        .iter_mut()
        .find(|s| s.name == server_name)
        .ok_or(format!("server not found: {}", server_name))?;
    server.profile = profile.to_string();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use smith_config::{ServerEntry, SmithConfig};

    fn test_config() -> SmithConfig {
        let mut cfg = SmithConfig::default();
        cfg.servers.push(ServerEntry {
            name: "github".to_string(),
            source: "npm".to_string(),
            version: "1.0.0".to_string(),
            command: "npx -y @modelcontextprotocol/server-github".to_string(),
            args: vec![],
            env: std::collections::HashMap::new(),
            enabled: true,
            profile: "default".to_string(),
        });
        cfg.servers.push(ServerEntry {
            name: "filesystem".to_string(),
            source: "npm".to_string(),
            version: "1.0.0".to_string(),
            command: "npx -y @modelcontextprotocol/server-filesystem".to_string(),
            args: vec!["/tmp".to_string()],
            env: std::collections::HashMap::new(),
            enabled: true,
            profile: "work".to_string(),
        });
        cfg
    }

    #[test]
    fn test_switch_profile() {
        let mut cfg = test_config();
        let result = switch(&mut cfg, "work");
        assert!(result.is_ok());
        assert_eq!(current(&cfg), "work");
    }

    #[test]
    fn test_switch_to_new_profile() {
        let mut cfg = test_config();
        let result = switch(&mut cfg, "personal");
        assert!(result.is_ok());
        assert_eq!(current(&cfg), "personal");
    }

    #[test]
    fn test_list_profiles() {
        let cfg = test_config();
        let profiles = list(&cfg);
        assert!(!profiles.is_empty());
    }

    #[test]
    fn test_current_profile() {
        let cfg = test_config();
        assert_eq!(current(&cfg), "default");
    }

    #[test]
    fn test_assign_server_to_profile() {
        let mut cfg = test_config();
        let result = assign(&mut cfg, "github", "work");
        assert!(result.is_ok());
        let server = cfg.servers.iter().find(|s| s.name == "github").unwrap();
        assert_eq!(server.profile, "work");
    }

    #[test]
    fn test_assign_nonexistent_server() {
        let mut cfg = test_config();
        let result = assign(&mut cfg, "nonexistent", "work");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("server not found"));
    }
}
