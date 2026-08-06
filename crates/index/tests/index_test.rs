use smith_index::{build_index, format_index_compact, format_stats, get_schema_json, ComposedToolRef};

#[test]
fn test_build_index_basic() {
    let tools = vec![
        ComposedToolRef {
            server: "filesystem".to_string(),
            name: "read_file".to_string(),
            description: "Read the contents of a file from the filesystem".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to the file"}
                },
                "required": ["path"]
            }),
        },
        ComposedToolRef {
            server: "github".to_string(),
            name: "create_issue".to_string(),
            description: "Create a new issue in a GitHub repository with title and body".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "owner": {"type": "string"},
                    "repo": {"type": "string"},
                    "title": {"type": "string"},
                    "body": {"type": "string"},
                    "labels": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["owner", "repo", "title"]
            }),
        },
    ];

    let index = build_index(&tools);
    assert_eq!(index.entries.len(), 2);
    assert_eq!(index.entries[0].name, "read_file");
    assert_eq!(index.entries[0].server, "filesystem");
    assert_eq!(index.entries[1].name, "create_issue");
    assert_eq!(index.entries[1].server, "github");
    assert!(index.total_full_schema_tokens > 0);
    assert!(index.index_token_cost < index.total_full_schema_tokens);
}

#[test]
fn test_format_index_compact() {
    let tools = vec![ComposedToolRef {
        server: "fs".to_string(),
        name: "read_file".to_string(),
        description: "Read file contents".to_string(),
        input_schema: serde_json::json!({"type": "object"}),
    }];
    let index = build_index(&tools);
    let output = format_index_compact(&index);
    assert!(output.contains("Tool Index"));
    assert!(output.contains("read_file"));
    assert!(output.contains("smith_get_tool_schema"));
}

#[test]
fn test_format_stats_shows_savings() {
    let tools = vec![
        ComposedToolRef {
            server: "s1".to_string(),
            name: "tool_a".to_string(),
            description: "Does thing A".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "x": {"type": "string", "description": "param x"},
                    "y": {"type": "string", "description": "param y"},
                    "z": {"type": "string", "description": "param z"}
                }
            }),
        },
        ComposedToolRef {
            server: "s2".to_string(),
            name: "tool_b".to_string(),
            description: "Does thing B".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "string", "description": "param a"},
                    "b": {"type": "string", "description": "param b"}
                }
            }),
        },
    ];
    let index = build_index(&tools);
    let stats = format_stats(&index);
    assert!(stats.contains("Savings"));
    assert!(stats.contains("%"));
}

#[test]
fn test_get_schema_json() {
    let tools = vec![ComposedToolRef {
        server: "fs".to_string(),
        name: "read_file".to_string(),
        description: "Read file".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"}
            }
        }),
    }];
    let schema = get_schema_json(&tools, "read_file");
    assert!(schema.is_some());
    assert_eq!(
        schema.unwrap()["type"],
        serde_json::json!("object")
    );
    let missing = get_schema_json(&tools, "nonexistent");
    assert!(missing.is_none());
}

#[test]
fn test_summary_truncation() {
    let long_desc = "This is a very long description that should be truncated to exactly eighty characters and not more".to_string();
    let tools = vec![ComposedToolRef {
        server: "s".to_string(),
        name: "t".to_string(),
        description: long_desc,
        input_schema: serde_json::json!({}),
    }];
    let index = build_index(&tools);
    assert!(index.entries[0].summary.len() <= 80);
}
