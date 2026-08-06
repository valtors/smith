use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIndexEntry {
    pub server: String,
    pub name: String,
    pub summary: String,
    pub schema_token_estimate: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIndex {
    pub entries: Vec<ToolIndexEntry>,
    pub total_full_schema_tokens: usize,
    pub index_token_cost: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedToolRef {
    pub server: String,
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub fn build_index(composed_tools: &[ComposedToolRef]) -> ToolIndex {
    let entries: Vec<ToolIndexEntry> = composed_tools
        .iter()
        .map(|t| {
            let summary: String = t.description.chars().take(80).collect();
            let schema_json = serde_json::to_string(&t.input_schema).unwrap_or_default();
            let schema_token_estimate = schema_json.len() / 4;

            ToolIndexEntry {
                server: t.server.clone(),
                name: t.name.clone(),
                summary,
                schema_token_estimate,
            }
        })
        .collect();

    let total_full_schema_tokens: usize = entries
        .iter()
        .map(|e| e.schema_token_estimate)
        .sum();

    let index_token_cost = entries
        .iter()
        .map(|e| (e.name.len() + e.summary.len() + 20) / 4)
        .sum();

    ToolIndex {
        entries,
        total_full_schema_tokens,
        index_token_cost,
    }
}

pub fn format_index_compact(index: &ToolIndex) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Tool Index ({} tools, ~{} tokens vs ~{} full load)\n\n",
        index.entries.len(),
        index.index_token_cost,
        index.total_full_schema_tokens
    ));
    for entry in &index.entries {
        out.push_str(&format!(
            "- {} [{}]: {}\n",
            entry.name, entry.server, entry.summary
        ));
    }
    out.push_str(
        "\nCall smith_get_tool_schema(name) to fetch full schema for a specific tool.\n",
    );
    out
}

pub fn find_tool<'a>(index: &'a ToolIndex, name: &str) -> Option<&'a ToolIndexEntry> {
    index.entries.iter().find(|e| e.name == name)
}

pub fn format_stats(index: &ToolIndex) -> String {
    let savings = index.total_full_schema_tokens.saturating_sub(index.index_token_cost);
    let pct = if index.total_full_schema_tokens > 0 {
        (savings as f64 / index.total_full_schema_tokens as f64 * 100.0) as usize
    } else {
        0
    };
    format!(
        "Tools: {}\nFull schema cost: ~{} tokens\nIndex cost: ~{} tokens\nSavings: ~{} tokens ({}%)\n",
        index.entries.len(),
        index.total_full_schema_tokens,
        index.index_token_cost,
        savings,
        pct
    )
}

pub fn get_schema_json(
    composed_tools: &[ComposedToolRef],
    name: &str,
) -> Option<serde_json::Value> {
    composed_tools
        .iter()
        .find(|t| t.name == name)
        .map(|t| t.input_schema.clone())
}
