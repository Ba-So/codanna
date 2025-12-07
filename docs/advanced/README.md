[Documentation](../README.md) / **Advanced**

---

# Advanced

## Nerds Section

Codanna respects `.gitignore` and adds its own `.codannaignore`.

## Unix-Native. Pipe it, baby!

Codanna speaks CLI like you do, positional when it's simple, key:value when it's not.
All MCP tools support `--json`, so piping isn't noise, it's music.

## In This Section

- **[Unix Piping](unix-piping.md)** - Advanced piping workflows and examples
- **[Slash Commands](slash-commands.md)** - Custom /find and /deps commands
- **[Project Resolution](project-resolution.md)** - TypeScript tsconfig.json and path aliases
- **[Performance](performance.md)** - Benchmarks and optimization

## Quick Examples

### Semantic Search with Language Filter
```bash
codanna mcp semantic_search_with_context query:"error handling" limit:2 lang:rust --json | jq -r '.data[] | "\(.symbol.name) (\(.symbol.scope_context)) (score: \(.score)) - \(.context.file_path) - \(.symbol.doc_comment)"'
```

### Build Complete Call Graphs
```bash
# Find a symbol, show what it calls, and trace one level deeper
codanna mcp semantic_search_with_context query:"file processing" limit:1 --json | \
jq -r '.data[0].symbol.id' | \
xargs -I {} sh -c '
  echo "=== Symbol ID: {} ==="
  codanna mcp get_calls symbol_id:{} --json | jq -r ".data[]? | \"\(.name) [symbol_id:\(.id)] - \(.file_path):\(.range.start_line)-\(.range.end_line)\""
'
```

### Using symbol_id for Unambiguous Queries
```bash
# Extract symbol_id from search results and use for precise follow-up
codanna mcp semantic_search_with_context query:"error handling" limit:1 --json | \
jq -r '.data[0] | "Symbol: \(.symbol.name) [symbol_id:\(.symbol.id)]"'

# Direct lookup by ID (no ambiguity)
codanna mcp get_calls symbol_id:1883 --json | jq -r '.data[] | "\(.name) [symbol_id:\(.id)]"'
```

### Extract System Messages
System messages guide agents toward the next hop. Humans don't see them, but piping with jq reveals them:
```bash
codanna mcp find_callers walk_and_stream --json | jq -r '.system_message'
# Output: Found 18 callers. Run 'analyze_impact' to map the change radius.
```

## Why It Matters

- Fewer round trips. The agent self-proposes the next command.
- Less narration. More execution.
- Grep-and-hope becomes directed hops.

## Next Steps

- Explore [Architecture](../architecture/) internals
- Read about [User Guide](../user-guide/) for basic usage
- Check [Reference](../reference/) for specifications

[Back to Documentation](../README.md)