# Performance

Codanna performance characteristics and optimization.

## Parser Performance

Benchmarks on a 750-symbol test file:

| Language | Parsing Speed | vs. Target (10k/s) | Status |
|----------|---------------|-------------------|--------|
| **Rust** | 91,318 symbols/sec | 9.1x faster | Production |
| **Python** | 75,047 symbols/sec | 7.5x faster | Production |
| **TypeScript** | 82,156 symbols/sec | 8.2x faster | Production |
| **PHP** | 68,432 symbols/sec | 6.8x faster | Production |
| **Go** | 74,655 symbols/second | 7.5x faster | Production |

Run performance benchmarks:
```bash
codanna benchmark all          # Test all parsers
codanna benchmark python       # Test specific language
codanna benchmark rust --file src/main.rs  # Test with custom file
```

## Search Performance

- **Symbol lookups**: <10ms
- **Semantic search**: ~160ms (including embedding generation)
- **Vector access**: <1μs after OS page cache warm-up
- **MCP response time**: ~300ms (HTTP/HTTPS), 160ms (stdio)

## Memory Usage

- **Per symbol**: ~100 bytes
- **Model storage**: ~150MB (downloaded on first use)
- **Index storage**: Few MB (varies by codebase size)
- **Vector cache**: 384-dimensional vectors, memory-mapped

## Optimization Tips

### Indexing

**Control thread count:**
```bash
codanna index . --threads 16
```

**Skip large files:**
```toml
[indexing]
max_file_size_mb = 10
```

**Use .codannaignore:**
```
target/
node_modules/
dist/
build/
```

### Searching

**First search after startup may be slower** - Cache warming occurs

**Use language filters:**
```bash
# Reduces search space by language
codanna mcp semantic_search_docs query:"auth" lang:rust
```

**Adjust result limits:**
```bash
# Fewer results = faster
codanna mcp semantic_search_docs query:"config" limit:3
```

### Server Mode

**Watch interval:**
```toml
[server]
watch_interval = 5  # Seconds between index checks
```

Lower intervals mean more frequent checks but higher CPU usage.

**Cache configuration:**
```toml
[performance]
cache_size_mb = 100
vector_cache_size = 10000
```

## Architecture Highlights

### Memory-Mapped Storage

Two caches for different access patterns:
- `symbol_cache.bin` - FNV-1a hashed symbol lookups
- `segment_0.vec` - 384-dimensional vectors

### Lock-Free Concurrency

DashMap for concurrent symbol reads, write coordination via single writer lock.

### Single-Pass Indexing

Symbols, relationships, and embeddings extracted in one AST traversal.

### Hot Reload

File watcher with 500ms debounce triggers re-indexing of changed files only.

## Performance Targets

- **ALL operations complete in ~300ms** (stable)
- **Any regression beyond 500ms requires investigation**
- **MCP guidance adds <1ms overhead**

Test performance after every change.

## Monitoring

### Index Statistics
```bash
codanna mcp get_index_info --json
```

Shows:
- Total symbols
- Symbols by language
- Symbols by kind
- Index timestamps

### Benchmark Specific Files
```bash
codanna benchmark rust --file src/large_file.rs
```

## Troubleshooting Slow Performance

### Indexing Takes Too Long

1. Check thread count: `codanna index . --threads 8`
2. Use `.codannaignore` to skip unnecessary directories
3. Increase `max_file_size_mb` or skip large generated files

### Search is Slow

1. First search warms caches - subsequent searches are faster
2. Use language filters to reduce search space
3. Reduce result limits
4. Check system has sufficient RAM for caching

### High Memory Usage

1. Reduce cache sizes in configuration
2. Index fewer files
3. Check for memory leaks (report if found)

## See Also

- [Architecture](../architecture/) - System internals
- [Configuration](../user-guide/configuration.md) - Performance tuning options
- [CLI Reference](../user-guide/cli-reference.md) - Benchmark command