# smith

[![CI](https://github.com/valtors/smith/actions/workflows/ci.yml/badge.svg)](https://github.com/valtors/smith/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-0F172A?style=flat-square)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-CE422B?style=flat-square)](https://www.rust-lang.org/)
[![tests](https://img.shields.io/badge/tests-74-green?style=flat-square)]()
[![crates.io](https://img.shields.io/crates/v/smith-mcp?style=flat-square&label=crates.io)](https://crates.io/crates/smith-mcp)

npm for MCP. install, compose, secure, and manage MCP servers. one binary.

## what

smith is the package manager for MCP servers. MCP gave agents tools. smith makes them installable.

no more manually editing json config files. no more copy-pasting server commands from markdown lists. one command installs, configures, and wires any MCP server into any agent.

```bash
smith install @modelcontextprotocol/filesystem
smith install valtors/cairn
smith list
smith update
smith remove filesystem
smith compose
```

## why

every single person using MCP hits the same wall: "how do i install an MCP server?" there is no good answer. you copy-paste from a markdown list into your agent config. hope the path is right. hope the version works. hope it doesn't break when something updates.

npm made javascript installable. cargo made rust installable. smith makes MCP servers installable.

| | manual config | mcp-cli | smith |
|---|---|---|---|
| install from npm | no | yes | yes |
| install from git | no | no | yes |
| install from local | no | no | yes |
| security audit | no | no | yes |
| compose multiple servers | no | no | yes |
| profiles | no | no | yes |
| update all | manual | no | one command |
| config format | json by hand | json | json, managed |

## how it works

smith manages a single config file (`~/.smith/config.json`) that any MCP-compatible agent can read. it handles:

- **install.** `smith install <name>` fetches the server, verifies it, adds it to your config. supports git repos, npm packages, and local paths.
- **compose.** `smith compose` starts all your installed servers and exposes one unified MCP endpoint. the agent sees one set of tools, not 15 separate configs.
- **secure.** `smith secure <name>` runs a security probe on a server before activation. checks for dangerous tool patterns, excessive permissions, data exfiltration. (powered by mcprobe concepts.)
- **profiles.** `smith profile work` switches your active server set. `smith profile personal` switches back. one command.
- **update.** `smith update` pulls latest versions of all installed servers. `smith update <name>` updates one.
- **registry.** a github-based index. anyone can publish. `smith publish` from any git repo. no walled garden.

## architecture

```
smith/
  crates/
    config/       read/write smith config, agent configs
    install/      fetch from npm/git/local, verify, activate
    compose/      spawn servers, route tool calls, unified endpoint
    secure/       security probe, tool audit, permission check
    profile/      named server sets, switching
    registry/     github-based index, search, publish
  bin/
    smith/        CLI entry point
```

one binary. reads/writes json config. spawns and manages server processes.

## install

```bash
cargo install smith-mcp
```

or build from source:

```bash
git clone https://github.com/valtors/smith
cd smith
cargo build --release
cp target/release/smith /usr/local/bin/
```

## usage

```bash
# install from npm
smith install @modelcontextprotocol/filesystem

# install from git
smith install valtors/cairn

# install from local path
smith install /path/to/my/server

# see what's installed
smith list

# start all servers, expose unified endpoint
smith compose

# security check before activating
smith secure filesystem

# switch profiles
smith profile work
smith profile personal

# update everything
smith update

# remove
smith remove filesystem

# publish your own server to the registry
smith publish
```

## tests

74 tests across 6 crates. all pass.

```bash
cargo test --workspace
```

## contributing

see [CONTRIBUTING.md](CONTRIBUTING.md). we welcome contributions of all kinds - bug fixes, new patterns, transport support, docs.

good first issues are labeled `good first issue`.

## license

MIT. strictly open source. no cloud tier, no enterprise plan, no proprietary fork.
