# Architecture

## Overview

Smith is a package manager for MCP servers. Install from npm, git, or local paths. Compose multiple servers into one unified tool set. Audit servers for security risks before activation. Manage server sets with profiles. One config file, one CLI, one MCP endpoint.

```
  +--------+   +--------+   +--------+   +--------+
  |  npm   |   |  git   |   | local  |   | registry|
  | source |   | source |   | path   |   | search  |
  +---+----+   +---+----+   +---+----+   +---+----+
      |            |            |            |
      +-----+------+-----+------+
            |                     |
       +----v----+          +-----v-----+
       | install |          |  search   |
       | (resolve)|         |  (filter) |
       +----+----+          +-----------+
            |
       +----v----+
       | config  |
       | (json)  |
       +----+----+
            |
  +---------+---------+---------+
  |         |         |         |
  v         v         v         v
+--+--+  +--+--+  +--+--+  +---+---+
|compose| |secure| |profile| |update|
|(spawn)| (audit)|(switch)| (fetch)|
+--+--+  +--+--+  +-------+ +-------+
   |         |
   v         v
+--+---------+--+
|  MCP endpoint |
|  (one pipe)   |
+---------------+
```

## Design Principles

1. **npm for MCP.** MCP servers should be as easy to install as npm packages. `smith install @modelcontextprotocol/filesystem` and it is ready. No manual config editing.
2. **One config file.** `~/.smith/config.json` is the source of truth. Any MCP-compatible agent can read it. No database, no registry service, no lock-in.
3. **Compose, do not configure.** `smith compose` spawns all active servers, probes each for its tool list, and merges them with a `[servername]` prefix. The agent sees one set of tools, not 15 separate configs.
4. **Audit before activate.** `smith secure` runs static analysis on a server before it touches your agent. Checks for dangerous env vars, excessive permissions, and suspicious tool names. Safety first.
5. **Profiles for context switching.** Work tools do not leak into personal agent sessions. `smith profile work` switches the active server set in one command.
6. **GitHub-based registry.** The index lives at `github.com/valtors/smith-registry` as a JSON file. Anyone can publish. No walled garden, no API key, no auth.

## Components

### config (`crates/config`)

Reads and writes `~/.smith/config.json`.

```json
{
  "servers": [
    {
      "name": "filesystem",
      "source": "@modelcontextprotocol/filesystem",
      "version": "latest",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/filesystem"],
      "env": {},
      "enabled": true,
      "profile": "default"
    }
  ],
  "active_profile": "default",
  "smith_dir": "~/.smith"
}
```

**Source resolution:** `parse_source(source)` detects:
- `@scope/pkg` or `pkg-name` -> `SourceType::Npm`
- `github.com/user/repo` -> `SourceType::GitRepo`
- `https://github.com/...` or `git+https://...` -> `SourceType::Git`
- `./path` or `/abs/path` -> `SourceType::Local`

### install (`crates/install`)

Resolves a source string into a command + args that the compose layer can spawn.

| Source type | Command | Args |
|---|---|---|
| npm `@scope/pkg` | `npx` | `["-y", "@scope/pkg"]` |
| git `github.com/user/repo` | `npx` | `["-y", "github:user/repo"]` |
| git `https://...` | `npx` | `["-y", "git+https://..."]` |
| local `./path` | (absolute path) | `[]` |

Install writes to config. It does not download anything -- `npx` handles that at spawn time. The install is just config writing.

### compose (`crates/compose`)

Spawns all active servers and merges their tool lists.

**Spawn:** `spawn_all(config)` starts each active server as a child process with stdin/stdout/stderr pipes.

**Probe:** `list_all_tools(config)` sends `initialize` + `tools/list` to each server, collects all tools, prefixes tool names with `[servername]` to prevent conflicts.

**Route:** When the agent calls a tool, compose routes the JSON-RPC request to the correct server process based on the prefix. One pipe in, many pipes out.

### secure (`crates/secure`)

Static security audit. Runs before activation. Checks:

| Check | Severity | Trigger |
|---|---|---|
| Sensitive env vars | info | Env var name contains key/secret/token/password |
| Missing package name | warn | `npx` without explicit package |
| Dangerous command patterns | warn | Command includes shell pipe, eval, or exec |
| Excessive env vars | warn | More than 10 env vars (possible credential sprawl) |
| Shell execution | danger | Command is `sh -c` or `bash -c` |

**Risk levels:** Safe (no findings), Caution (info/warn findings), Dangerous (danger findings).

**Output:** `SecurityReport` with server name, risk level, findings list, and passed flag.

### profile (`crates/profile`)

Named server sets for context switching.

- `switch(config, "work")` -- sets `active_profile`. Only servers with matching profile are active.
- `list(config)` -- returns all profile names found in the server list.
- `current(config)` -- returns the active profile name.
- `assign(config, "server", "profile")` -- reassigns a server to a different profile.

### registry (`crates/registry`)

GitHub-based package index. Fetches `registry.json` from `github.com/valtors/smith-registry`.

**Registry entry:**
```json
{
  "name": "filesystem",
  "source": "@modelcontextprotocol/filesystem",
  "description": "File system access for MCP agents",
  "category": "devtools",
  "verified": true
}
```

- `fetch_registry()` -- curl the JSON file from GitHub raw URL.
- `search(entries, query)` -- case-insensitive match on name, description, or category.
- `format_entry(entry)` -- pretty-print with verified badge.

No API key, no auth, no rate limits. Just a JSON file on a public repo.

### update (`crates/install`)

`update(name)` re-resolves a server's source and updates the command/args in config. If name is omitted, updates all installed servers.

## CLI Commands

| Command | Purpose |
|---|---|
| `smith install <source> [--profile X]` | Install an MCP server |
| `smith remove <name>` | Uninstall a server |
| `smith list` | List all installed servers |
| `smith update [name]` | Update one or all servers |
| `smith compose` | Spawn all active servers as composed MCP endpoint |
| `smith secure [name]` | Audit one or all servers |
| `smith profile list` | List all profiles |
| `smith profile current` | Show active profile |
| `smith profile switch <name>` | Switch active profile |
| `smith profile assign <server> <profile>` | Reassign a server |
| `smith search <query>` | Search the registry |

## File Layout

```
~/.smith/
  config.json          # Server list + active profile
```

## Testing

74 tests across all crates. Integration tests in `tests/integration.rs` exercise the full pipeline: install, list, compose, secure, profile switch, search.

## Dependencies

- `clap` -- CLI parsing
- `serde` / `serde_json` -- serialization
- `std::process::Command` -- spawning MCP servers
- `curl` (system binary) -- fetching registry

## Published Crates

All published to crates.io:
- `smith-mcp` (binary: `smith`) -- main CLI
- `smith-mcp-config` -- config management
- `smith-mcp-install` -- source resolution
- `smith-mcp-compose` -- server composition
- `smith-mcp-secure` -- security audit
- `smith-mcp-profile` -- profile management
- `smith-mcp-registry` -- registry search

Install: `cargo install smith-mcp`
