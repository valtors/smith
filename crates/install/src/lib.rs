//! fetch from npm, git, or local path. verify. activate.
//!
//! smith doesn't run a registry. it resolves sources: npm packages,
//! git repos, local paths. each gets turned into a command + args that
//! the compose layer can spawn. the install is just config writing.

use smith_mcp_config::{parse_source, ServerEntry, SmithConfig, SourceType};
use std::collections::HashMap;

pub struct InstallResult {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub message: String,
}

pub fn install(
    config: &mut SmithConfig,
    source: &str,
    profile: Option<&str>,
) -> Result<InstallResult, String> {
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
            let name = pkg.split('/').next_back().unwrap_or(pkg).to_string();
            let command = "npx".to_string();
            let args = vec!["-y".to_string(), pkg.clone()];
            Ok((name, command, args))
        }
        SourceType::GitRepo(repo) => {
            let name = repo.split('/').next_back().unwrap_or(repo).to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use smith_mcp_config::SmithConfig;

    #[test]
    fn install_npm_creates_npx_command() {
        let mut config = SmithConfig::default();
        let result = install(&mut config, "@scope/mypackage", None).unwrap();
        assert_eq!(result.command, "npx");
        assert!(result.args.len() > 0);
        assert_eq!(config.servers.len(), 1);
    }

    #[test]
    fn install_github_creates_git_clone() {
        let mut config = SmithConfig::default();
        let result = install(&mut config, "user/repo", None).unwrap();
        assert_eq!(result.name, "repo");
    }

    #[test]
    fn install_replaces_existing() {
        let mut config = SmithConfig::default();
        install(&mut config, "@scope/pkg", None).unwrap();
        install(&mut config, "@scope/pkg", None).unwrap();
        assert_eq!(config.servers.len(), 1);
    }

    #[test]
    fn uninstall_removes() {
        let mut config = SmithConfig::default();
        install(&mut config, "@scope/pkg", None).unwrap();
        assert!(uninstall(&mut config, "pkg").unwrap());
        assert_eq!(config.servers.len(), 0);
    }

    #[test]
    fn uninstall_missing_returns_false() {
        let mut config = SmithConfig::default();
        assert!(!uninstall(&mut config, "nope").unwrap());
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;

    #[test]
    fn install_npm_scoped() {
        let mut config = SmithConfig::default();
        let result = install(&mut config, "@scope/my-server", None).unwrap();
        assert_eq!(result.name, "my-server");
        assert_eq!(result.command, "npx");
        assert!(result.args.contains(&"-y".to_string()));
    }

    #[test]
    fn install_git_repo() {
        let mut config = SmithConfig::default();
        let result = install(&mut config, "github:user/repo", None).unwrap();
        assert_eq!(result.name, "repo");
        assert_eq!(result.command, "npx");
    }

    #[test]
    fn install_with_profile() {
        let mut config = SmithConfig::default();
        let result = install(&mut config, "@scope/server", Some("dev")).unwrap();
        assert_eq!(result.name, "server");
    }

    #[test]
    fn install_result_has_message() {
        let mut config = SmithConfig::default();
        let result = install(&mut config, "@scope/server", None).unwrap();
        assert_eq!(result.message, "installed");
    }

    #[test]
    fn install_adds_to_config() {
        let mut config = SmithConfig::default();
        assert_eq!(config.servers.len(), 0);
        install(&mut config, "@scope/server", None).unwrap();
        assert_eq!(config.servers.len(), 1);
    }

    #[test]
    fn install_multiple() {
        let mut config = SmithConfig::default();
        install(&mut config, "@scope/server1", None).unwrap();
        install(&mut config, "@scope/server2", None).unwrap();
        assert_eq!(config.servers.len(), 2);
    }

    #[test]
    fn install_default_profile() {
        let mut config = SmithConfig::default();
        install(&mut config, "@scope/server", None).unwrap();
        assert_eq!(config.servers[0].profile, "default");
    }

    #[test]
    fn install_custom_profile() {
        let mut config = SmithConfig::default();
        install(&mut config, "@scope/server", Some("staging")).unwrap();
        assert_eq!(config.servers[0].profile, "staging");
    }

    #[test]
    fn resolve_npm_package_name() {
        let st = SourceType::Npm("@org/pkg".to_string());
        let (name, cmd, args) = resolve_server(&st).unwrap();
        assert_eq!(name, "pkg");
        assert_eq!(cmd, "npx");
        assert!(args.len() >= 2);
    }
}
