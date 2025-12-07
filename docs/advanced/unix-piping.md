# Unix Piping

Codanna speaks CLI like you do, positional when it's simple, key:value when it's not.
All MCP tools support `--json`, so piping isn't noise, it's music.

## Basic Piping

### MCP Semantic Search with Language Filter
```bash
codanna mcp semantic_search_with_context query:"error handling" limit:2 lang:rust --json | jq -r '.data[] | "\(.symbol.name) (\(.symbol.scope_context)) (score: \(.score)) - \(.context.file_path) - \(.symbol.doc_comment)"'
# Output: error (ClassMember) (score: 0.6421908) - src/io/format.rs:148 - Create a generic error response.
#         add_error (ClassMember) (score: 0.6356536) - src/indexing/progress.rs:46 - Add an error (limited to first 100 errors)
```

### Show Symbol Types, Names and Locations
```bash
codanna retrieve search "config" --json | jq -r '.items[] | "\(.symbol.kind) \(.symbol.name) @ \(.file_path)"'
# Output: Function test_partial_config @ src/config.rs:911
#         Method config_key @ src/parsing/language.rs:114

# Get unique file paths for search results
codanna retrieve search "parser" --json | jq -r '.items[].file_path' | sort -u

# Extract function signatures with scope context
codanna retrieve search "create_parser" --json | jq -r '.items[] | "\(.symbol.name) (\(.symbol.scope_context)) - \(.file_path)\n  \(.symbol.signature)"'
```

## Advanced Piping: Extract System Messages and Map Call Graphs

System messages guide agents toward the next hop. Humans don't see them, but piping with jq reveals them:

```bash
# Extract system guidance from tool responses
codanna mcp find_callers walk_and_stream --json | jq -r '.system_message'
# Output: Found 18 callers. Run 'analyze_impact' to map the change radius.

# Build a complete call graph: find a symbol, show what it calls, and trace one level deeper
codanna mcp semantic_search_with_context query:"file processing" limit:1 --json | \
jq -r '.data[0].symbol.name' | \
xargs -I {} sh -c '
  echo "=== Symbol: {} ==="
  codanna mcp get_calls {} --json | jq -r ".data[]? | \"\(.name) - \(.file_path):\(.range.start_line)-\(.range.end_line)\""
'
# Output:
# === Symbol: walk_and_stream ===
# process_entry - src/io/parse.rs:285-291
# parse_file - src/io/parse.rs:219-282
# ...

# Reverse it: find who calls a critical function and show exact line ranges
codanna mcp find_callers parse_file --json | \
jq -r '.data[]? | "\(.name) (\(.kind)) - \(.file_path):\(.range.start_line)-\(.range.end_line)"'
# Output:
# walk_and_stream (Function) - src/io/parse.rs:144-213
# index_project (Method) - src/indexing/mod.rs:423-502
```

## Common Patterns

### Find and Count
```bash
# Count symbols by type
codanna retrieve search "" --json | jq -r '.items[].symbol.kind' | sort | uniq -c | sort -rn

# Count symbols per file
codanna retrieve search "" --json | jq -r '.items[].file_path' | sort | uniq -c | sort -rn
```

### Filter and Transform
```bash
# Find all public functions
codanna retrieve search "" --json | jq -r '.items[] | select(.symbol.kind == "Function") | .symbol.name'

# Find all structs with their file locations
codanna retrieve search "" --json | jq -r '.items[] | select(.symbol.kind == "Struct") | "\(.symbol.name) in \(.file_path)"'
```

### Chain Commands
```bash
# Find a trait and all its implementations
TRAIT="Parser"
echo "=== Trait: $TRAIT ==="
codanna mcp find_symbol $TRAIT --json | jq -r '.data[0].file_path'
echo "=== Implementations ==="
codanna retrieve implementations $TRAIT --json | jq -r '.items[].symbol.name'
```

### Analyze Dependencies
```bash
# Get a symbol's complete dependency graph
SYMBOL="SimpleIndexer"
codanna retrieve dependencies $SYMBOL --json | \
jq -r '.dependencies[] | "\(.name) (\(.kind)) - \(.file_path)"'
```

## Tips

- Use `--json` flag with all commands for structured output
- Pipe to `jq` for JSON manipulation
- Combine with standard Unix tools: `sort`, `uniq`, `grep`, `awk`
- Use `xargs` to chain commands based on output
- Save complex pipelines as shell scripts

## Exit Codes

All retrieve commands use exit code 3 when not found, useful for scripting:

```bash
if codanna retrieve symbol MySymbol --json > /dev/null 2>&1; then
    echo "Symbol found"
else
    if [ $? -eq 3 ]; then
        echo "Symbol not found"
    else
        echo "Error occurred"
    fi
fi
```

## See Also

- [CLI Reference](../user-guide/cli-reference.md)
- [MCP Tools](../user-guide/mcp-tools.md)