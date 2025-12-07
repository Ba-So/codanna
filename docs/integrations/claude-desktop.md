# Claude Desktop Integration

Configure Codanna with Claude Desktop application.

## Configuration

For Claude Desktop, you need the `--config` flag since it runs from a different location.

Configure in `~/Library/Application Support/Claude/claude_desktop_config.json` (Mac):

```json
{
  "mcpServers": {
    "codanna": {
      "command": "codanna",
      "args": ["--config", "/absolute/path/to/your/project/.codanna/settings.toml", "serve", "--watch"]
    }
  }
}
```

Replace `/absolute/path/to/your/project/` with your actual project path.

## Why --config Flag?

Claude Desktop runs from a different working directory than your project, so it needs the absolute path to your project's settings file.

## Features

- Same capabilities as Claude Code
- File watching with `--watch`
- stdio transport

## Verification

After configuration:
1. Restart Claude Desktop
2. In your project directory, run:
   ```bash
   codanna mcp-test
   ```

## Multiple Projects

To work with multiple projects, you can:
1. Use different config files for each project
2. Update the path in claude_desktop_config.json when switching projects

## Troubleshooting

- Use absolute paths, not relative
- Ensure `.codanna/settings.toml` exists at the specified path
- Check that Codanna is in your system PATH

## See Also

- [MCP Tools Reference](../user-guide/mcp-tools.md)
- [Configuration](../user-guide/configuration.md)
- [Claude Code Integration](claude-code.md)