//! fetch from npm, git, or local path. verify. activate.
//!
//! smith doesn't run a registry. it resolves sources: npm packages,
//! git repos, local paths. each gets turned into a command + args that
//! the compose layer can spawn. the install is just config writing.

use smith_config::{parse_source, ServerEntry, SmithConfig, SourceType};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct InstallResult {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub message: String,
}

pub fn install(config: &mut SmithConfig, source: &str, profile: Option<&str>) -> Result<InstallResult, String> {
    let source_type = parse_source(source);
    let (name, command, args) = resolve_server(&source_type)?;

    let entry = ServerEntry {
        name: name.clone(),
        source: source.to_string(),
        version: "latest".to_string(),
        command: command.clone(),
        args: args.clone(),
        env: HashMap::new(),
        enabled: true,
        profile: profile.unwrap_or("default").to_string(),
    };

    config.add_server(entry);

    Ok(InstallResult {
        name,
        command,
        args,
        message: "installed".to_string(),
    })
}

fn resolve_server(source_type: &SourceType) -> Result<(String, String, Vec<String>), String> {
    match source_type {
        SourceType::Npm(pkg) => {
            let name = pkg.split('/').last().unwrap_or(pkg).to_string();
            let command = "npx".to_string();
            let args = vec!["-y".to_string(), pkg.clone()];
            Ok((name, command, args))
        }
        SourceType::GitRepo(repo) => {
            let name = repo.split('/').last().unwrap_or(repo).to_string();
            let command = "npx".to_string();
            let args = vec!["-y".to_string(), format!("github:{}", repo)];
            Ok((name, command, args))
        }
        SourceType::Git(url) => {
            let name = url
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("server")
                .to_string();
            let command = "npx".to_string();
            let args = vec!["-y".to_string(), format!("git+{}", url)];
            Ok((name, command, args))
        }
        SourceType::Local(path) => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or("local".to_string());
            let abs = std::fs::canonicalize(path).map_err(|e| format!("path not found: {}", e))?;
            Ok((name, abs.to_string_lossy().to_string(), vec![]))
        }
    }
}

pub fn uninstall(config: &mut SmithConfig, name: &str) -> Result<bool, String> {
    Ok(config.remove_server(name))
}

pub fn update(config: &mut SmithConfig, name: Option<&str>) -> Result<Vec<String>, String> {
    let mut updated = Vec::new();
    for server in &mut config.servers {
        if let Some(n) = name {
            if server.name != n {
                continue;
            }
        }
        server.version = "latest".to_string();
        updated.push(server.name.clone());
    }
    Ok(updated)
}
