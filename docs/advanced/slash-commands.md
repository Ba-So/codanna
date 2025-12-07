# Slash Commands

Codanna provides custom slash commands for Claude through the plugin system.

## Available via Plugin

Slash commands are now distributed as plugins. Install the core plugin to get access to intelligent code exploration workflows:

```bash
codanna plugin add https://github.com/bartolli/codanna-plugins.git codanna
```

## Included Commands

| Command | Description |
|---------|-------------|
| `/symbol <name>` | Find and analyze a symbol with complete context |
| `/x-ray <query>` | Deep semantic search with relationship mapping |

## How They Work

These commands use Codanna's MCP tools under the hood but provide guided workflows with comprehensive analysis and automatic report generation.

### `/symbol` Command

Find and analyze a specific symbol:
- Exact symbol lookup
- Complete context and documentation
- Relationship mapping
- Usage analysis

### `/x-ray` Command

Deep semantic search with full context:
- Natural language queries
- Semantic understanding of code
- Relationship tracking
- Impact analysis

## Creating Custom Commands

You can create your own slash commands as plugins. See [Plugin Documentation](../plugins/) for details on creating and distributing custom commands.

## See Also

- [Plugin System](../plugins/) - Installing and creating plugins
- [MCP Tools](../user-guide/mcp-tools.md) - Underlying tools used by commands
- [Agent Guidance](../integrations/agent-guidance.md) - How commands guide AI assistants