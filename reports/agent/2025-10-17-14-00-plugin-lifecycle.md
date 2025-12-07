# Research Report: Plugin Add/Remove/Update Lifecycle

**Date**: 2025-10-17 14:00
**Agent**: Research-Agent-v5
**Model**: Sonnet 4.5

## Summary

Codanna implements a Git-based plugin management system that extends Claude Code with custom commands, agents, hooks, scripts, and MCP servers. Plugins are installed from marketplace repositories, tracked in a lockfile with integrity checksums, and can be safely updated or removed with automatic rollback on failures.

## Key Findings

### 1. Plugin Add Process

**Entry Point**: `src/plugins/mod.rs:82-135` - `add_plugin()`

**Steps**:
1. **Resolve workspace root** - Determines target directory from settings
2. **Load lockfile** - Reads `.codanna/plugins/lockfile.json`
3. **Check existing installation** - Returns error if already installed (unless `--force`)
4. **Prepare plugin** - Downloads and validates plugin
5. **Execute installation** - Copies files, merges MCP config, updates lockfile
6. **Return commit SHA** - Confirms installation

**Evidence**: `src/plugins/mod.rs:82-135`

**Created Directories**:
```
.claude/
├── commands/     # Plugin commands
├── agents/       # Plugin agents
├── hooks/        # Plugin hooks
├── scripts/      # Plugin scripts
└── plugins/      # Plugin payload files

.codanna/
└── plugins/
    └── lockfile.json  # Tracks installed plugins
```

**Evidence**: `src/plugins/mod.rs:44-50`, `src/plugins/mod.rs:417-427`

### 2. Plugin Prepare Phase

**Function**: `src/plugins/mod.rs:766-869` - `prepare_plugin()`

**Steps**:
1. **Clone marketplace repository** to temp directory
2. **Load marketplace manifest** from `.claude-plugin/marketplace.json`
3. **Find plugin entry** in marketplace manifest
4. **Resolve plugin source**:
   - **MarketplacePath**: Extract subdirectory from marketplace repo
   - **Git**: Clone external repository (with optional subdir/ref)
5. **Load plugin manifest** from `.claude-plugin/plugin.json`
6. **Collect component files** based on manifest
7. **Check file conflicts** before installation
8. **Load MCP servers** configuration

**Evidence**: `src/plugins/mod.rs:766-869`

**Plugin Manifest Structure**:
```json
{
  "name": "plugin-name",
  "version": "1.0.0",
  "description": "Plugin description",
  "author": {"name": "Author Name"},
  "commands": "./commands",
  "agents": "./agents",
  "hooks": "./hooks",
  "scripts": "./scripts",
  "mcpServers": {
    "server-name": {
      "command": "command",
      "args": ["arg1", "arg2"]
    }
  }
}
```

**Evidence**: `src/plugins/plugin.rs:15-50`

### 3. Plugin Execute Installation

**Function**: `src/plugins/mod.rs:970-1112` - `execute_install_with_plan()`

**Steps with Rollback Protection**:

1. **Backup existing plugin** (if updating):
   - Saves all file contents
   - Saves MCP configuration
   - Evidence: `src/plugins/mod.rs:991-992`

2. **Uninstall previous version** (if updating):
   - Removes old files
   - Removes MCP servers
   - Evidence: `src/plugins/mod.rs:993`

3. **Copy component files**:
   - Commands → `.claude/commands/<plugin>/`
   - Agents → `.claude/agents/<plugin>/`
   - Hooks → `.claude/hooks/<plugin>/`
   - Scripts → `.claude/scripts/<plugin>/`
   - Evidence: `src/plugins/mod.rs:996-1020`

4. **Copy payload files**:
   - Everything else → `.claude/plugins/<plugin>/`
   - Excludes: `.git/`, component directories
   - Evidence: `src/plugins/mod.rs:1022-1046`

5. **Merge MCP servers**:
   - Loads `.mcp.json` or inline config
   - Merges into project `.mcp.json`
   - Tracks added server keys
   - Evidence: `src/plugins/mod.rs:1048-1066`

6. **Calculate integrity checksum**:
   - SHA-256 hash of all installed files
   - Excludes `.mcp.json` from hash
   - Evidence: `src/plugins/mod.rs:1068-1077`

7. **Update lockfile**:
   - Creates PluginLockEntry with metadata
   - Saves to `.codanna/plugins/lockfile.json`
   - Evidence: `src/plugins/mod.rs:1079-1095`

**Evidence**: `src/plugins/mod.rs:970-1112`

**Rollback Mechanism**:
If any step fails, `rollback_install()` executes:
- Removes copied files
- Restores previous plugin version
- Restores MCP configuration
- Cleans up plugin directories

**Evidence**: `src/plugins/mod.rs:1114-1149`

### 4. Plugin Update Process

**Function**: `src/plugins/mod.rs:196-292` - `update_plugin()`

**Update Detection**:
1. **Load existing entry** from lockfile
2. **Resolve remote commit SHA** from Git repository
3. **Compare commits**:
   - Same commit + passes verification → Already up to date
   - Same commit + fails verification → Reinstall
   - Different commit → Update
4. **Force mode** (`--force`) bypasses commit check

**Evidence**: `src/plugins/mod.rs:203-248`

**Update Execution**:
Uses same `execute_install_with_plan()` as add, but:
- Passes `previous_entry` for backup
- Uninstalls old version first
- Installs new version
- Rolls back to old version on failure

**Evidence**: `src/plugins/mod.rs:250-275`

**Version Tracking**:
```json
{
  "commit": "abc123...",
  "installed_at": "2025-10-17T13:58:03Z",
  "updated_at": "2025-10-17T14:00:00Z"
}
```

**Evidence**: `src/plugins/lockfile.rs:15-56`

### 5. Plugin Remove Process

**Function**: `src/plugins/mod.rs:153-194` - `remove_plugin()`

**Steps**:
1. **Load lockfile** - Get plugin entry
2. **Check if installed** - Return error if not found
3. **Uninstall plugin** - Remove files and config
4. **Update lockfile** - Remove plugin entry
5. **Save lockfile** - Persist changes

**Evidence**: `src/plugins/mod.rs:153-194`

**Uninstall Function**: `src/plugins/mod.rs:594-606` - `uninstall_plugin()`

**Cleanup Actions**:
1. **Remove tracked files**:
   - Iterates through lockfile `files` array
   - Deletes each file from filesystem
   - Evidence: `src/plugins/mod.rs:594-596`

2. **Remove MCP servers**:
   - Removes server entries from `.mcp.json`
   - Uses `mcp_keys` from lockfile
   - Evidence: `src/plugins/mod.rs:600-602`

3. **Cleanup directories**:
   - `.claude/plugins/<plugin-name>/`
   - `.claude/scripts/<plugin-name>/`
   - Evidence: `src/plugins/mod.rs:1151-1160`

4. **Remove lockfile entry**:
   - Deletes from `plugins` map
   - Evidence: `src/plugins/mod.rs:604`

**Evidence**: `src/plugins/mod.rs:594-606`

**Safety Checks**:
- TODO: Dependency graph checking (currently ignored)
- Integrity verification available via `verify_plugin()`

**Evidence**: `src/plugins/mod.rs:188` (comment)

### 6. Plugin Storage and Configuration

**Lockfile Location**: `.codanna/plugins/lockfile.json`

**Lockfile Structure**:
```json
{
  "version": "1.0.0",
  "plugins": {
    "plugin-name": {
      "name": "plugin-name",
      "version": "1.0.0",
      "commit": "abc123...",
      "marketplace_url": "https://github.com/user/marketplace.git",
      "installed_at": "2025-10-17T13:58:03Z",
      "updated_at": "2025-10-17T13:58:03Z",
      "integrity": "sha256:...",
      "files": ["relative/path/to/file.md"],
      "mcp_keys": ["server-name"],
      "source": {
        "type": "marketplace_path",
        "relative": "plugins/plugin-dir"
      }
    }
  }
}
```

**Evidence**: `.codanna/plugins/lockfile.json`, `src/plugins/lockfile.rs:8-62`

**Plugin Manifest Location**: `.claude-plugin/plugin.json` (inside plugin)

**Marketplace Manifest Location**: `.claude-plugin/marketplace.json` (inside marketplace repo)

**Evidence**: `src/plugins/mod.rs:776-781`

**MCP Configuration**: `.mcp.json` (project root)

Merged from plugin configuration:
```json
{
  "mcpServers": {
    "plugin-server": {
      "command": "command",
      "args": ["--flag"]
    }
  }
}
```

**Evidence**: `.mcp.json`, `src/plugins/merger.rs:15-83`

### 7. File Operations

**Copy Strategy**: `src/plugins/fsops.rs:8-48` - `copy_plugin_files()`

Component files (commands, agents, hooks, scripts):
- Destination: `.claude/<type>/<plugin>/<file>`
- Conflict detection before copying
- Creates parent directories as needed

**Evidence**: `src/plugins/fsops.rs:8-48`

**Payload Strategy**: `src/plugins/fsops.rs:50-104` - `copy_plugin_payload()`

Non-component files:
- Destination: `.claude/plugins/<plugin>/<file>`
- Excludes: `.git/`, component directories
- Preserves directory structure

**Evidence**: `src/plugins/fsops.rs:50-104`

**Conflict Detection**:
```rust
if dest_path.exists() && !force {
    let owner = conflict_owner(&dest_path);
    return Err(PluginError::FileConflict { path, owner });
}
```

**Evidence**: `src/plugins/fsops.rs:20-26`

**Integrity Calculation**: `src/plugins/fsops.rs:108-140` - `calculate_integrity()`

- SHA-256 hash of concatenated file contents
- Files sorted for deterministic ordering
- Evidence: `src/plugins/fsops.rs:108-140`

### 8. CLI Commands

**Command Structure**: `src/main.rs:324-434`

```bash
# Install plugin
codanna plugin add <marketplace> <plugin> [--ref <ref>] [--force] [--dry-run]

# Remove plugin
codanna plugin remove <plugin> [--force] [--dry-run]

# Update plugin
codanna plugin update <plugin> [--ref <ref>] [--force] [--dry-run]

# List installed plugins
codanna plugin list [--verbose] [--json]

# Verify plugin integrity
codanna plugin verify <plugin> [--verbose]
codanna plugin verify-all [--verbose]
```

**Evidence**: `src/main.rs:324-434`

**Flags**:
- `--force` - Overwrite conflicting files
- `--dry-run` - Show what would happen without executing
- `--ref` - Specify Git ref (branch/tag/commit)
- `--verbose` - Show detailed information
- `--json` - Output as JSON

### 9. Error Handling

**Error Types**: `src/plugins/error.rs:1-200`

- `AlreadyInstalled` - Plugin already exists (use `--force` to override)
- `NotInstalled` - Plugin not found in lockfile
- `PluginNotFound` - Plugin not in marketplace
- `FileConflict` - File owned by different plugin
- `McpServerConflict` - MCP server key already exists
- `IntegrityCheckFailed` - File contents don't match checksum
- `LockfileCorrupted` - Cannot parse lockfile
- `InvalidPluginManifest` - Manifest validation failed

**Evidence**: `src/plugins/error.rs:1-200`

**Verification**: `src/plugins/mod.rs:348-383` - `verify_plugin()`

Checks:
1. All tracked files exist
2. File contents match integrity checksum
3. MCP server keys present in `.mcp.json`

**Evidence**: `src/plugins/mod.rs:348-383`, `src/plugins/mod.rs:537-592`

## Architecture/Patterns Identified

### 1. Transactional Installation

Plugins use a prepare-execute-verify pattern with automatic rollback:
- Prepare phase validates in temp directories
- Execute phase copies files with conflict detection
- Rollback mechanism restores previous state on failure

### 2. Git-Based Marketplace

Plugins distributed via Git repositories:
- Marketplace manifest lists available plugins
- Plugins can be embedded or reference external repos
- Git commit SHA provides version tracking

### 3. Integrity Protection

SHA-256 checksums protect against file tampering:
- Calculated during installation
- Verified on demand
- Excludes `.mcp.json` (merged configuration)

### 4. Namespaced Installation

Each plugin gets isolated directories:
- Commands: `.claude/commands/<plugin>/`
- Agents: `.claude/agents/<plugin>/`
- Plugins: `.claude/plugins/<plugin>/`
- Scripts: `.claude/scripts/<plugin>/`

Prevents file conflicts between plugins.

### 5. MCP Configuration Merging

MCP servers merged into project configuration:
- Plugin specifies server configuration
- Merged into project `.mcp.json`
- Keys tracked for removal
- Conflict detection with `--force` override

## Conclusions

**Plugin Lifecycle Summary**:

1. **Add**: Clone marketplace → Validate manifest → Copy files → Merge MCP → Update lockfile
2. **Update**: Check remote commit → Backup existing → Uninstall old → Install new → Rollback on failure
3. **Remove**: Load entry → Remove files → Remove MCP servers → Cleanup directories → Update lockfile

**Key Design Decisions**:
- Git-based distribution enables version control and easy updates
- Lockfile with integrity checksums ensures consistency
- Transactional installation with rollback prevents partial states
- Namespaced directories prevent conflicts
- MCP configuration merging enables server extensions

**Missing Features** (noted in code):
- Dependency graph checking (TODO in remove process)
- No automatic update checking
- No plugin signing/verification beyond Git commit

**Recommendations for Documentation**:
1. Provide example marketplace structure
2. Document plugin manifest schema
3. Show common plugin patterns
4. Explain MCP server merging behavior
5. Document rollback guarantees
