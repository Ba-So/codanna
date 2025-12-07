//! Hash-based memory-mapped symbol cache for zero-overhead lookups
//!
//! This module provides a fast, scalable symbol lookup system that bypasses
//! Tantivy for simple symbol name queries. It uses memory-mapped files with
//! hash-based buckets to achieve <10μs lookup times.

use crate::Symbol;
use crate::types::SymbolId;
use memmap2::{Mmap, MmapOptions};
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Magic bytes to identify symbol cache files
const MAGIC_BYTES: &[u8; 4] = b"SYMC";

/// Version of the cache format
const VERSION: u32 = 1;

/// Default number of hash buckets (power of 2 for fast modulo)
const DEFAULT_BUCKET_COUNT: usize = 256;

/// Header size in bytes
const HEADER_SIZE: usize = 32;

/// Maximum symbols per bucket before resize
#[allow(dead_code)]
const MAX_BUCKET_SIZE: usize = 1024;

/// FNV-1a hash function for good distribution
fn fnv1a_hash(data: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// A compact symbol entry for cache storage
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CacheEntry {
    symbol_id: u32,
    name_hash: u64, // Pre-computed hash of name for fast comparison
    file_id: u32,
    line: u32,
    column: u16,
    kind: u8,
    _padding: u8,
}

impl CacheEntry {
    const SIZE: usize = 24;

    fn from_symbol(symbol: &Symbol) -> Self {
        Self {
            symbol_id: symbol.id.value(),
            name_hash: fnv1a_hash(symbol.name.as_bytes()),
            file_id: symbol.file_id.value(),
            line: symbol.range.start_line,
            column: symbol.range.start_column,
            kind: symbol.kind as u8,
            _padding: 0,
        }
    }
}

/// Hash-based symbol cache with memory-mapped storage
pub struct SymbolHashCache {
    path: PathBuf,
    mmap: Option<Mmap>,
    bucket_count: usize,
    symbol_count: usize,
    bucket_offsets: Vec<u64>,
}

impl SymbolHashCache {
    /// Create a new symbol cache at the given path
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        Ok(Self {
            path,
            mmap: None,
            bucket_count: DEFAULT_BUCKET_COUNT,
            symbol_count: 0,
            bucket_offsets: Vec::new(),
        })
    }

    /// Open an existing cache file
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        // Read header
        if mmap.len() < HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Cache file too small",
            ));
        }

        // Validate magic bytes
        if &mmap[0..4] != MAGIC_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid cache file format",
            ));
        }

        // Read metadata
        let version = u32::from_le_bytes([mmap[4], mmap[5], mmap[6], mmap[7]]);
        if version != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported cache version: {version}"),
            ));
        }

        let bucket_count = u32::from_le_bytes([mmap[8], mmap[9], mmap[10], mmap[11]]) as usize;
        let symbol_count = u64::from_le_bytes([
            mmap[12], mmap[13], mmap[14], mmap[15], mmap[16], mmap[17], mmap[18], mmap[19],
        ]) as usize;

        // Read bucket offsets
        let mut bucket_offsets = Vec::with_capacity(bucket_count);
        let offset_start = HEADER_SIZE;
        for i in 0..bucket_count {
            let offset_pos = offset_start + i * 8;
            let offset = u64::from_le_bytes([
                mmap[offset_pos],
                mmap[offset_pos + 1],
                mmap[offset_pos + 2],
                mmap[offset_pos + 3],
                mmap[offset_pos + 4],
                mmap[offset_pos + 5],
                mmap[offset_pos + 6],
                mmap[offset_pos + 7],
            ]);
            bucket_offsets.push(offset);
        }

        Ok(Self {
            path,
            mmap: Some(mmap),
            bucket_count,
            symbol_count,
            bucket_offsets,
        })
    }

    /// Get the path to the cache file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the number of symbols in the cache
    pub fn symbol_count(&self) -> usize {
        self.symbol_count
    }

    /// Lookup a symbol by name (fast path)
    pub fn lookup_by_name(&self, name: &str) -> Option<SymbolId> {
        let mmap = self.mmap.as_ref()?;
        let name_hash = fnv1a_hash(name.as_bytes());
        let bucket_idx = (name_hash as usize) % self.bucket_count;

        // Get bucket boundaries
        let bucket_start = self.bucket_offsets[bucket_idx] as usize;
        let bucket_end = if bucket_idx + 1 < self.bucket_count {
            self.bucket_offsets[bucket_idx + 1] as usize
        } else {
            mmap.len()
        };

        // Scan bucket for matching hash
        let mut pos = bucket_start;

        // Read bucket entry count
        if pos + 4 > bucket_end {
            return None;
        }
        let entry_count =
            u32::from_le_bytes([mmap[pos], mmap[pos + 1], mmap[pos + 2], mmap[pos + 3]]) as usize;
        pos += 4;

        // Linear probe within bucket
        for _ in 0..entry_count {
            if pos + CacheEntry::SIZE > bucket_end {
                break;
            }

            // Read entry hash first (fast rejection)
            let entry_hash = u64::from_le_bytes([
                mmap[pos + 4],
                mmap[pos + 5],
                mmap[pos + 6],
                mmap[pos + 7],
                mmap[pos + 8],
                mmap[pos + 9],
                mmap[pos + 10],
                mmap[pos + 11],
            ]);

            if entry_hash == name_hash {
                // Hash matches, extract symbol ID
                let symbol_id =
                    u32::from_le_bytes([mmap[pos], mmap[pos + 1], mmap[pos + 2], mmap[pos + 3]]);
                return SymbolId::new(symbol_id);
            }

            pos += CacheEntry::SIZE;
        }

        None
    }

    /// Collect up to `max_candidates` symbol IDs whose name hash matches.
    /// This enables disambiguation at the call site without changing the cache format.
    pub fn lookup_candidates(&self, name: &str, max_candidates: usize) -> Vec<SymbolId> {
        let mut results = Vec::new();
        let Some(mmap) = self.mmap.as_ref() else {
            return results;
        };

        let name_hash = fnv1a_hash(name.as_bytes());
        let bucket_idx = (name_hash as usize) % self.bucket_count;

        // Get bucket boundaries
        let bucket_start = self.bucket_offsets[bucket_idx] as usize;
        let bucket_end = if bucket_idx + 1 < self.bucket_count {
            self.bucket_offsets[bucket_idx + 1] as usize
        } else {
            mmap.len()
        };

        // Scan bucket for matching hash
        let mut pos = bucket_start;

        // Read bucket entry count
        if pos + 4 > bucket_end {
            return results;
        }
        let entry_count =
            u32::from_le_bytes([mmap[pos], mmap[pos + 1], mmap[pos + 2], mmap[pos + 3]]) as usize;
        pos += 4;

        // Linear probe within bucket collecting matches up to max_candidates
        for _ in 0..entry_count {
            if pos + CacheEntry::SIZE > bucket_end {
                break;
            }

            // Read entry hash first (fast rejection)
            let entry_hash = u64::from_le_bytes([
                mmap[pos + 4],
                mmap[pos + 5],
                mmap[pos + 6],
                mmap[pos + 7],
                mmap[pos + 8],
                mmap[pos + 9],
                mmap[pos + 10],
                mmap[pos + 11],
            ]);

            if entry_hash == name_hash {
                // Hash matches, extract symbol ID
                if let Some(symbol_id) = SymbolId::new(u32::from_le_bytes([
                    mmap[pos],
                    mmap[pos + 1],
                    mmap[pos + 2],
                    mmap[pos + 3],
                ])) {
                    results.push(symbol_id);
                    if results.len() >= max_candidates {
                        break;
                    }
                }
            }

            pos += CacheEntry::SIZE;
        }

        results
    }

    /// Build cache from symbols (called during indexing)
    pub fn build_from_symbols<'a>(
        path: impl AsRef<Path>,
        symbols: impl Iterator<Item = &'a Symbol>,
    ) -> io::Result<()> {
        let path = path.as_ref();
        let mut buckets: Vec<Vec<CacheEntry>> = vec![Vec::new(); DEFAULT_BUCKET_COUNT];
        let mut symbol_count = 0;

        // Distribute symbols into buckets
        for symbol in symbols {
            let entry = CacheEntry::from_symbol(symbol);
            let bucket_idx = (entry.name_hash as usize) % DEFAULT_BUCKET_COUNT;
            buckets[bucket_idx].push(entry);
            symbol_count += 1;
        }

        // Calculate bucket offsets
        let mut bucket_offsets = Vec::with_capacity(DEFAULT_BUCKET_COUNT);
        let mut current_offset = HEADER_SIZE as u64 + (DEFAULT_BUCKET_COUNT * 8) as u64;

        for bucket in &buckets {
            bucket_offsets.push(current_offset);
            current_offset += 4; // Entry count
            current_offset += (bucket.len() * CacheEntry::SIZE) as u64;
        }

        // Write to file (with Windows file locking retry logic)
        let mut file = {
            let mut attempts = 0;
            const MAX_ATTEMPTS: u32 = 3;

            loop {
                match OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                {
                    Ok(file) => break file,
                    Err(e) if attempts < MAX_ATTEMPTS => {
                        // Check for Windows file locking error (os error 1224)
                        if cfg!(windows) && e.to_string().contains("os error 1224") {
                            attempts += 1;
                            eprintln!(
                                "Attempt {attempts}/{MAX_ATTEMPTS}: Windows file lock detected, retrying..."
                            );

                            // Try to delete the file if it exists to break the lock
                            if path.exists() {
                                if let Err(del_err) = std::fs::remove_file(path) {
                                    eprintln!("Warning: Could not delete cache file: {del_err}");
                                } else {
                                    eprintln!("Deleted existing cache file to break file lock");
                                }
                            }

                            // Brief delay before retry
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }
                        return Err(e);
                    }
                    Err(e) => return Err(e),
                }
            }
        };

        // Write header
        file.write_all(MAGIC_BYTES)?;
        file.write_all(&VERSION.to_le_bytes())?;
        file.write_all(&(DEFAULT_BUCKET_COUNT as u32).to_le_bytes())?;
        file.write_all(&(symbol_count as u64).to_le_bytes())?;
        file.write_all(&[0u8; 12])?; // Reserved

        // Write bucket offsets
        for offset in &bucket_offsets {
            file.write_all(&offset.to_le_bytes())?;
        }

        // Write bucket data
        for bucket in &buckets {
            file.write_all(&(bucket.len() as u32).to_le_bytes())?;
            for entry in bucket {
                file.write_all(&entry.symbol_id.to_le_bytes())?;
                file.write_all(&entry.name_hash.to_le_bytes())?;
                file.write_all(&entry.file_id.to_le_bytes())?;
                file.write_all(&entry.line.to_le_bytes())?;
                file.write_all(&entry.column.to_le_bytes())?;
                file.write_all(&[entry.kind, entry._padding])?;
            }
        }

        file.sync_all()?;
        Ok(())
    }
}

/// Thread-safe wrapper for concurrent access
pub struct ConcurrentSymbolCache {
    inner: Arc<RwLock<SymbolHashCache>>,
}

impl ConcurrentSymbolCache {
    pub fn new(cache: SymbolHashCache) -> Self {
        Self {
            inner: Arc::new(RwLock::new(cache)),
        }
    }

    pub fn lookup_by_name(&self, name: &str) -> Option<SymbolId> {
        self.inner.read().lookup_by_name(name)
    }

    pub fn lookup_candidates(&self, name: &str, max_candidates: usize) -> Vec<SymbolId> {
        self.inner.read().lookup_candidates(name, max_candidates)
    }
}
