//! Simplified persistence layer for Tantivy-only storage
//!
//! This module manages metadata and ensures Tantivy index exists.
//! All actual data is stored in Tantivy.

use crate::storage::{DataSource, IndexMetadata};
use crate::{IndexError, IndexResult, Settings, SimpleIndexer};
use std::path::PathBuf;
use std::sync::Arc;

/// Manages persistence of the index
#[derive(Debug)]
pub struct IndexPersistence {
    base_path: PathBuf,
}

impl IndexPersistence {
    /// Create a new persistence manager
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Get path for semantic search data
    fn semantic_path(&self) -> PathBuf {
        self.base_path.join("semantic")
    }

    /// Save metadata for the index
    #[must_use = "Save errors should be handled to ensure data is persisted"]
    pub fn save(&self, indexer: &SimpleIndexer) -> IndexResult<()> {
        // Update metadata
        let mut metadata =
            IndexMetadata::load(&self.base_path).unwrap_or_else(|_| IndexMetadata::new());

        metadata.update_counts(indexer.symbol_count() as u32, indexer.file_count());

        // Update metadata to reflect Tantivy
        metadata.data_source = DataSource::Tantivy {
            path: self.base_path.join("tantivy"),
            doc_count: indexer.document_count().unwrap_or(0),
            timestamp: crate::indexing::get_utc_timestamp(),
        };

        metadata.save(&self.base_path)?;

        // Update project registry with latest metadata
        self.update_project_registry(&metadata)?;

        // Save semantic search if enabled
        if indexer.has_semantic_search() {
            let semantic_path = self.semantic_path();
            std::fs::create_dir_all(&semantic_path).map_err(|e| {
                IndexError::General(format!("Failed to create semantic directory: {e}"))
            })?;

            indexer
                .save_semantic_search(&semantic_path)
                .map_err(|e| IndexError::General(format!("Failed to save semantic search: {e}")))?;
        }

        Ok(())
    }

    /// Load the indexer from disk
    #[must_use = "Load errors should be handled appropriately"]
    pub fn load(&self) -> IndexResult<SimpleIndexer> {
        self.load_with_settings(Arc::new(Settings::default()), false)
    }

    /// Load the indexer from disk with custom settings
    #[must_use = "Load errors should be handled appropriately"]
    pub fn load_with_settings(
        &self,
        settings: Arc<Settings>,
        info: bool,
    ) -> IndexResult<SimpleIndexer> {
        self.load_with_settings_lazy(settings, info, false)
    }

    /// Load the indexer from disk with custom settings and lazy initialization options
    #[must_use = "Load errors should be handled appropriately"]
    pub fn load_with_settings_lazy(
        &self,
        settings: Arc<Settings>,
        info: bool,
        skip_trait_resolver: bool,
    ) -> IndexResult<SimpleIndexer> {
        // Load metadata to understand data sources
        let metadata = IndexMetadata::load(&self.base_path).ok();

        // Check if Tantivy index exists
        let tantivy_path = self.base_path.join("tantivy");
        if tantivy_path.join("meta.json").exists() {
            // Extract debug flag before moving settings
            let debug = settings.debug;

            // Create indexer that will load from Tantivy
            // Note: skip_trait_resolver no longer needed - behaviors handle resolution now
            let mut indexer = if skip_trait_resolver {
                SimpleIndexer::with_settings_lazy(settings)
            } else {
                SimpleIndexer::with_settings(settings)
            };

            // Display source info with fresh counts
            if let Some(meta) = metadata {
                // Get fresh counts from the actual index
                let fresh_symbol_count = indexer.symbol_count();
                let fresh_file_count = indexer.file_count();

                // Display the metadata but with fresh counts
                if info {
                    match &meta.data_source {
                        DataSource::Tantivy {
                            path, doc_count, ..
                        } => {
                            eprintln!(
                                "Loaded from Tantivy index: {} ({} documents)",
                                path.display(),
                                doc_count
                            );
                        }
                        DataSource::Fresh => {
                            eprintln!("Created fresh index");
                        }
                    }
                    eprintln!(
                        "Index contains {fresh_symbol_count} symbols from {fresh_file_count} files"
                    );
                }
            }

            // NEW: Always try to load semantic search - let the actual load determine if data exists
            // This is more robust than checking filesystem paths which can fail due to resolution issues
            let semantic_path = self.semantic_path();
            if info || debug {
                eprintln!("DEBUG: Persistence base_path: {}", self.base_path.display());
                eprintln!(
                    "DEBUG: Semantic path computed as: {}",
                    semantic_path.display()
                );
            }
            match indexer.load_semantic_search(&semantic_path, info) {
                Ok(true) => {
                    // Successfully loaded (message already printed by load_semantic_search)
                }
                Ok(false) => {
                    // No semantic data found - this is fine, semantic search is optional
                    if info || debug {
                        eprintln!("DEBUG: No semantic data found (this is optional)");
                    }
                }
                Err(e) => {
                    // Log error but continue - semantic search is optional
                    eprintln!("Warning: Failed to load semantic search: {e}");
                }
            }

            Ok(indexer)
        } else {
            Err(IndexError::FileRead {
                path: tantivy_path,
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Tantivy index not found",
                ),
            })
        }
    }

    /// Check if an index exists
    pub fn exists(&self) -> bool {
        // Check if Tantivy index exists
        let tantivy_path = self.base_path.join("tantivy");
        tantivy_path.join("meta.json").exists()
    }

    /// Delete the persisted index
    pub fn clear(&self) -> Result<(), std::io::Error> {
        let tantivy_path = self.base_path.join("tantivy");
        if tantivy_path.exists() {
            std::fs::remove_dir_all(tantivy_path)?;
        }
        Ok(())
    }

    /// Update the project registry with latest metadata
    fn update_project_registry(&self, metadata: &IndexMetadata) -> IndexResult<()> {
        // Try to read the project ID file
        let local_dir = crate::init::local_dir_name();
        let project_id_path = PathBuf::from(local_dir).join(".project-id");

        if !project_id_path.exists() {
            // No project ID file means project wasn't registered during init
            // This is fine for legacy projects
            return Ok(());
        }

        let project_id =
            std::fs::read_to_string(&project_id_path).map_err(|e| IndexError::FileRead {
                path: project_id_path.clone(),
                source: e,
            })?;

        // Load the registry
        let mut registry = crate::init::ProjectRegistry::load()
            .map_err(|e| IndexError::General(format!("Failed to load project registry: {e}")))?;

        // Update the project metadata
        if let Some(project) = registry.find_project_by_id_mut(&project_id) {
            project.symbol_count = metadata.symbol_count;
            project.file_count = metadata.file_count;
            project.last_modified = metadata.last_modified;

            // Get doc count from data source
            if let DataSource::Tantivy { doc_count, .. } = &metadata.data_source {
                project.doc_count = *doc_count;
            }

            // Save the updated registry
            registry.save().map_err(|e| {
                IndexError::General(format!("Failed to save project registry: {e}"))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    /// Check if semantic data exists (test helper)
    fn has_semantic_data(persistence: &IndexPersistence) -> bool {
        // Check if metadata exists - that's the definitive indicator
        persistence.semantic_path().join("metadata.json").exists()
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();

        // Create a custom settings with temp_dir as the index path
        let settings = Settings {
            index_path: temp_dir.path().to_path_buf(),
            ..Settings::default()
        };

        let persistence = IndexPersistence::new(temp_dir.path().to_path_buf());

        // Create required directories for the indexer
        std::fs::create_dir_all(temp_dir.path().join("tantivy")).unwrap();

        // Create an indexer with custom settings
        let indexer = SimpleIndexer::with_settings(Arc::new(settings));

        // Save it
        persistence.save(&indexer).unwrap();

        // Check metadata exists
        let metadata_path = temp_dir.path().join("index.meta");
        assert!(metadata_path.exists());
    }

    #[test]
    fn test_exists() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path().to_path_buf());

        // Initially doesn't exist
        assert!(!persistence.exists());

        // Create tantivy directory with meta.json
        let tantivy_path = temp_dir.path().join("tantivy");
        std::fs::create_dir_all(&tantivy_path).unwrap();
        std::fs::write(tantivy_path.join("meta.json"), "{}").unwrap();

        // Now it exists
        assert!(persistence.exists());
    }

    #[test]
    fn test_semantic_paths() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path().to_path_buf());

        // Test semantic_path
        let semantic_path = persistence.semantic_path();
        assert_eq!(semantic_path, temp_dir.path().join("semantic"));

        // Initially has no semantic data
        assert!(!has_semantic_data(&persistence));

        // Create semantic directory and metadata file
        std::fs::create_dir_all(&semantic_path).unwrap();
        std::fs::write(semantic_path.join("metadata.json"), "{}").unwrap();

        // Now has semantic data
        assert!(has_semantic_data(&persistence));
    }
}
