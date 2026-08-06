# smith

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

## usage

```bash
# install
smith install @modelcontextprotocol/filesystem
smith install valtors/cairn
smith install github.com/someuser/some-server

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

## license

MIT. strictly open source. no cloud tier, no enterprise plan, no proprietary fork.
