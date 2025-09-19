---
Title: File Watcher and Hot Reload System Analysis
Repo: codanna
Commit: 78486ad
Index: 3476 symbols, 129 files
Languages: Rust
Date: September 07, 2025 at 10:03 PM
Model: claude-opus-4-1-20250805
---

# Code Research Report

## 1. Inputs and Environment

Tools: codanna-https MCP server v0.5.9
Limits: Unknown

## 2. Investigation Path

| Step | Tool        | Input                  | Output summary          | Artifact             |
|------|-------------|------------------------|-------------------------|----------------------|
| 1    | semantic_search_with_context | "file watcher hot reload IndexWatcher FileSystemWatcher" | Found 10 components including IndexWatcher, FileSystemWatcher, check_and_reload | see Evidence §5.1 |
| 2    | find_symbol | "IndexWatcher" | Found struct at src/mcp/watcher.rs:20 with 7 methods | see Evidence §5.2 |
| 3    | find_symbol | "FileSystemWatcher" | Found struct at src/indexing/fs_watcher.rs:57 with 5 methods | see Evidence §5.3 |
| 4    | semantic_search_docs | "process file events debounce channel receiver" | Found event_rx field and process methods | see Evidence §5.4 |
| 5    | find_symbol | "NotificationBroadcaster" | Found struct at src/mcp/notifications.rs:19 with 4 methods | see Evidence §5.5 |
| 6    | search_symbols | "serve function watch" | Found serve_http and serve_https functions | see Evidence §5.6 |
| 7    | get_index_info | "" | 3476 symbols, 129 files, semantic search enabled | see §4 |

## 3. Mechanics of the Code

### Two-Layer Watching Architecture
- **IndexWatcher**: Monitors index file changes (meta.json), reloads on modifications
- **FileSystemWatcher**: Monitors indexed source files, triggers re-indexing on changes
- Both watchers operate independently but share notification infrastructure

### Event Flow
- File change detected by notify crate → FileSystemWatcher event channel
- Events debounced (default 500ms) to batch rapid changes  
- Re-indexing triggered, hash comparison prevents unnecessary work
- Notifications broadcast to all MCP server instances via NotificationBroadcaster
- IndexWatcher checks source files if file_watch.enabled = true

### Communication Layer
- NotificationBroadcaster uses tokio broadcast channel (capacity 100)
- FileChangeEvent enum: FileReindexed, FileCreated, FileDeleted, IndexReloaded
- MCP servers subscribe and forward notifications as resource_updated events

## 4. Quantified Findings

**Index Statistics:**
- Symbols: 3476
- Files: 129  
- Relationships: 6904
- Semantic embeddings: 1274

**Watcher Configuration:**
- Default debounce: 500ms (file changes)
- Default check interval: 5 seconds (index reload)
- Channel buffer: 100 events
- Broadcast capacity: 100 subscribers

**Component Counts:**
- IndexWatcher methods: 7
- FileSystemWatcher methods: 5
- NotificationBroadcaster methods: 4
- FileChangeEvent variants: 4

## 5. Evidence

### 5.1 IndexWatcher Implementation
```rust
// src/mcp/watcher.rs:92
async fn check_and_reload(&mut self) -> Result<(), Box<dyn std::error::Error>>
```

### 5.2 IndexWatcher Structure
```rust  
// src/mcp/watcher.rs:20
pub struct IndexWatcher {
    index_path: PathBuf,
    indexer: Arc<RwLock<SimpleIndexer>>,
    settings: Arc<Settings>,
    persistence: IndexPersistence,
    last_modified: Option<SystemTime>,
    check_interval: Duration,
    mcp_server: Option<Arc<CodeIntelligenceServer>>,
    broadcaster: Option<Arc<NotificationBroadcaster>>,
}
```

### 5.3 FileSystemWatcher Structure
```rust
// src/indexing/fs_watcher.rs:57
pub struct FileSystemWatcher {
    indexer: Arc<RwLock<SimpleIndexer>>,
    debounce_ms: u64,
    event_rx: mpsc::Receiver<notify::Result<Event>>,
    _watcher: notify::RecommendedWatcher,
    mcp_debug: bool,
    broadcaster: Option<Arc<NotificationBroadcaster>>,
}
```

### 5.4 Debounce Processing
```rust
// src/indexing/fs_watcher.rs:299
pending_changes.retain(|path, last_change| {
    if now.duration_since(*last_change) >= debounce_duration {
        files_to_process.push(path.clone());
        false // Remove from pending
    } else {
        true // Keep in pending
    }
});
```

### 5.5 Notification Broadcasting
```rust
// src/mcp/notifications.rs:41
pub fn send(&self, event: FileChangeEvent) {
    match self.sender.send(event.clone()) {
        Ok(count) => { /* broadcast to count subscribers */ }
        Err(_) => { /* no receivers */ }
    }
}
```

### 5.6 Watcher Initialization in HTTP Server
```rust
// src/mcp/http_server.rs:93
match FileSystemWatcher::new(watcher_indexer, debounce_ms, config.mcp.debug) {
    Ok(watcher) => {
        let watcher = watcher.with_broadcaster(watcher_broadcaster);
        tokio::spawn(async move {
            tokio::select! {
                result = watcher.watch() => { /* handle result */ }
                _ = watcher_ct.cancelled() => { /* shutdown */ }
            }
        });
    }
}
```

## 6. Implications

### Memory Usage
- Each watcher maintains HashMap for pending changes
- Broadcast channel holds up to 100 events = ~10KB buffer
- Index reload creates new SimpleIndexer instance temporarily

### Latency Calculations
- Minimum file change detection: 100ms (tokio select timeout)
- Typical re-index latency: 500ms debounce + indexing time
- Index reload check: Every 5 seconds
- Maximum delay for change propagation: 5.6 seconds worst case

## 7. Hidden Patterns

### Dual Watching Modes
- IndexWatcher also checks source files when file_watch.enabled
- Creates redundancy with FileSystemWatcher but different granularity

### Hash-Based Optimization
- Both watchers use content hashing to skip unchanged files
- IndexingResult::Cached indicates hash match, no re-processing

### Directory-Level Monitoring
- FileSystemWatcher watches parent directories, not individual files
- Filters events to only process indexed files
- compute_watch_dirs() minimizes watched directory set

### Dynamic Watch List Updates
- FileSystemWatcher subscribes to IndexReloaded events
- Refreshes watched file list when index changes
- Adds/removes directories as needed

## 8. Research Opportunities

- Investigate impact of debounce_ms on large file batch changes using semantic_search_with_context query:"batch process multiple files"
- Analyze notification delivery patterns with find_callers function_name:send
- Explore hash collision handling with search_symbols query:"hash content file_hash"
- Check memory leak potential in pending_changes HashMap using analyze_impact symbol_name:pending_changes

## 9. Code Map Table

| Component        | File                 | Line  | Purpose              |
|------------------|----------------------|-------|----------------------|
| IndexWatcher | `src/mcp/watcher.rs` | 20 | Monitor index file changes |
| check_and_reload | `src/mcp/watcher.rs` | 92 | Check and reload index |
| check_and_reindex_source_files | `src/mcp/watcher.rs` | 156 | Re-index changed source files |
| FileSystemWatcher | `src/indexing/fs_watcher.rs` | 57 | Monitor indexed file changes |
| watch | `src/indexing/fs_watcher.rs` | 148 | Main event loop |
| compute_watch_dirs | `src/indexing/fs_watcher.rs` | 444 | Calculate minimal directory set |
| NotificationBroadcaster | `src/mcp/notifications.rs` | 19 | Broadcast file change events |
| FileChangeEvent | `src/mcp/notifications.rs` | 10 | Event type enum |
| serve_http | `src/mcp/http_server.rs` | 7 | HTTP server with watchers |
| Commands::Serve | `src/main.rs` | 644 | CLI serve command handler |

## 10. Confidence and Limitations

- Watcher architecture: High (direct code evidence)
- Event flow: High (traced through implementation)
- Timing calculations: Medium (based on defaults, not runtime measurements)
- Memory usage: Low (estimates only, no profiling data)
- Unknown: Actual performance under high file churn rates

## 11. Footer

GeneratedAt=September 07, 2025 at 10:03 PM  Model=claude-opus-4-1-20250805