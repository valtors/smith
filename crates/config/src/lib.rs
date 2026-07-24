//! smith config. reads and writes ~/.smith/config.json.
//!
//! any MCP-compatible agent can read this config. smith is the source of
//! truth for what's installed, what's active, and what profile you're in.
//! one json file. no database, no registry service, no lock-in.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub source: String,
    pub version: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmithConfig {
    pub servers: Vec<ServerEntry>,
    pub active_profile: String,
    pub smith_dir: String,
}

impl Default for SmithConfig {
    fn default() -> Self {
        Self {
            servers: vec![],
            active_profile: "default".to_string(),
            smith_dir: smith_dir().to_string_lossy().to_string(),
        }
    }
}

pub fn smith_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".smith")
}

pub fn config_path() -> PathBuf {
    smith_dir().join("config.json")
}

pub fn load() -> SmithConfig {
    let path = config_path();
    if !path.exists() {
        return SmithConfig::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => SmithConfig::default(),
    }
}

pub fn save(config: &SmithConfig) -> Result<(), String> {
    let dir = smith_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = config_path();
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

impl SmithConfig {
    pub fn add_server(&mut self, entry: ServerEntry) {
        self.servers.retain(|s| s.name != entry.name);
        self.servers.push(entry);
    }

    pub fn remove_server(&mut self, name: &str) -> bool {
        let before = self.servers.len();
        self.servers.retain(|s| s.name != name);
        self.servers.len() < before
    }

    pub fn get_server(&self, name: &str) -> Option<&ServerEntry> {
        self.servers.iter().find(|s| s.name == name)
    }

    pub fn active_servers(&self) -> Vec<&ServerEntry> {
        self.servers
            .iter()
            .filter(|s| s.enabled && s.profile == self.active_profile)
            .collect()
    }

    pub fn set_profile(&mut self, profile: &str) {
        self.active_profile = profile.to_string();
    }

    pub fn list_profiles(&self) -> Vec<String> {
        let mut profiles: Vec<String> = self.servers.iter().map(|s| s.profile.clone()).collect();
        profiles.sort();
        profiles.dedup();
        if !profiles.contains(&"default".to_string()) {
            profiles.insert(0, "default".to_string());
        }
        profiles
    }
}

pub fn parse_source(source: &str) -> SourceType {
    if source.starts_with('@') {
        SourceType::Npm(source.to_string())
    } else if source.contains('/') && !source.starts_with('.') && !source.starts_with('/') {
        if source.starts_with("http://") || source.starts_with("https://") {
            SourceType::Git(source.to_string())
        } else if source.contains("github.com") {
            SourceType::Git(format!("https://{}", source))
        } else {
            SourceType::GitRepo(source.to_string())
        }
    } else if source.starts_with('.') || source.starts_with('/') {
        SourceType::Local(PathBuf::from(source))
    } else {
        SourceType::Npm(source.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum SourceType {
    Npm(String),
    Git(String),
    GitRepo(String),
    Local(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_no_servers() {
        let c = SmithConfig::default();
        assert!(c.servers.is_empty());
        assert_eq!(c.active_profile, "default");
    }

    #[test]
    fn parse_npm_package() {
        let st = parse_source("@scope/pkg");
        assert!(matches!(st, SourceType::Npm(_)));
    }

    #[test]
    fn parse_plain_npm() {
        let st = parse_source("mypackage");
        assert!(matches!(st, SourceType::Npm(_)));
    }

    #[test]
    fn parse_github_shorthand() {
        let st = parse_source("user/repo");
        assert!(matches!(st, SourceType::GitRepo(_)));
    }

    #[test]
    fn parse_https_git() {
        let st = parse_source("https://github.com/user/repo");
        assert!(matches!(st, SourceType::Git(_)));
    }

    #[test]
    fn parse_local_relative() {
        let st = parse_source("./local/path");
        assert!(matches!(st, SourceType::Local(_)));
    }

    #[test]
    fn parse_local_absolute() {
        let st = parse_source("/usr/local/bin/server");
        assert!(matches!(st, SourceType::Local(_)));
    }

    #[test]
    fn set_and_get_profile() {
        let mut c = SmithConfig::default();
        c.set_profile("work");
        assert_eq!(c.active_profile, "work");
    }

    #[test]
    fn active_servers_filters_by_profile() {
        let mut c = SmithConfig::default();
        let s1 = ServerEntry {
            name: "a".into(),
            command: "npx".into(),
            args: vec![],
            env: Default::default(),
            source: "@scope/a".into(),
            profile: "work".into(),
            enabled: true,
            version: "latest".into(),
        };
        let s2 = ServerEntry {
            name: "b".into(),
            command: "npx".into(),
            args: vec![],
            env: Default::default(),
            source: "@scope/b".into(),
            profile: "personal".into(),
            enabled: true,
            version: "latest".into(),
        };
        c.servers.push(s1);
        c.servers.push(s2);
        c.set_profile("work");
        assert_eq!(c.active_servers().len(), 1);
        assert_eq!(c.active_servers()[0].name, "a");
    }

    #[test]
    fn active_servers_default_includes_unprofiled() {
        let mut c = SmithConfig::default();
        let s = ServerEntry {
            name: "a".into(),
            command: "npx".into(),
            args: vec![],
            env: Default::default(),
            source: "@scope/a".into(),
            profile: "default".into(),
            enabled: true,
            version: "latest".into(),
        };
        c.servers.push(s);
        assert_eq!(c.active_servers().len(), 1);
    }
}
