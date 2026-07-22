use serde::{Deserialize, Serialize};

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/valtors/smith-registry/main/registry.json";

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
        return Err("registry not available yet. the index lives at github.com/valtors/smith-registry".to_string());
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<RegistryEntry> = serde_json::from_str(&body)
        .map_err(|e| format!("failed to parse registry: {}", e))?;

    Ok(entries)
}

pub fn search<'a>(entries: &'a [RegistryEntry], query: &str) -> Vec<&'a RegistryEntry> {
    let lower = query.to_lowercase();
    entries.iter()
        .filter(|e| {
            e.name.to_lowercase().contains(&lower)
                || e.description.to_lowercase().contains(&lower)
                || e.category.to_lowercase().contains(&lower)
        })
        .collect()
}

pub fn format_entry(entry: &RegistryEntry) -> String {
    let verified = if entry.verified { " [verified]" } else { "" };
    format!("{}{} - {} ({})", entry.name, verified, entry.description, entry.category)
}
