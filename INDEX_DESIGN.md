# Smith Index-Based Tool Loading

## Problem

MCP sends full JSON schema for every enabled tool on every API call. For small / open-weight models with 8K context, this means 20K+ tokens of tool definitions before the user says anything. The model spends half its context window understanding what tools exist.

Evidence:
- u/Anoop_kathait: 20K+ tokens burned on a single "hi" (r/AI_Agents)
- MCP discussions: issues at 80-90 tools
- pristren blog: 50K+ tokens per server
- r/LocalLLaMA: "MCP == context bloat. The more crap you load up for your agent the dumber and more expensive it is."

## Solution

Index-based tool loading. Smith generates a compact tool index (name + one-line description) and exposes it to the model. Full schemas are fetched on demand only when the model decides to call a specific tool.

## How It Works

```
Traditional MCP:
  Model <-- [50 tools x full JSON schema] --> MCP Server
  Cost: 20K-50K tokens upfront, every call

Smith Index Mode:
  1. Smith probes all active MCP servers (already does this)
  2. Smith builds compact index: [{name, summary}] per tool
  3. Model gets index only (200-500 tokens for 50 tools)
  4. Model picks a tool by name
  5. Smith fetches full schema for that one tool
  6. Model calls tool with full schema in context
  Cost: 500 tokens upfront + 200-500 tokens per actual call
```

## Implementation

### 1. New crate: `smith-index`

Generates and serves the compact tool index.

```rust
// crates/index/src/lib.rs

pub struct ToolIndexEntry {
    pub server: String,       // which MCP server owns this
    pub name: String,         // tool name
    pub summary: String,      // first 80 chars of description, trimmed
    pub schema_token_estimate: usize,  // rough token count of full schema
}

pub struct ToolIndex {
    pub entries: Vec<ToolIndexEntry>,
    pub total_full_schema_tokens: usize,  // what it WOULD cost to load everything
    pub index_token_cost: usize,          // what the index itself costs
}

pub fn build_index(composed_tools: &[ComposedTool]) -> ToolIndex {
    let entries: Vec<ToolIndexEntry> = composed_tools
        .iter()
        .map(|t| {
            let summary = t.description.chars().take(80).collect::<String>();
            let schema_json = serde_json::to_string(&t.input_schema).unwrap_or_default();
            let schema_token_estimate = schema_json.len() / 4; // rough: 4 chars per token

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
        .map(|e| e.name.len() + e.summary.len() + 20) // name + summary + overhead
        .sum::<usize>() / 4;

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
    out.push_str(&format!(
        "\nCall smith_get_tool_schema(name) to fetch full schema for a specific tool.\n"
    ));
    out
}

pub fn find_tool<'a>(index: &'a ToolIndex, name: &str) -> Option<&'a ToolIndexEntry> {
    index.entries.iter().find(|e| e.name == name)
}
```

### 2. New CLI command: `smith index`

```bash
# Generate and print the compact tool index
smith index

# Show token savings
smith index --stats

# Export as JSON for programmatic use
smith index --json

# Get full schema for one tool (for on-demand loading)
smith index --get <tool_name>
```

### 3. New compose mode: `--index-only`

When running `smith compose`, add a flag to emit only the index instead of full schemas:

```bash
smith compose --index-only > tool-index.md
```

This output goes into the model's system prompt as a lightweight tool menu. When the model wants to call a tool, it first calls `smith_get_tool_schema` to get the full schema, then calls the actual tool.

### 4. MCP tool: `smith_get_tool_schema`

Smith's compose server exposes one meta-tool:

```json
{
  "name": "smith_get_tool_schema",
  "description": "Fetch full input schema for a specific tool by name. Use this after reviewing the tool index to get calling details.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "Tool name from the index"
      }
    },
    "required": ["name"]
  }
}
```

This is the only tool schema sent upfront. It costs ~50 tokens. The model uses it to pull full schemas on demand.

## Token Budget Comparison

| Scenario | 50 tools, traditional MCP | 50 tools, Smith index |
|----------|--------------------------|-----------------------|
| Upfront cost | 20K-50K tokens | 500 tokens (index) + 50 tokens (meta-tool) |
| Per-call cost | 0 (already loaded) | 200-500 tokens (one schema fetch) |
| 10 calls in one session | 20K-50K | 500 + 10x300 = 3.5K |
| 8K context model | Overflows | Fits with room for actual work |

## What Changes in Smith

1. New crate `crates/index/` with `ToolIndex`, `build_index`, `format_index_compact`, `find_tool`
2. New CLI subcommand `smith index` with `--stats`, `--json`, `--get <name>` flags
3. `smith compose --index-only` flag for compose mode
4. `smith_get_tool_schema` meta-tool in compose server
5. `smith index --watch` mode that rebuilds index when servers change

## What Changes in mcprobe

mcprobe already probes MCP servers and stores `Tool` structs with full `InputSchema`. Add:
- `mcprobe --token-cost` flag: estimates token cost of all tool schemas
- `mcprobe --index-only` flag: outputs compact index instead of full snapshot
- Token cost per tool in security report (warns when total exceeds threshold)

## What Changes in Cairn

Cairn stores memory. With index mode:
- Cairn can cache tool schemas fetched via `smith_get_tool_schema`
- Avoids re-fetching the same schema across calls
- Cache keyed by `server_name + tool_name + schema_hash`
