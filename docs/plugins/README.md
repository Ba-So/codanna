[Documentation](../README.md) / **Plugins**

---

# Plugins

Codanna plugins are project-scoped. They install to `.claude/` in your project directory, not globally. This lets each project have different plugin versions.

## codanna-cc Plugin

Available via Claude Code's `/plugin` command or codanna's CLI.

**Via Claude Code:**
```bash
# Add the Codanna marketplace
/plugin marketplace add bartolli/codanna-plugins

# Install the plugin
/plugin install codanna-cc@codanna-plugins
```

**Via Codanna CLI:**
```bash
codanna plugin add https://github.com/bartolli/codanna-plugins.git codanna
codanna plugin add https://github.com/bartolli/codanna-plugins.git codanna --ref v1.2.0  # Specific version
```

The CLI method gives you version control - install different tags per project.

### Token-Efficient Workflows

The plugin includes Node.js scripts that parse JSON output to save tokens. See [codanna-plugins](https://github.com/bartolli/codanna-plugins) for examples.

**Example: Piping with Node.js wrapper**
```bash
# Node script handles JSON parsing and formatting
node .claude/scripts/codanna/context-provider.js find "error handling" --limit=3

# Output includes symbol_id for follow-up queries
# 1. IndexError (Enum) [symbol_id:205]
#    Use: node .claude/scripts/codanna/context-provider.js calls symbol_id:205
```

This approach reduces token usage by pre-processing results before presenting to the AI assistant.

## Quick Start

**Install our core plugin**

```bash
codanna plugin add https://github.com/bartolli/codanna-plugins.git codanna
```

**Update a plugin**

```bash
codanna plugin update my-plugin
```

**Remove a plugin**

```bash
codanna plugin remove my-plugin
```

**List installed plugins**

```bash
codanna plugin list --verbose
```

**Verify plugin integrity**

```bash
codanna plugin verify my-plugin
```

**Adding Plugins**

When you add a plugin, codanna:

1. Clones the marketplace repository to a temporary directory
2. Validates the plugin manifest (.claude-plugin/plugin.json)
3. Checks for file conflicts with existing plugins
4. Copies component files to namespaced directories:
   - Commands → .claude/commands/<plugin>/
   - Agents → .claude/agents/<plugin>/
   - Hooks → .claude/hooks/<plugin>/
   - Scripts → .claude/scripts/<plugin>/
   - Other files → .claude/plugins/<plugin>/

5. Merges MCP server configuration into .mcp.json
6. Calculates integrity checksum (SHA-256) of all installed files
7. Updates the lockfile (.codanna/plugins/lockfile.json)

## Advanced Options

**Install specific Git ref (branch/tag/commit)**

```bash
codanna plugin add https://github.com/user/marketplace.git my-plugin --ref v1.2.0
```

**Force installation (overwrite conflicts)**

```bash
codanna plugin add https://github.com/user/marketplace.git my-plugin --force
```

**Preview changes without installing**

```bash
codanna plugin add https://github.com/user/marketplace.git my-plugin --dry-run
```

Rollback Protection: If any step fails, codanna automatically:
- Removes partially copied files
- Restores previous plugin version (during updates)
- Restores MCP configuration
- Cleans up directories

**Updating Plugins**

Updates detect changes via Git commit SHA comparison:

**Update to latest commit**

```bash
codanna plugin update my-plugin
```

**Update to specific ref**

```bash
codanna plugin update my-plugin --ref main
```

**Force reinstall (bypass commit check)**

```bash
codanna plugin update my-plugin --force
```

**Update Process:**

1. Resolves remote commit SHA from Git repository
2. Compares with installed commit:

- Same commit + passes verification → "Already up to date"
- Same commit + fails verification → Reinstall
- Different commit → Update

3. Backs up existing plugin before changes
4. Uninstalls old version completely
5. Installs new version with new files
6. Rolls back to backup if installation fails

## Removing Plugins

**Safe removal with complete cleanup:**

```bash
codanna plugin remove my-plugin
```

**Force removal (skip safety checks)**

```bash
codanna plugin remove my-plugin --force
```

**Preview removal**

```bash
codanna plugin remove my-plugin --dry-run
```

Cleanup Actions:

1. Removes all tracked files from filesystem
2. Removes MCP server entries from .mcp.json
3. Cleans up plugin directories (.claude/plugins/<plugin>/, etc.)
4. Updates lockfile to remove plugin entry

Plugin Storage Structure

```text
.claude/
├── commands/<plugin>/ # Slash commands
├── agents/<plugin>/ # Custom agents
├── hooks/<plugin>/ # Event hooks
├── scripts/<plugin>/ # Utility scripts
└── plugins/<plugin>/ # Additional payload files

.codanna/
└── plugins/
└── lockfile.json # Installation tracking with integrity checksums
```

**Lockfile Structure**

The lockfile (.codanna/plugins/lockfile.json) tracks all installed plugins:

```json
{
  "version": "1.0.0",
  "plugins": {
    "my-plugin": {
      "name": "my-plugin",
      "version": "1.0.0",
      "commit": "abc123def456...",
      "marketplace_url": "https://github.com/user/marketplace.git",
      "installed_at": "2025-10-17T13:58:03Z",
      "updated_at": "2025-10-17T14:00:00Z",
      "integrity": "sha256:...",
      "files": [".claude/commands/my-plugin/command.md"],
      "mcp_keys": ["my-plugin-server"],
      "source": {
        "type": "marketplace_path",
        "relative": "plugins/my-plugin"
      }
    }
  }
}
```

## MCP Server Integration

Plugins can provide MCP servers that get merged into your project's .mcp.json:

Before Installation:

```json
{
  "mcpServers": {
    "existing-server": { "command": "cmd" }
  }
}
```

After Installing Plugin with MCP Server:

```json
{
  "mcpServers": {
    "existing-server": { "command": "cmd" },
    "my-plugin-server": {
      "command": "node",
      "args": ["server.js"]
    }
  }
}
```

Conflict Handling: If an MCP server key already exists, installation fails unless you use --force (which overwrites the existing
entry).

## Verification and Integrity

Verify plugin integrity at any time:

### Verify specific plugin

```bash
codanna plugin verify my-plugin --verbose
```

Verification Checks:

1. All tracked files exist on filesystem
2. File contents match SHA-256 integrity checksum
3. MCP server keys present in .mcp.json

Failed verification indicates tampering or corruption. Reinstall with --force to fix.

## Listing Plugins

### Basic list

```bash
codanna plugin list
```

### Verbose output with details

```bash
codanna plugin list --verbose
```

### JSON output for scripting

```bash
codanna plugin list --json
```

Verbose Output Shows:

- Plugin name and version
- Installation and update timestamps
- Git commit SHA
- Marketplace URL
- Number of installed files
- MCP server keys

Command Reference

| Command                                   | Description                     | Flags                     |
| ----------------------------------------- | ------------------------------- | ------------------------- |
| codanna plugin add <marketplace> <plugin> | Install plugin from marketplace | --ref, --force, --dry-run |
| codanna plugin remove <plugin>            | Remove installed plugin         | --force, --dry-run        |
| codanna plugin update <plugin>            | Update plugin to latest version | --ref, --force, --dry-run |
| codanna plugin list                       | List installed plugins          | --verbose, --json         |
| codanna plugin verify <plugin>            | Verify plugin integrity         | --verbose                 |

Common Flags:

- --ref <ref>: Specify Git branch, tag, or commit SHA
- --force: Override conflicts and safety checks
- --dry-run: Preview changes without executing
- --verbose: Show detailed information
- --json: Output as JSON for scripting

Safety Features

1. Transactional Installation: All-or-nothing installation with automatic rollback on failures
2. File Conflict Detection: Prevents overwriting files owned by other plugins (unless --force)
3. Integrity Verification: SHA-256 checksums detect file tampering or corruption
4. Backup and Restore: Updates back up existing version before changes
5. MCP Conflict Detection: Prevents duplicate MCP server keys (unless --force)
6. Namespaced Directories: Each plugin isolated in its own subdirectories

Error Handling

Common errors and solutions:

| Error                 | Cause                        | Solution                                            |
| --------------------- | ---------------------------- | --------------------------------------------------- |
| AlreadyInstalled      | Plugin already exists        | Use --force to reinstall                            |
| FileConflict          | File owned by another plugin | Check conflict owner, use --force to override       |
| McpServerConflict     | MCP key already exists       | Rename server or use --force                        |
| IntegrityCheckFailed  | Files modified/corrupted     | Reinstall with codanna plugin update <name> --force |
| PluginNotFound        | Plugin not in marketplace    | Check plugin name and marketplace URL               |
| InvalidPluginManifest | Manifest validation failed   | Contact plugin author to fix manifest               |

Creating Plugins

To create your own plugin:

1. Create a Git repository with this structure:

```text
   my-plugin/
   ├── .claude-plugin/
   │ └── plugin.json # Required manifest
   ├── commands/ # Optional: slash commands
   ├── agents/ # Optional: custom agents
   ├── hooks/ # Optional: event hooks
   └── scripts/ # Optional: utility scripts
```

2. Define the manifest (.claude-plugin/plugin.json):

```json
{
    "name": "my-plugin",
    "version": "1.0.0",
    "description": "What this plugin does",
    "author": { "name": "Your Name" },
    "commands": "./commands",
    "agents": "./agents"
}
```

3. Create a marketplace manifest (.claude-plugin/marketplace.json):

```json
{
    "name": "My Marketplace",
    "version": "1.0.0",
    "plugins": [
    {
        "name": "my-plugin",
        "description": "Plugin description",
        "source": {
        "type": "marketplace_path",
        "relative": "."
        }
    }
    ]
}
```

4. Publish to Git and share the repository URL

Users can then install with:

```bash
codanna plugin add https://github.com/you/my-plugin.git my-plugin
```

---

[Back to Documentation](../README.md)
