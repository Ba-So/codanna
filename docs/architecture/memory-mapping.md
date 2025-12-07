# Memory-Mapped Storage

Codanna uses memory-mapped files for instant loading and high-performance access.

## Two-Cache Architecture

Different access patterns require different cache designs:

### Symbol Cache (`symbol_cache.bin`)
- **Purpose**: Fast symbol lookups by name
- **Hash**: FNV-1a for distribution
- **Access**: <10ms response time
- **Size**: ~100 bytes per symbol
- **Storage**: Compact symbol representation

### Vector Cache (`segment_0.vec`)
- **Purpose**: Semantic similarity search
- **Dimensions**: Configurable (384/768/1024 based on model)
- **Access**: <1μs after OS page cache warm-up
- **Storage**: Binary-packed floating-point arrays
- **Organization**: IVFFlat clustering for fast lookup

## Memory-Mapped Benefits

### Instant Startup
- No deserialization on load
- OS maps file directly to memory
- Application sees it as regular memory
- First access triggers page loading

### Efficient Memory Usage
- OS manages paging automatically
- Inactive pages can be swapped out
- Multiple processes share same physical memory
- No manual cache management needed

### Persistence
- Data persists between runs
- No rebuild on restart
- Atomic writes prevent corruption
- File system handles durability

## Symbol Cache Structure

```rust
struct CompactSymbol {
    id: NonZeroU32,           // 4 bytes
    kind: u8,                 // 1 byte
    file_id: NonZeroU32,      // 4 bytes
    range: CompactRange,      // 8 bytes (start/end)
    name_hash: u64,           // 8 bytes (FNV-1a)
    flags: u8,                // 1 byte
    // Total: 26 bytes + padding = 32 bytes (cache-line aligned)
}
```

**Cache-line alignment**: 32 bytes per symbol, 2 symbols fit per 64-byte cache line.

## Vector Cache Structure

```
segment_0.vec:
├── Header (metadata)
│   ├── Model name
│   ├── Dimensions
│   ├── Vector count
│   └── Cluster count
├── Cluster metadata
│   ├── Cluster centroids
│   └── Cluster boundaries
└── Vector data
    ├── Vector 0: [f32; dimensions]
    ├── Vector 1: [f32; dimensions]
    └── ...
```

**Storage format**: Binary-packed f32 arrays using bincode for serialization.

## IVFFlat Clustering

Vectors are organized using Inverted File with Flat vectors:

1. **K-means clustering** groups similar vectors
2. **Centroids** represent each cluster
3. **Search** checks nearby clusters first
4. **Reduces** comparisons from N to ~sqrt(N)

Example with 10,000 vectors:
- Without clustering: 10,000 comparisons
- With 100 clusters: ~1,000 comparisons (10x faster)

## Cache Warming

First access loads pages into OS cache:

```
Cold start: 100-200ms (loading from disk)
Warm cache: <1μs (already in RAM)
```

**Hot paths warm up quickly** - frequent queries benefit from OS caching.

## Write Operations

### Symbol Cache Updates
1. Build new symbol cache in memory
2. Write to temporary file
3. Atomic rename to `symbol_cache.bin`
4. OS remaps memory on next access

### Vector Cache Updates
1. Generate new embeddings
2. Re-cluster vectors with K-means
3. Write new segment file
4. Delete old embeddings
5. Update metadata

**Crash safety**: Old files remain valid until new ones are complete.

## Storage Layout

```
.codanna/index/
├── symbol_cache.bin        # FNV-1a hashed symbols
└── vectors/
    ├── segment_0.vec       # Vector data
    ├── segment_1.vec       # (if needed)
    ├── metadata.bin        # Index metadata
    └── clusters.bin        # Cluster information
```

## Memory Requirements

For a project with 100,000 symbols:

**Symbol cache:**
- 100,000 symbols × 32 bytes = 3.2 MB

**Vector cache (384-dim model):**
- 100,000 vectors × 384 floats × 4 bytes = 153.6 MB

**Total:** ~157 MB (plus OS overhead)

## Scalability

Memory-mapped files scale to:
- Millions of symbols
- Gigabytes of vector data
- Multiple concurrent readers
- OS handles paging automatically

## Performance Characteristics

### Read Performance
- Symbol lookup: O(1) with FNV-1a hash
- Vector search: O(sqrt(N)) with IVFFlat
- No serialization overhead
- Cache-line aligned access

### Write Performance
- Batch updates preferred
- Atomic file replacement
- No locking for readers
- Background re-clustering

## Zero-Copy Deserialization

Using `rkyv` for zero-copy:
- No parsing on load
- Direct memory access
- Type-safe operations
- Instant availability

## Troubleshooting

### High Memory Usage
- OS maps entire file but doesn't load it all
- Use `vmstat` to see actual RAM usage
- Inactive pages get swapped naturally

### Slow First Search
- OS loading pages from disk
- Subsequent searches are fast
- Pre-warm with `cat .codanna/index/vectors/segment_0.vec > /dev/null`

### Corruption Recovery
- Delete corrupted cache files
- Re-run `codanna index` to rebuild
- Atomic writes prevent partial updates

## See Also

- [How It Works](how-it-works.md) - System overview
- [Embedding Model](embedding-model.md) - Vector generation
- [Performance](../advanced/performance.md) - Optimization tips