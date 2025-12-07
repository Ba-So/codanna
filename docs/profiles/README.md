# Profile System

Codanna profiles let teams package reusable configuration, hooks, and commands. Profiles are distributed by **providers** (git repositories or local folders) and installed per workspace while their registry lives in `~/.codanna`.

---

## Key Concepts

| Term | Description |
|------|-------------|
| **Provider** | Source of profiles (GitHub shorthand, git URL, or local path) |
| **Profile** | Bundle containing manifests, hooks, and optional MCP agents |
| **Global Registry** | Stored at `~/.codanna/providers.json`; tracks registered providers |
| **Workspace Install** | Profiles installed into `.codanna/profiles.lock.json` for a project |

---

## Typical Workflow

1. **Register a provider**
   ```bash
   codanna profile provider add bartolli/codanna-profiles
   ```
2. **Preview available profiles**
   ```bash
   codanna profile list --verbose
   ```
3. **Install to the current workspace**
   ```bash
   codanna profile install claude
   ```
4. **Inspect installed profiles**
   ```bash
   codanna profile status
   ```
5. **Update or verify as the project evolves**
   ```bash
   codanna profile update claude
   codanna profile verify claude
   ```

---

## Provider Sources

| Type | Format | Example |
|------|--------|---------|
| GitHub shorthand | `owner/repo` | `bartolli/codanna-profiles` |
| Git URL | `https://...` | `https://github.com/bartolli/codanna-profiles` |
| Local path | absolute or relative | `/Users/name/my-profiles` |

### Register a Provider

```bash
codanna profile provider add bartolli/codanna-profiles
codanna profile provider add https://github.com/org/profiles.git
codanna profile provider add ./my-profiles
```

### Remove / Inspect Providers

```bash
codanna profile provider remove codanna-profiles
codanna profile provider list --verbose
```

---

## Installing and Managing Profiles

| Command | Purpose | Flags |
|---------|---------|-------|
| `codanna profile install <name>` | Install profile into workspace | `--force` |
| `codanna profile update <name>` | Update installed profile | `--force` |
| `codanna profile remove <name>` | Uninstall profile | `--verbose` |
| `codanna profile list` | List profiles from providers | `--verbose`, `--json` |
| `codanna profile status` | Show installed profiles | `--verbose` |
| `codanna profile sync` | Install from team config | `--force` |
| `codanna profile verify [<name>]` | Check integrity | `--all`, `--verbose` |

Examples:
```bash
codanna profile install claude
codanna profile update claude --force
codanna profile remove claude --verbose
codanna profile sync --force
codanna profile verify --all --verbose
```

---

## Profile Structure

Providers follow this layout:

```
.codanna-profile/
├── provider.json          # Provider metadata
└── profiles/
    └── profile-name/
        ├── profile.json   # Manifest (hooks, prompts, requirements)
        ├── .claude/       # Claude Code instructions / assets
        └── CLAUDE.md      # Optional documentation
```

---

## Storage Locations

| Location | Purpose |
|----------|---------|
| `~/.codanna/providers.json` | Global provider registry |
| `~/.codanna/profiles/` | Cached provider clones |
| `<workspace>/.codanna/profiles.lock.json` | Installed profiles for the project |

---

## Tips

- Use `--verbose` to inspect what a provider offers before installing.
- `codanna profile sync` is ideal for onboarding—repositories can commit a lockfile and teammates run `sync` to match.
- Combine `profile verify` in CI to ensure workspaces aren’t using stale or tampered hooks.
- Providers can host multiple profiles (e.g., `backend`, `frontend`, `ops`), letting teams mix and match.

For CLI command syntax, see the [CLI Reference](../user-guide/cli-reference.md#profile-system).
