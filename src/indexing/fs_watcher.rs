//! File system watcher for automatic re-indexing of changed files
//!
//! This module implements the "watch what you indexed" philosophy:
//! - Only watches files that are already in the index
//! - Ignores all other files, even in watched directories
//! - No auto-indexing of new files (by design)

use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, sleep};

use crate::mcp::notifications::{FileChangeEvent, NotificationBroadcaster};
use crate::{IndexError, IndexResult, SimpleIndexer};

/// Errors specific to file watching operations
#[derive(Error, Debug)]
pub enum FileWatchError {
    #[error(
        "Failed to initialize file watcher: {reason}\nSuggestion: Check file system permissions and ensure the notify crate is properly installed"
    )]
    WatcherInitFailed { reason: String },

    #[error(
        "Cannot watch path {path:?}: {reason}\nSuggestion: Verify the path exists and you have read permissions"
    )]
    PathWatchFailed { path: PathBuf, reason: String },

    #[error(
        "File system event error: {details}\nSuggestion: Check disk space and file system health"
    )]
    EventError { details: String },

    #[error(
        "Failed to query indexed files from database\nSuggestion: Ensure the index is properly initialized before starting the watcher"
    )]
    IndexQueryFailed,

    #[error(
        "Failed to re-index file {path:?}: {reason}\nSuggestion: Check if the file still exists and is readable"
    )]
    ReindexFailed { path: PathBuf, reason: String },
}

/// Watches ONLY the files that are in the index for changes
///
/// Key behavior:
/// - Queries the index to determine what files to watch
/// - Watches parent directories but only processes events for indexed files
/// - Never auto-indexes new files
/// - Re-indexes modified files using existing hash comparison
/// - Debounces rapid changes to prevent excessive re-indexing
pub struct FileSystemWatcher {
    /// Reference to the indexer (shared with MCP server)
    indexer: Arc<RwLock<SimpleIndexer>>,
    /// How long to wait before processing changes (milliseconds)
    debounce_ms: u64,
    /// Channel receiver for file events
    event_rx: mpsc::Receiver<notify::Result<Event>>,
    /// The actual file watcher (kept alive by storing it)
    _watcher: notify::RecommendedWatcher,
    /// MCP debug flag for controlling verbosity
    mcp_debug: bool,
    /// Optional notification broadcaster for MCP notifications
    broadcaster: Option<Arc<NotificationBroadcaster>>,
    /// Index path for semantic search persistence
    index_path: PathBuf,
}

impl FileSystemWatcher {
    /// Create a new watcher that will watch ONLY indexed files
    ///
    /// # Arguments
    /// * `indexer` - Shared reference to the indexer
    /// * `debounce_ms` - Milliseconds to wait before processing changes (for batching)
    /// * `mcp_debug` - Debug flag for verbose output
    /// * `index_path` - Path to the index directory for semantic search persistence
    ///
    /// # Returns
    /// A configured file watcher ready to start watching
    pub fn new(
        indexer: Arc<RwLock<SimpleIndexer>>,
        debounce_ms: u64,
        mcp_debug: bool,
        index_path: &Path,
    ) -> IndexResult<Self> {
        // Create channel for events with reasonable buffer
        let (tx, rx) = mpsc::channel(100);

        // Create the notify watcher with our channel
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            // Send events to our async channel
            // We use blocking_send because this callback is sync
            let _ = tx.blocking_send(res);
        })
        .map_err(|e| {
            IndexError::General(format!(
                "Failed to create file watcher: {e}. Check system resources"
            ))
        })?;

        Ok(Self {
            indexer,
            debounce_ms,
            event_rx: rx,
            _watcher: watcher,
            mcp_debug,
            broadcaster: None,
            index_path: index_path.to_path_buf(),
        })
    }

    /// Set the notification broadcaster
    pub fn with_broadcaster(mut self, broadcaster: Arc<NotificationBroadcaster>) -> Self {
        self.broadcaster = Some(broadcaster);
        self
    }

    /// Get the list of files that are currently indexed
    /// This is the KEY method - we ONLY watch these files
    async fn get_indexed_paths(&self) -> Vec<PathBuf> {
        let indexer = self.indexer.read().await;
        let paths = indexer.get_all_indexed_paths();

        if paths.is_empty() {
            eprintln!("No indexed files found in the index");
        } else {
            eprintln!("Found {} indexed files to watch", paths.len());
            // Show detailed file list only when mcp_debug is true
            if self.mcp_debug {
                for (i, path) in paths.iter().take(3).enumerate() {
                    eprintln!("  [{}] {}", i + 1, path.display());
                }
                if paths.len() > 3 {
                    eprintln!("  ... and {} more", paths.len() - 3);
                }
            }
        }

        paths
    }

    /// Start watching the indexed files for changes
    ///
    /// This method:
    /// 1. Queries the index for all indexed files
    /// 2. Determines parent directories to watch
    /// 3. Sets up watching on those directories
    /// 4. Processes events ONLY for indexed files with debouncing
    pub async fn watch(mut self) -> IndexResult<()> {
        // 1. Get list of indexed files from the index
        let indexed_paths = self.get_indexed_paths().await;

        if indexed_paths.is_empty() {
            eprintln!("Warning: No indexed files found. File watcher has nothing to watch.");
            eprintln!("Run 'codanna index <path>' first to index some files.");
            // Still continue - maybe files will be indexed later
        } else {
            eprintln!(
                "File watcher: Monitoring {} indexed files for changes",
                indexed_paths.len()
            );
        }

        // Get workspace root to convert relative paths to absolute
        let workspace_root = {
            let indexer = self.indexer.read().await;
            indexer
                .settings()
                .workspace_root
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        };

        // 2. Compute which directories to watch (parent dirs of indexed files)
        let watch_dirs = Self::compute_watch_dirs(&indexed_paths);

        if !watch_dirs.is_empty() {
            eprintln!(
                "Watching {} directories containing indexed files",
                watch_dirs.len()
            );
        }

        // 3. Start watching those directories
        for dir in &watch_dirs {
            // Convert relative paths to absolute for the watcher
            let watch_path = if dir.is_absolute() {
                dir.clone()
            } else {
                workspace_root.join(dir)
            };

            match self
                ._watcher
                .watch(&watch_path, RecursiveMode::NonRecursive)
            {
                Ok(_) => {
                    if self.mcp_debug {
                        eprintln!("  Watching: {}", watch_path.display());
                    }
                }
                Err(e) => {
                    eprintln!("  Warning: Failed to watch {}: {}", watch_path.display(), e);
                    // Continue with other directories
                }
            }
        }

        // 4. Convert paths to absolute paths in HashSet for efficient lookup
        // The notify crate gives us absolute paths, but our index stores relative paths
        let mut indexed_set: HashSet<PathBuf> = indexed_paths
            .into_iter()
            .map(|p| {
                if p.is_absolute() {
                    p
                } else {
                    workspace_root.join(&p)
                }
            })
            .collect();

        // 5. Set up debouncing state
        let mut pending_changes: HashMap<PathBuf, Instant> = HashMap::new();
        let debounce_duration = Duration::from_millis(self.debounce_ms);

        // 6. Subscribe to broadcast notifications if broadcaster is available
        let mut broadcast_receiver = self.broadcaster.as_ref().map(|b| b.subscribe());
        if broadcast_receiver.is_some() {
            eprintln!("File watcher subscribed to index reload notifications");
        }

        // 7. Event handling loop with debouncing
        eprintln!("File watcher started. Press Ctrl+C to stop.");

        loop {
            // Use timeout to periodically process pending changes
            let timeout = sleep(Duration::from_millis(100));
            tokio::pin!(timeout);

            tokio::select! {
                // Handle incoming file events
                Some(res) = self.event_rx.recv() => {
                    match res {
                        Ok(event) => {
                            // Handle different event types for indexed files
                            for path in &event.paths {
                                if indexed_set.contains(path) {
                                    match event.kind {
                                        EventKind::Modify(_) => {
                                            // Record this change with current timestamp
                                            pending_changes.insert(path.clone(), Instant::now());
                                        }
                                        EventKind::Remove(_) => {
                                            // File was deleted - remove it from index immediately
                                            let path_display = path.display();
                                            eprintln!("Detected deletion of indexed file: {path_display}");
                                            eprintln!("  Removing from index...");

                                            // Convert absolute path to relative path for the index
                                            let relative_path = if path.is_absolute() {
                                                if let Ok(cwd) = std::env::current_dir() {
                                                    match path.strip_prefix(&cwd) {
                                                        Ok(rel) => rel.to_path_buf(),
                                                        Err(_) => path.clone(),
                                                    }
                                                } else {
                                                    path.clone()
                                                }
                                            } else {
                                                path.clone()
                                            };

                                            let relative_display = relative_path.display();
                                            eprintln!("  Using relative path: {relative_display}");

                                            let mut indexer = self.indexer.write().await;
                                            if let Err(e) = indexer.remove_file(&relative_path) {
                                                eprintln!("  ✗ Failed to remove from index: {e}");
                                            } else {
                                                eprintln!("  ✓ Removed from index successfully");

                                                // Send notification to MCP clients
                                                if let Some(ref broadcaster) = self.broadcaster {
                                                    if self.mcp_debug {
                                                        eprintln!("DEBUG: Sending FileDeleted notification for: {}", path.display());
                                                    }
                                                    broadcaster.send(FileChangeEvent::FileDeleted {
                                                        path: path.clone(),
                                                    });
                                                }
                                            }
                                        }
                                        _ => {} // Ignore other event types
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("File watch error: {e}");
                        }
                    }
                }

                // Periodically check for changes that have passed debounce period
                _ = &mut timeout => {
                    let now = Instant::now();
                    let mut files_to_process = Vec::new();

                    // Find files that have been stable for the debounce period
                    pending_changes.retain(|path, last_change| {
                        if now.duration_since(*last_change) >= debounce_duration {
                            files_to_process.push(path.clone());
                            false // Remove from pending
                        } else {
                            true // Keep in pending
                        }
                    });

                    // Process debounced files
                    for path in files_to_process {
                        eprintln!("Detected change in indexed file: {}", path.display());
                        eprintln!("  Re-indexing...");

                        eprintln!("  Using absolute path for file reading: {}", path.display());

                        let mut indexer = self.indexer.write().await;
                        match indexer.index_file(&path) {
                            Ok(result) => {
                                use crate::IndexingResult;
                                match result {
                                    IndexingResult::Indexed(_) => {
                                        eprintln!("  ✓ Re-indexed successfully (file updated)");

                                        // CRITICAL: Save semantic search data after re-indexing
                                        if indexer.has_semantic_search() {
                                            let semantic_path = self.index_path.join("semantic");
                                            if let Err(e) = indexer.save_semantic_search(&semantic_path) {
                                                eprintln!("  ✗ Failed to save semantic search after re-indexing: {e}");
                                            } else {
                                                eprintln!("  ✓ Semantic search saved successfully");
                                            }
                                        }

                                        // Send notification if broadcaster is available
                                        if let Some(ref broadcaster) = self.broadcaster {
                                            if self.mcp_debug {
                                                eprintln!("DEBUG: FileSystemWatcher sending notification for: {}", path.display());
                                            }
                                            broadcaster.send(FileChangeEvent::FileReindexed {
                                                path: path.clone(),
                                            });
                                        } else if self.mcp_debug {
                                            eprintln!("DEBUG: No broadcaster available to send notification");
                                        }
                                    }
                                    IndexingResult::Cached(_) => {
                                        eprintln!("  ✓ File unchanged (hash match, skipped)");
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("  ✗ Re-index failed: {e}");
                            }
                        }
                    }
                }

                // Handle broadcast notifications for index reloads
                Some(event) = async {
                    match &mut broadcast_receiver {
                        Some(rx) => rx.recv().await.ok(),
                        None => None,
                    }
                } => {
                    match event {
                        FileChangeEvent::IndexReloaded => {
                            eprintln!("File watcher received IndexReloaded notification");
                            eprintln!("  Refreshing watched file list...");

                            // Get the updated list of indexed files
                            let new_indexed_paths = self.get_indexed_paths().await;

                            // Convert to absolute paths
                            let new_indexed_set: HashSet<PathBuf> = new_indexed_paths
                                .into_iter()
                                .map(|p| {
                                    if p.is_absolute() {
                                        p
                                    } else {
                                        workspace_root.join(&p)
                                    }
                                })
                                .collect();

                            // Calculate differences
                            let added: Vec<_> = new_indexed_set.difference(&indexed_set).cloned().collect();
                            let removed: Vec<_> = indexed_set.difference(&new_indexed_set).cloned().collect();

                            if !added.is_empty() {
                                let added_count = added.len();
                                eprintln!("  Added {added_count} new files to watch");
                                if self.mcp_debug {
                                    for path in &added {
                                        let path_display = path.display();
                                        eprintln!("    + {path_display}");
                                    }
                                }

                                // Watch new directories if needed
                                let new_dirs = Self::compute_watch_dirs(&added);
                                for watch_path in new_dirs {
                                    if let Err(e) = self._watcher.watch(&watch_path, RecursiveMode::NonRecursive) {
                                        let watch_display = watch_path.display();
                                        eprintln!("  Warning: Failed to watch {watch_display}: {e}");
                                    }
                                }
                            }

                            if !removed.is_empty() {
                                let removed_count = removed.len();
                                eprintln!("  Removed {removed_count} files from watch");
                                if self.mcp_debug {
                                    for path in &removed {
                                        let path_display = path.display();
                                        eprintln!("    - {path_display}");
                                    }
                                }
                            }

                            // Update the indexed set
                            indexed_set = new_indexed_set;
                            let total_files = indexed_set.len();
                            eprintln!("  ✓ Now watching {total_files} files");
                        }
                        _ => {
                            // Ignore other event types
                        }
                    }
                }

                else => {
                    // Channel closed, exit
                    break;
                }
            }
        }

        Ok(())
    }

    /// Compute minimal set of directories to watch
    ///
    /// Given a list of file paths, returns the unique parent directories
    /// This is more efficient than watching individual files
    fn compute_watch_dirs(paths: &[PathBuf]) -> HashSet<PathBuf> {
        let mut dirs = HashSet::new();

        for path in paths {
            if let Some(parent) = path.parent() {
                // Handle root-level files (parent is empty string)
                if parent.as_os_str().is_empty() {
                    // Watch current directory for root-level files
                    dirs.insert(PathBuf::from("."));
                } else {
                    dirs.insert(parent.to_path_buf());
                }
            }
        }

        dirs
    }
}

// Re-export the error type for convenience
pub use FileWatchError as WatchError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleIndexer;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_system_watcher_creation() {
        println!("\n=== TEST: FileSystemWatcher Creation and Initialization ===");

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        // Create indexer with a temporary index location
        println!("Step 1: Creating indexer and indexing test file...");
        let index_dir = temp_dir.path().join(".test_index");
        let settings = crate::Settings {
            index_path: index_dir.clone(),
            ..Default::default()
        };
        let mut indexer = SimpleIndexer::with_settings(Arc::new(settings));
        let result = indexer.index_file(&test_file);
        assert!(result.is_ok());
        println!("  ✓ Indexed test file: {}", test_file.display());

        // Verify the file is in the index
        let paths = indexer.get_all_indexed_paths();
        println!("  ✓ Indexer reports {} files", paths.len());
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], test_file);

        // Create the watcher
        println!("\nStep 2: Creating FileSystemWatcher...");
        let indexer_arc = Arc::new(RwLock::new(indexer));
        let index_path = PathBuf::from(".codanna/index");
        let watcher = FileSystemWatcher::new(indexer_arc.clone(), 500, false, &index_path);

        assert!(watcher.is_ok());
        let _watcher = watcher.unwrap();
        println!("  ✓ FileSystemWatcher created successfully");
        println!("  - Debounce: 500ms");

        // Test that get_indexed_paths works (indirectly via the indexer)
        println!("\nStep 3: Verifying watcher has access to indexed files...");
        let indexer_lock = indexer_arc.read().await;
        let watcher_paths = indexer_lock.get_all_indexed_paths();
        println!("  ✓ Watcher's indexer sees {} files", watcher_paths.len());
        assert_eq!(watcher_paths.len(), 1);
        assert_eq!(watcher_paths[0], test_file);

        // Test compute_watch_dirs
        println!("\nStep 4: Testing compute_watch_dirs()...");
        let watch_dirs = FileSystemWatcher::compute_watch_dirs(&watcher_paths);
        println!("  Computed {} watch directories", watch_dirs.len());
        assert_eq!(watch_dirs.len(), 1);
        assert!(watch_dirs.contains(temp_dir.path()));
        println!(
            "  ✓ Correctly computed parent directory: {}",
            temp_dir.path().display()
        );

        println!("\n=== TEST PASSED: FileSystemWatcher initialization works ===");
    }

    #[tokio::test]
    async fn test_get_indexed_paths() {
        println!("\n=== TEST: FileSystemWatcher::get_indexed_paths() ===");

        // Create temp directory and multiple test files
        let temp_dir = TempDir::new().unwrap();
        let test_files = vec![
            temp_dir.path().join("main.rs"),
            temp_dir.path().join("lib.rs"),
            temp_dir.path().join("mod.rs"),
        ];

        for file in &test_files {
            fs::write(file, "// test file").unwrap();
        }

        // Create indexer with a temporary index location
        println!("Step 1: Indexing {} test files...", test_files.len());
        let index_dir = temp_dir.path().join(".test_index");
        let settings = crate::Settings {
            index_path: index_dir.clone(),
            ..Default::default()
        };
        let mut indexer = SimpleIndexer::with_settings(Arc::new(settings));
        for file in &test_files {
            indexer.index_file(file).unwrap();
            println!(
                "  - Indexed: {}",
                file.file_name().unwrap().to_string_lossy()
            );
        }

        // Create watcher
        println!("\nStep 2: Creating FileSystemWatcher...");
        let indexer_arc = Arc::new(RwLock::new(indexer));
        let index_path = PathBuf::from(".codanna/index");
        let watcher = FileSystemWatcher::new(indexer_arc.clone(), 100, false, &index_path).unwrap();

        // Call get_indexed_paths (private method, but we can test via the indexer)
        println!("\nStep 3: Testing get_indexed_paths() behavior...");
        let paths = watcher.get_indexed_paths().await;

        println!("  Retrieved {} paths from watcher:", paths.len());
        for (i, path) in paths.iter().enumerate() {
            println!(
                "    [{}] {}",
                i + 1,
                path.file_name().unwrap().to_string_lossy()
            );
        }

        assert_eq!(paths.len(), test_files.len());
        println!("  ✓ Correct number of paths retrieved");

        // Verify all files are present
        for test_file in &test_files {
            assert!(paths.contains(test_file), "Missing file: {test_file:?}");
        }
        println!("  ✓ All indexed files are present");

        println!("\n=== TEST PASSED: get_indexed_paths() works correctly ===");
    }

    #[test]
    fn test_compute_watch_dirs() {
        println!("\n=== TEST: compute_watch_dirs() ===");

        let paths = vec![
            PathBuf::from("/project/src/main.rs"),
            PathBuf::from("/project/src/lib.rs"),
            PathBuf::from("/project/tests/test1.rs"),
            PathBuf::from("/project/tests/test2.rs"),
            PathBuf::from("/project/benches/bench.rs"),
        ];

        println!("Input paths:");
        for path in &paths {
            println!("  - {}", path.display());
        }

        let dirs = FileSystemWatcher::compute_watch_dirs(&paths);

        println!("\nComputed watch directories:");
        for dir in &dirs {
            println!("  - {}", dir.display());
        }

        assert_eq!(dirs.len(), 3);
        assert!(dirs.contains(&PathBuf::from("/project/src")));
        assert!(dirs.contains(&PathBuf::from("/project/tests")));
        assert!(dirs.contains(&PathBuf::from("/project/benches")));

        println!(
            "\n✓ Correctly computed {} unique parent directories",
            dirs.len()
        );
        println!("=== TEST PASSED ===");
    }
}
