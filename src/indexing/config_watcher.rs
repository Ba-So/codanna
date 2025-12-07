//! Configuration file watcher for automatic reindexing when config changes
//!
//! Watches the settings.toml file for changes and triggers reindexing
//! when indexed_paths are added or removed.

use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use crate::config::Settings;
use crate::mcp::notifications::{FileChangeEvent, NotificationBroadcaster};
use crate::{IndexError, IndexResult, SimpleIndexer};

/// Watches the configuration file for changes to indexed_paths
///
/// When indexed_paths are added:
/// - Indexes new directories
/// - Sends IndexReloaded notification to update watchers
///
/// When indexed_paths are removed:
/// - Cleanup happens via next index command
/// - Sends IndexReloaded notification
pub struct ConfigFileWatcher {
    /// Path to settings.toml
    settings_path: PathBuf,
    /// Reference to the indexer (shared with MCP server)
    indexer: Arc<RwLock<SimpleIndexer>>,
    /// Optional notification broadcaster
    broadcaster: Option<Arc<NotificationBroadcaster>>,
    /// Last known indexed paths
    last_indexed_paths: HashSet<PathBuf>,
    /// MCP debug flag
    mcp_debug: bool,
    /// Channel receiver for file events
    event_rx: mpsc::Receiver<notify::Result<Event>>,
    /// The actual file watcher
    _watcher: notify::RecommendedWatcher,
}

impl ConfigFileWatcher {
    /// Create a new config watcher
    pub fn new(
        settings_path: PathBuf,
        indexer: Arc<RwLock<SimpleIndexer>>,
        mcp_debug: bool,
    ) -> IndexResult<Self> {
        // Create channel for events
        let (tx, rx) = mpsc::channel(10);

        // Create the notify watcher
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            let _ = tx.blocking_send(res);
        })
        .map_err(|e| IndexError::FileRead {
            path: settings_path.clone(),
            source: std::io::Error::other(e.to_string()),
        })?;

        // Load initial indexed_paths
        let initial_config =
            Settings::load_from(&settings_path).map_err(|e| IndexError::ConfigError {
                reason: format!("Failed to load config: {e}"),
            })?;
        let last_indexed_paths: HashSet<_> =
            initial_config.indexing.indexed_paths.into_iter().collect();

        Ok(Self {
            settings_path,
            indexer,
            broadcaster: None,
            last_indexed_paths,
            mcp_debug,
            event_rx: rx,
            _watcher: watcher,
        })
    }

    /// Set the notification broadcaster
    pub fn with_broadcaster(mut self, broadcaster: Arc<NotificationBroadcaster>) -> Self {
        self.broadcaster = Some(broadcaster);
        self
    }

    /// Start watching the configuration file
    pub async fn watch(mut self) -> IndexResult<()> {
        // Watch the settings file's parent directory (watching the file directly can be unreliable)
        let watch_dir = self
            .settings_path
            .parent()
            .ok_or_else(|| IndexError::FileRead {
                path: self.settings_path.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Config file has no parent directory",
                ),
            })?;

        self._watcher
            .watch(watch_dir, RecursiveMode::NonRecursive)
            .map_err(|e| IndexError::FileRead {
                path: watch_dir.to_path_buf(),
                source: std::io::Error::other(e.to_string()),
            })?;

        eprintln!(
            "Config watcher: Monitoring {}",
            self.settings_path.display()
        );

        // Check for any pending changes on startup (config modified while server was down)
        if let Err(e) = self.check_initial_sync().await {
            eprintln!("Warning: Initial config sync failed: {e}");
        }

        // Event loop
        loop {
            if let Some(res) = self.event_rx.recv().await {
                match res {
                    Ok(event) => {
                        // Only process events for our settings file
                        if event.paths.iter().any(|p| p == &self.settings_path) {
                            match event.kind {
                                EventKind::Modify(_) | EventKind::Create(_) => {
                                    if let Err(e) = self.handle_config_change().await {
                                        eprintln!("Config watcher error: {e}");
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Config watch error: {e}");
                    }
                }
            }
        }
    }

    /// Check if config has changes that need syncing on startup
    async fn check_initial_sync(&mut self) -> IndexResult<()> {
        if self.mcp_debug {
            eprintln!("DEBUG: Checking for pending config changes on startup");
        }

        // Get current indexed paths from the indexer
        let indexer = self.indexer.read().await;
        let currently_indexed: HashSet<PathBuf> =
            indexer.get_indexed_paths().iter().cloned().collect();
        drop(indexer); // Release read lock

        // Find paths in config that aren't indexed yet
        let added: Vec<_> = self
            .last_indexed_paths
            .difference(&currently_indexed)
            .cloned()
            .collect();

        if added.is_empty() {
            if self.mcp_debug {
                eprintln!("DEBUG: No pending config changes detected");
            }
            return Ok(());
        }

        eprintln!(
            "Initial sync: Found {} new directories to index",
            added.len()
        );
        for path in &added {
            eprintln!("  + {}", path.display());
        }

        // Index new directories
        let mut indexer = self.indexer.write().await;
        for path in &added {
            eprintln!("Indexing new directory: {}", path.display());
            match indexer.index_directory(path, false, false) {
                Ok(stats) => {
                    eprintln!(
                        "  ✓ Indexed {} files, {} symbols",
                        stats.files_indexed, stats.symbols_found
                    );
                }
                Err(e) => {
                    eprintln!("  ✗ Failed to index {}: {e}", path.display());
                }
            }
        }
        drop(indexer); // Release write lock

        // Send notification to update file watcher and MCP clients
        if let Some(ref broadcaster) = self.broadcaster {
            if self.mcp_debug {
                eprintln!("DEBUG: Sending IndexReloaded notification");
            }
            broadcaster.send(FileChangeEvent::IndexReloaded);
            eprintln!("  ✓ Notified watchers of index changes");
        }

        Ok(())
    }

    /// Handle configuration file change
    async fn handle_config_change(&mut self) -> IndexResult<()> {
        if self.mcp_debug {
            eprintln!("DEBUG: Config file changed, checking indexed_paths");
        }

        // Small delay to ensure file write is complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Reload config
        let new_config =
            Settings::load_from(&self.settings_path).map_err(|e| IndexError::ConfigError {
                reason: format!("Failed to reload config: {e}"),
            })?;
        let new_paths: HashSet<PathBuf> = new_config.indexing.indexed_paths.into_iter().collect();

        // Check if indexed_paths changed
        if new_paths == self.last_indexed_paths {
            if self.mcp_debug {
                eprintln!("DEBUG: indexed_paths unchanged, ignoring");
            }
            return Ok(());
        }

        eprintln!("Config change detected: indexed_paths modified");

        // Find added and removed paths
        let added: Vec<_> = new_paths
            .difference(&self.last_indexed_paths)
            .cloned()
            .collect();
        let removed: Vec<_> = self
            .last_indexed_paths
            .difference(&new_paths)
            .cloned()
            .collect();

        if !added.is_empty() {
            eprintln!("New directories to index: {}", added.len());
            for path in &added {
                eprintln!("  + {}", path.display());
            }

            // Index new directories
            let mut indexer = self.indexer.write().await;
            for path in &added {
                eprintln!("Indexing new directory: {}", path.display());
                match indexer.index_directory(path, false, false) {
                    Ok(stats) => {
                        eprintln!(
                            "  ✓ Indexed {} files, {} symbols",
                            stats.files_indexed, stats.symbols_found
                        );
                    }
                    Err(e) => {
                        eprintln!("  ✗ Failed to index {}: {e}", path.display());
                    }
                }
            }
        }

        if !removed.is_empty() {
            eprintln!("Directories removed from config: {}", removed.len());
            for path in &removed {
                eprintln!("  - {}", path.display());
            }
            eprintln!(
                "Run 'codanna clean' or 'codanna index' to remove symbols from these directories"
            );
        }

        // Update tracked paths
        self.last_indexed_paths = new_paths;

        // Send notification to update file watcher and MCP clients
        if let Some(ref broadcaster) = self.broadcaster {
            if self.mcp_debug {
                eprintln!("DEBUG: Sending IndexReloaded notification");
            }
            broadcaster.send(FileChangeEvent::IndexReloaded);
            eprintln!("  ✓ Notified watchers of index changes");
        }

        Ok(())
    }
}
