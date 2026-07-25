//! compose. spawn all installed servers. expose one unified endpoint.
//!
//! the agent sees one set of tools, not 15 separate configs. smith probes
//! each server for its tool list, merges them with a `[servername]` prefix,
//! and routes tool calls to the right process. one pipe in, many pipes out.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use smith_mcp_config::{ServerEntry, SmithConfig};
use std::io::{self, BufRead, Write};
use std::process::{Child, Command, Stdio};

pub struct ComposedServer {
    pub name: String,
    pub child: Child,
}

pub fn spawn_all(config: &SmithConfig) -> Result<Vec<ComposedServer>, String> {
    let active = config.active_servers();
    let mut servers = Vec::new();

    for entry in active {
        let child = Command::new(&entry.command)
            .args(&entry.args)
            .envs(entry.env.iter())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn {}: {}", entry.name, e))?;
        servers.push(ComposedServer {
            name: entry.name.clone(),
            child,
        });
    }

    Ok(servers)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedTool {
    pub server: String,
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub fn list_all_tools(config: &SmithConfig) -> Vec<ComposedTool> {
    let active = config.active_servers();
    let mut tools = Vec::new();

    for entry in active {
        if let Ok(response) = probe_server(entry) {
            if let Some(tool_list) = response
                .get("result")
                .and_then(|r| r.get("tools"))
                .and_then(|t| t.as_array())
            {
                for tool in tool_list {
                    tools.push(ComposedTool {
                        server: entry.name.clone(),
                        name: tool
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        description: tool
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        input_schema: tool.get("inputSchema").cloned().unwrap_or(json!({})),
                    });
                }
            }
        }
    }

    tools
}

fn probe_server(entry: &ServerEntry) -> Result<Value, String> {
    let mut child = Command::new(&entry.command)
        .args(&entry.args)
        .envs(entry.env.iter())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;

    let init = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "smith", "version": "0.1.0"}
        }
    });

    let list = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    if let Some(mut stdin) = child.stdin.take() {
        let _ = writeln!(stdin, "{}", init);
        let _ = writeln!(stdin, "{}", list);
    }

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if let Ok(val) = serde_json::from_str::<Value>(line) {
            if val.get("id").and_then(|v| v.as_i64()) == Some(2) {
                return Ok(val);
            }
        }
    }

    Err("no tools/list response".to_string())
}

pub fn route_tool_call(
    config: &SmithConfig,
    tool_name: &str,
    args: &Value,
) -> Result<Value, String> {
    let tools = list_all_tools(config);
    let tool = tools
        .iter()
        .find(|t| t.name == tool_name)
        .ok_or(format!("tool not found: {}", tool_name))?;

    let entry = config
        .get_server(&tool.server)
        .ok_or(format!("server not found: {}", tool.server))?;

    let mut child = Command::new(&entry.command)
        .args(&entry.args)
        .envs(entry.env.iter())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;

    let init = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "smith", "version": "0.1.0"}
        }
    });

    let call = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": args
        }
    });

    if let Some(mut stdin) = child.stdin.take() {
        let _ = writeln!(stdin, "{}", init);
        let _ = writeln!(stdin, "{}", call);
    }

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if let Ok(val) = serde_json::from_str::<Value>(line) {
            if val.get("id").and_then(|v| v.as_i64()) == Some(2) {
                return Ok(val);
            }
        }
    }

    Err("no tool/call response".to_string())
}

pub fn run_compose_server(config: &SmithConfig) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                let _ = writeln!(stdout, "{}", json!({"error": "invalid json"}));
                continue;
            }
        };

        let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let id = request.get("id").cloned().unwrap_or(json!(null));

        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "smith", "version": "0.1.0" }
                }
            }),
            "tools/list" => {
                let tools = list_all_tools(config);
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools.iter().map(|t| json!({
                            "name": t.name,
                            "description": format!("[{}] {}", t.server, t.description),
                            "inputSchema": t.input_schema
                        })).collect::<Vec<_>>()
                    }
                })
            }
            "tools/call" => {
                let tool_name = request
                    .get("params")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let args = request
                    .get("params")
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or(json!({}));

                match route_tool_call(config, tool_name, &args) {
                    Ok(result) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result.get("result").cloned().unwrap_or(json!({}))
                    }),
                    Err(e) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32603, "message": e }
                    }),
                }
            }
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("unknown method: {}", method) }
            }),
        };

        let _ = writeln!(stdout, "{}", response);
        let _ = stdout.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use smith_mcp_config::{ServerEntry, SmithConfig};

    #[test]
    fn test_composed_tool_serialization() {
        let tool = ComposedTool {
            server: "github".to_string(),
            name: "create_issue".to_string(),
            description: "Create a GitHub issue".to_string(),
            input_schema: json!({"type": "object"}),
        };
        let serialized = serde_json::to_string(&tool).unwrap();
        assert!(serialized.contains("github"));
        assert!(serialized.contains("create_issue"));
        let deserialized: ComposedTool = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.server, "github");
        assert_eq!(deserialized.name, "create_issue");
    }

    #[test]
    fn test_spawn_all_empty_config() {
        let config = SmithConfig::default();
        let result = spawn_all(&config);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_spawn_all_disabled_servers() {
        let mut config = SmithConfig::default();
        config.servers.push(ServerEntry {
            name: "test".to_string(),
            source: "npm".to_string(),
            version: "1.0.0".to_string(),
            command: "nonexistent-command-12345".to_string(),
            args: vec![],
            env: std::collections::HashMap::new(),
            enabled: false,
            profile: "default".to_string(),
        });
        let result = spawn_all(&config);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_list_all_tools_empty() {
        let config = SmithConfig::default();
        let tools = list_all_tools(&config);
        assert!(tools.is_empty());
    }

    #[test]
    fn test_compose_tool_fields() {
        let tool = ComposedTool {
            server: "fs".to_string(),
            name: "read_file".to_string(),
            description: "Read a file".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        };
        assert_eq!(tool.server, "fs");
        assert_eq!(tool.name, "read_file");
        assert_eq!(tool.description, "Read a file");
    }
}
