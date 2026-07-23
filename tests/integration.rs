use smith_config::{SmithConfig, ServerEntry, parse_source, SourceType, save, load, config_path};
use smith_install::{install, uninstall, update};
use smith_profile;
use smith_secure;
use std::collections::HashMap;

fn tmp_config_path() -> String {
    format!("/home/container/smith-test-{}.json", std::process::id())
}

fn fresh_config() -> SmithConfig {
    SmithConfig::default()
}

#[test]
fn parse_npm_source() {
    let st = parse_source("@modelcontextprotocol/filesystem");
    assert!(matches!(st, SourceType::Npm(_)));
}

#[test]
fn parse_github_source() {
    let st = parse_source("valtors/cairn");
    assert!(matches!(st, SourceType::GitRepo(_)));
}

#[test]
fn parse_git_url() {
    let st = parse_source("https://github.com/valtors/cairn");
    assert!(matches!(st, SourceType::Git(_)));
}

#[test]
fn parse_local_path() {
    let st = parse_source("./my-server");
    assert!(matches!(st, SourceType::Local(_)));
}

#[test]
fn install_adds_server() {
    let mut config = fresh_config();
    let result = install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    assert_eq!(result.name, "filesystem");
    assert_eq!(result.command, "npx");
    assert_eq!(config.servers.len(), 1);
}

#[test]
fn install_then_list() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    install(&mut config, "valtors/cairn", None).unwrap();
    assert_eq!(config.servers.len(), 2);
}

#[test]
fn install_replaces_same_name() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    assert_eq!(config.servers.len(), 1);
}

#[test]
fn uninstall_removes_server() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    assert!(uninstall(&mut config, "filesystem").unwrap());
    assert_eq!(config.servers.len(), 0);
}

#[test]
fn uninstall_nonexistent_returns_false() {
    let mut config = fresh_config();
    assert!(!uninstall(&mut config, "nope").unwrap());
}

#[test]
fn active_servers_filter_by_profile() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", Some("work")).unwrap();
    install(&mut config, "valtors/cairn", Some("personal")).unwrap();

    config.set_profile("work");
    assert_eq!(config.active_servers().len(), 1);
    assert_eq!(config.active_servers()[0].name, "filesystem");

    config.set_profile("personal");
    assert_eq!(config.active_servers().len(), 1);
    assert_eq!(config.active_servers()[0].name, "cairn");
}

#[test]
fn profile_switch() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", Some("work")).unwrap();
    install(&mut config, "valtors/cairn", Some("personal")).unwrap();

    let msg = smith_profile::switch(&mut config, "work").unwrap();
    assert!(msg.contains("work"));
    assert_eq!(smith_profile::current(&config), "work");

    smith_profile::switch(&mut config, "personal").unwrap();
    assert_eq!(smith_profile::current(&config), "personal");
}

#[test]
fn profile_list_includes_default() {
    let config = fresh_config();
    let profiles = smith_profile::list(&config);
    assert!(profiles.contains(&"default".to_string()));
}

#[test]
fn profile_assign() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    smith_profile::assign(&mut config, "filesystem", "custom").unwrap();
    let server = config.get_server("filesystem").unwrap();
    assert_eq!(server.profile, "custom");
}

#[test]
fn secure_audit_safe_server() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    let report = smith_secure::audit(&config, "filesystem").unwrap();
    assert!(report.passed);
}

#[test]
fn secure_audit_nonexistent_server() {
    let config = fresh_config();
    let result = smith_secure::audit(&config, "nope");
    assert!(result.is_err());
}

#[test]
fn secure_audit_flags_sensitive_env() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    let server = config.servers.iter_mut().find(|s| s.name == "filesystem").unwrap();
    server.env.insert("API_KEY".to_string(), "secret123".to_string());
    let report = smith_secure::audit(&config, "filesystem").unwrap();
    assert!(report.findings.iter().any(|f| f.category == "env"));
}

#[test]
fn update_bumps_version() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    let updated = update(&mut config, None).unwrap();
    assert!(updated.contains(&"filesystem".to_string()));
}

#[test]
fn update_single_server() {
    let mut config = fresh_config();
    install(&mut config, "@modelcontextprotocol/filesystem", None).unwrap();
    install(&mut config, "valtors/cairn", None).unwrap();
    let updated = update(&mut config, Some("cairn")).unwrap();
    assert_eq!(updated, vec!["cairn"]);
}
