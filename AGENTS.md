# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Core crate. Key modules: `parsing/` (Rust, Python, TypeScript, PHP, Go parsers), `indexing/`, `storage/` (incl. Tantivy), `semantic/`, `io/` (CLI I/O), `mcp/` (server + client), `display/`, `guidance/`, and `vector/`.
- `tests/`: Integration tests (e.g., `test_rust_resolution.rs`, `test_python_*`).
- `examples/`: Small codebases and scripts used by tests and docs.
- `benches/`: Criterion benchmarks.
- `.github/workflows/`: CI pipelines (quick checks, full test, release).
- `contributing/`, `reports/`, `.claude/`: Contributor docs, investigations, and agent assets.

## Build, Test, and Development Commands
- Build: `cargo build` (debug) · `cargo build --release` (optimized).
- Run CLI: `cargo run -- <command>`
  - Examples: `cargo run -- init`, `cargo run -- index src`, `cargo run -- retrieve search "parse" --limit 5`.
- Serve (HTTP/HTTPS): `cargo run --features http-server -- serve --http --watch` or `cargo run --features https-server -- serve --https`.
- Tests: `cargo test` · Filter: `cargo test test_python_scope_tracking` · One file: `cargo test --test test_rust_resolution`.
- Benchmarks: `cargo bench`.

## Coding Style & Naming Conventions
- Rust style via `rustfmt` and `clippy`.
  - Format: `cargo fmt --all` (CI expects formatted code).
  - Lint: `cargo clippy -- -D warnings` (treat warnings as errors).
- Naming: `snake_case` (fns/modules), `CamelCase` (types/traits), `SCREAMING_SNAKE_CASE` (consts).
- Docs/Markdown: markdownlint config present (`markdownlint.toml`/`.json`); if installed, run: `markdownlint '**/*.md'`.

## Testing Guidelines
- Prefer fast, deterministic tests. Unit tests live inline under `#[cfg(test)]`; integration tests live in `tests/` and typically start with `test_`.
- Add tests for new language behaviors, relationship resolution, and CLI output paths.
- Use filters to iterate quickly (see commands above). Bench heavy work under `benches/`.

## Commit & Pull Request Guidelines
- Commits: clear, imperative, scoped (e.g., "parser: fix TypeScript re-exports"). Group related changes; keep diffs focused.
- PRs must: include a concise description, link related issues, add/adjust tests, update docs/CHANGELOG when user‑visible, and pass CI (quick + full).
- Include usage examples when changing CLI (e.g., `codanna retrieve ...`).

## Security & Configuration Tips
- Initialize config: `cargo run -- init` creates `.codanna/settings.toml`. Use `--config` to point at custom files; `--info` prints load details.
- Do not commit local indexes or secrets. Local caches live under `.codanna/` (ignored); keep credentials and tokens out of the repo.

## Codanna MCP Tools
- Start with semantic tools to anchor on the right files and APIs; they provide the highest‑quality context.
- Then use find_symbol and search_symbols to lock onto exact files and kinds.
- Treat get_calls/find_callers/analyze_impact as hints; confirm with code reading or tighter queries (unique names, kind filters).

## Development Guidelines

### Mandatory Reading

**IMPORTANT**: All code must follow our [Rust Development Guidelines](contributing/development/guidelines.md). Key principles:

1. **Zero-Cost Abstractions**: No unnecessary allocations
2. **Type Safety**: Use newtypes, not primitives
3. **Performance**: Must meet performance targets
4. **Error Handling**: Structured errors with suggestions
5. **Function Design**: Decompose complex logic into focused helper methods

## Current Sprint

**IMPORTANT** before start implementing make sure you read an understand the documentation below! Think deeper for this session.

[ABI-15 Exploration Test Suite](tests/abi15_exploration.rs)
[Language Support Documentation](contributing/development/language-support.md)