# Codex CLI Integration

Codanna works with Codex CLI as a standard MCP server.

## Configuration

Configure in `~/.codex/config.toml`:

```toml
[mcp_servers.codanna]
command = "codanna"
args = ["serve", "--watch"]
startup_timeout_ms = 20_000
```

## Features

- Standard MCP server integration
- File watching capability
- Configurable startup timeout

## Verification

After configuration, verify the connection:

```bash
codanna mcp-test
```

## Usage

Once configured, Codex CLI will automatically start Codanna when needed and provide access to all MCP tools.

## Troubleshooting

- Ensure Codanna is in your PATH
- Check that `.codanna/settings.toml` exists in your project
- Adjust `startup_timeout_ms` if indexing takes longer on large codebases

## See Also

- [MCP Tools Reference](../user-guide/mcp-tools.md)
- [Configuration](../user-guide/configuration.md)