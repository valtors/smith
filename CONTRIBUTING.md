# Contributing to Smith

Thanks for your interest in contributing. Smith is a package manager for MCP servers, built with Rust.

## Ways to Contribute

- **Bug fixes** - Check issues labeled `bug`
- **Features** - Check issues labeled `enhancement` or `good first issue`
- **Install sources** - Add support for new package sources (homebrew, apt, etc.)
- **Security** - Improve the security probe and tool audit
- **Compose** - Enhance multi-server composition and routing
- **Registry** - Improve the GitHub-based registry and search
- **Docs** - Improve README, add examples, write guides
- **Tests** - Add test coverage across crates

## Setup

```bash
git clone https://github.com/valtors/smith.git
cd smith
cargo build
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test --workspace

# Install a server
cargo run -- install @modelcontextprotocol/filesystem
```

## AI Agent Contribution Guide

If you use AI tools to contribute, document which tools you used and which parts they generated. Keep human review in the loop.

## License

By contributing, you agree that your contributions will be licensed under the MIT license.
