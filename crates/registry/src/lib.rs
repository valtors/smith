//! github-based registry. anyone can publish. no walled garden.
//!
//! the index lives at github.com/valtors/smith-registry as a json file.
//! smith fetches it, searches by name/description/category, and prints
//! results. no API key, no auth, no rate limits. just a json file on
//! a public repo.

use serde::{Deserialize, Serialize};

const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/valtors/smith-registry/main/registry.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub source: String,
    pub description: String,
    pub category: String,
    pub verified: bool,
}

pub fn fetch_registry() -> Result<Vec<RegistryEntry>, String> {
    let output = std::process::Command::new("curl")
        .arg("-s")
        .arg("--fail")
        .arg(REGISTRY_URL)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(
            "registry not available yet. the index lives at github.com/valtors/smith-registry"
                .to_string(),
        );
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<RegistryEntry> =
        serde_json::from_str(&body).map_err(|e| format!("failed to parse registry: {}", e))?;

    Ok(entries)
}

pub fn search<'a>(entries: &'a [RegistryEntry], query: &str) -> Vec<&'a RegistryEntry> {
    let lower = query.to_lowercase();
    entries
        .iter()
        .filter(|e| {
            e.name.to_lowercase().contains(&lower)
                || e.description.to_lowercase().contains(&lower)
                || e.category.to_lowercase().contains(&lower)
        })
        .collect()
}

pub fn format_entry(entry: &RegistryEntry) -> String {
    let verified = if entry.verified { " [verified]" } else { "" };
    format!(
        "{}{} - {} ({})",
        entry.name, verified, entry.description, entry.category
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entries() -> Vec<RegistryEntry> {
        vec![
            RegistryEntry {
                name: "github-mcp".to_string(),
                source: "github.com/valtors/github-mcp".to_string(),
                description: "GitHub MCP server".to_string(),
                category: "devtools".to_string(),
                verified: true,
            },
            RegistryEntry {
                name: "filesystem-mcp".to_string(),
                source: "github.com/valtors/fs-mcp".to_string(),
                description: "File system access".to_string(),
                category: "tools".to_string(),
                verified: false,
            },
            RegistryEntry {
                name: "postgres-mcp".to_string(),
                source: "github.com/valtors/pg-mcp".to_string(),
                description: "PostgreSQL query tool".to_string(),
                category: "database".to_string(),
                verified: true,
            },
        ]
    }

    #[test]
    fn test_search_by_name() {
        let entries = test_entries();
        let results = search(&entries, "github");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "github-mcp");
    }

    #[test]
    fn test_search_by_description() {
        let entries = test_entries();
        let results = search(&entries, "file system");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "filesystem-mcp");
    }

    #[test]
    fn test_search_by_category() {
        let entries = test_entries();
        let results = search(&entries, "database");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "postgres-mcp");
    }

    #[test]
    fn test_search_case_insensitive() {
        let entries = test_entries();
        let results = search(&entries, "GITHUB");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_no_match() {
        let entries = test_entries();
        let results = search(&entries, "nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_empty_query() {
        let entries = test_entries();
        let results = search(&entries, "");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_format_entry_verified() {
        let entry = &test_entries()[0];
        let formatted = format_entry(entry);
        assert!(formatted.contains("[verified]"));
        assert!(formatted.contains("github-mcp"));
        assert!(formatted.contains("GitHub MCP server"));
        assert!(formatted.contains("devtools"));
    }

    #[test]
    fn test_format_entry_unverified() {
        let entry = &test_entries()[1];
        let formatted = format_entry(entry);
        assert!(!formatted.contains("[verified]"));
        assert!(formatted.contains("filesystem-mcp"));
    }

    #[test]
    fn test_registry_entry_serialization() {
        let entry = test_entries()[0].clone();
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: RegistryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "github-mcp");
        assert!(deserialized.verified);
    }
}
