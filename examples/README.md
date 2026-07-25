# smith examples

## install servers

```bash
# from npm
smith install @modelcontextprotocol/filesystem
smith install @modelcontextprotocol/git
smith install @modelcontextprotocol/brave-search

# from git
smith install valtors/cairn
smith install valtors/observer

# from local path
smith install /home/user/my-mcp-server
```

## profiles

```bash
# create a work profile
smith profile create work
smith install @modelcontextprotocol/filesystem --profile work
smith install valtors/cairn --profile work

# create a personal profile
smith profile create personal
smith install @modelcontextprotocol/spotify --profile personal

# switch
smith profile work
smith profile personal
```

## compose

```bash
# start all servers in current profile, expose one endpoint
smith compose

# the agent sees one set of tools, not 15 separate servers
```

## security audit

```bash
# scan a server before activating it
smith secure filesystem

# output:
# [CRITICAL] prompt injection in tool description: "search"
# [HIGH] resource URI contains path traversal: "file:///etc/.."
# [MEDIUM] oversized description: 2500 chars
# risk score: 72/100 (HIGH)
# recommendation: do not activate
```

## claude desktop config

smith writes to `~/.smith/config.json`. point your agent at it:

```json
{
  "mcpServers": "smith compose --endpoint"
}
```

or use smith's generated claude config:

```bash
smith config --agent claude
# writes to ~/.claude/claude_desktop_config.json
```
