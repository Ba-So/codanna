# Tree-sitter CLI Local Development

This directory provides tools for exploring AST structures using the official tree-sitter CLI, helpful for debugging parser implementations.

## Quick Start

1. **Install a grammar** (one-time setup per language):
   ```bash
   ./scripts/setup.sh typescript
   ```

2. **Parse files and explore AST**:
   ```bash
   # Use tree-sitter directly (works from any directory)
   tree-sitter parse examples/typescript/comprehensive.ts

   # Or use our helper script
   ./scripts/explore-ast.sh examples/typescript/comprehensive.ts
   ```

3. **Compare with our parser**:
   ```bash
   # From project root
   ./scripts/compare-nodes.sh typescript
   ```

## Setup Script

The setup script configures tree-sitter and installs grammars on-demand:

```bash
# Show installed grammars
./scripts/setup.sh

# Install specific language grammar
./scripts/setup.sh python
./scripts/setup.sh rust
./scripts/setup.sh go
```

Supported languages: typescript, python, rust, go, php, c, cpp

## Available Scripts

All scripts are located in `contributing/tree-sitter/scripts/`:

| Script | Purpose | Input | Example |
|--------|---------|-------|---------|
| `setup.sh` | Configure tree-sitter and install grammars | Language name | `./scripts/setup.sh typescript` |
| `explore-ast.sh` | Parse ANY file and display its AST | File path + mode | `./scripts/explore-ast.sh file.ts both` |
| `compare-nodes.sh` | Compare codanna with tree-sitter | Language or file path | See below |

### explore-ast.sh
Parse files with codanna and/or tree-sitter:
```bash
# Default: Use codanna parse (named nodes only)
./scripts/explore-ast.sh examples/rust/main.rs

# Use tree-sitter
./scripts/explore-ast.sh examples/rust/main.rs tree-sitter

# Compare both parsers
./scripts/explore-ast.sh examples/rust/main.rs both
```

### compare-nodes.sh
**Two modes:**
- **Language mode**: `./scripts/compare-nodes.sh typescript`
  - Compares comprehensive.* files with our parser
  - Triggers audit report generation
  - Shows differences between parsers

- **File mode**: `./scripts/compare-nodes.sh path/to/file.ts`
  - Compares AST nodes between codanna and tree-sitter
  - Saves detailed output to `{filename}_comparison.log`
  - Shows matching statistics and differences

## How It Works

1. Tree-sitter config is saved to `~/.config/tree-sitter/config.json`
2. Grammars are cloned to `contributing/tree-sitter/grammars/`
3. Tree-sitter automatically finds grammars based on the config
4. File extensions determine which grammar to use (.ts → typescript, .py → python)

## Notes

- Grammars are cloned with `--depth 1` for speed
- The grammars directory is gitignored
- Each developer's tree-sitter config points to their local grammar directory
- No environment variables or .env files needed - tree-sitter handles it
