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
