# Agent Guidance

For optimal usage, add to your project instructions (`CLAUDE.md`, `AGENTS.md` or your system prompt):

```markdown
## Codanna MCP Tools

Tool priority:
- **Tier 1**: semantic_search_with_context, analyze_impact
- **Tier 2**: find_symbol, get_calls, find_callers
- **Tier 3**: search_symbols, semantic_search_docs, get_index_info

Workflow:
1. semantic_search_with_context - Find relevant code with context
2. analyze_impact - Map dependencies and change radius
3. find_symbol, get_calls, find_callers - Get specific details

Start with semantic search, then narrow with specific queries.
```

## Claude Sub Agent

We include a **codanna-navigator** sub agent (`.claude/agents/codanna-navigator.md`) that knows how to use codanna effectively.

## Agent Steering

Codanna's guidance is model-facing. Each tool response includes a system_message the LLM reads and acts on. Humans do not see it. The message tells the agent the next hop: drill down, follow calls, analyze impact, refine the query.

### Behavior Examples

```json
{
  "system_message": "Found 1 match. Use 'find_symbol' or 'get_calls' next."
}
```

```json
{
  "system_message": "Found 18 callers. Run 'analyze_impact' to map the change radius."
}
```

```json
{
  "system_message": "No semantic matches. Try broader phrasing or ensure docs exist."
}
```

## Configuration

Config is plain TOML `.codanna/settings.toml`:

```toml
[guidance]
enabled = true

[guidance.templates.find_callers]
no_results = "No callers found. Might be an entry point or dynamic dispatch."
single_result = "Found 1 caller. Use 'find_symbol' to inspect usage."
multiple_results = "Found {result_count} callers. Try 'analyze_impact' for the full graph."

[guidance.templates.analyze_impact]
no_results = "No impact detected. Likely isolated."
single_result = "Minimal impact radius."
multiple_results = "Impact touches {result_count} symbols. Focus critical paths."

[[guidance.templates.analyze_impact.custom]]
min = 20
template = "Significant impact with {result_count} symbols. Break the change into smaller parts."
```

## Why It Matters

- Fewer round trips. The agent self-proposes the next command.
- Less narration. More execution.
- Grep-and-hope becomes directed hops.

## Claude Slash Commands

Codanna includes custom slash commands for Claude that provide intelligent workflows for code exploration:

| Command | Description | Example Report |
|---------|-------------|----------------|
| `/find <query>` | Smart semantic search with natural language - finds symbols, patterns, and implementations using optimized queries | [Language Registry Investigation](../../reports/find/find-language-registry-scaffold.md) |
| `/deps <symbol>` | Analyze dependencies of a symbol - shows what it depends on, what depends on it, coupling metrics, and refactoring opportunities | [find_symbol Dependencies](../../reports/deps/find_symbol-method-dependencies.md) |

These commands use Codanna's MCP tools under the hood but provide guided workflows with comprehensive analysis and automatic report generation.

## Extracting System Messages

System messages guide agents but are hidden from users. Use piping to reveal them:

```bash
# Extract system guidance from tool responses
codanna mcp find_callers walk_and_stream --json | jq -r '.system_message'
# Output: Found 18 callers. Run 'analyze_impact' to map the change radius.
```

## See Also

- [MCP Tools](../user-guide/mcp-tools.md)
- [Claude Code Integration](claude-code.md)
- [Configuration](../user-guide/configuration.md)