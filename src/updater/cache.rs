//! Cache management system for update feature
//!
//! This module provides local caching functionality to reduce network requests
//! and improve performance when checking for updates. The cache stores release
//! information locally with proper expiration and validation.

use crate::updater::types::Release;
use crate::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default cache expiration time (24 hours)
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Maximum number of releases to keep in cache
const MAX_CACHED_RELEASES: usize = 50;

/// Cache metadata and data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheData {
    /// Timestamp when cache was created
    pub created_at: u64,
    /// Time-to-live in seconds
    pub ttl_seconds: u64,
    /// ETag from last GitHub API response (if available)
    pub etag: Option<String>,
    /// Cached release data
    pub releases: Vec<Release>,
    /// Cache format version for compatibility
    pub version: u32,
}

impl CacheData {
    /// Create new cache data
    pub fn new(releases: Vec<Release>, etag: Option<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            created_at: now,
            ttl_seconds: DEFAULT_CACHE_TTL.as_secs(),
            etag,
            releases: releases.into_iter().take(MAX_CACHED_RELEASES).collect(),
            version: 1,
        }
    }

    /// Check if cache is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let age = now.saturating_sub(self.created_at);
        age < self.ttl_seconds
    }

    /// Get cache age in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now.saturating_sub(self.created_at)
    }

    /// Check if cache format is compatible
    pub fn is_compatible(&self) -> bool {
        self.version == 1
    }
}

/// Cache manager for release data
pub struct CacheManager {
    /// Cache file path
    cache_path: PathBuf,
    /// Whether verbose logging is enabled
    verbose: bool,
    /// Custom TTL override
    custom_ttl: Option<Duration>,
}

impl CacheManager {
    /// Create a new CacheManager with default settings
    pub fn new() -> Result<Self> {
        let cache_path = Self::get_default_cache_path()?;
        Ok(Self {
            cache_path,
            verbose: false,
            custom_ttl: None,
        })
    }

    /// Create a new CacheManager with configuration
    pub fn with_config(cache_path: Option<PathBuf>, verbose: bool, ttl: Option<Duration>) -> Result<Self> {
        let cache_path = cache_path.unwrap_or_else(|| Self::get_default_cache_path().unwrap_or_else(|_| {
            // Fallback to current directory if XDG path fails
            PathBuf::from("releases_cache.json")
        }));

        Ok(Self {
            cache_path,
            verbose,
            custom_ttl: ttl,
        })
    }

    /// Get the default cache directory following XDG specification
    pub fn get_default_cache_path() -> Result<PathBuf> {
        // Try to get XDG cache directory
        let cache_dir = if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
            PathBuf::from(xdg_cache)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".cache")
        } else {
            // Fallback for systems without HOME environment variable
            return Ok(PathBuf::from("releases_cache.json"));
        };

        let app_cache_dir = cache_dir.join("network-latency-tester");
        Ok(app_cache_dir.join("releases.json"))
    }

    /// Ensure cache directory exists
    fn ensure_cache_directory(&self) -> Result<()> {
        if let Some(parent) = self.cache_path.parent() {
            if !parent.exists() {
                if self.verbose {
                    eprintln!("[CACHE] Creating cache directory: {}", parent.display());
                }
                fs::create_dir_all(parent)
                    .map_err(|e| AppError::cache(format!("Failed to create cache directory '{}': {}", parent.display(), e)))?;
            }
        }
        Ok(())
    }

    /// Load cached data if available and valid
    pub fn load_cache(&self) -> Result<Option<CacheData>> {
        if !self.cache_path.exists() {
            if self.verbose {
                eprintln!("[CACHE] No cache file found at: {}", self.cache_path.display());
            }
            return Ok(None);
        }

        if self.verbose {
            eprintln!("[CACHE] Loading cache from: {}", self.cache_path.display());
        }

        let content = fs::read_to_string(&self.cache_path)
            .map_err(|e| AppError::cache(format!("Failed to read cache file '{}': {}", self.cache_path.display(), e)))?;

        let cache_data: CacheData = serde_json::from_str(&content)
            .map_err(|e| AppError::cache(format!("Failed to parse cache file '{}': {}", self.cache_path.display(), e)))?;

        // Check cache compatibility
        if !cache_data.is_compatible() {
            if self.verbose {
                eprintln!("[CACHE] Cache format incompatible (version {}), invalidating", cache_data.version);
            }
            return Ok(None);
        }

        // Check cache validity
        if !cache_data.is_valid() {
            if self.verbose {
                eprintln!("[CACHE] Cache expired (age: {}s), invalidating", cache_data.age_seconds());
            }
            return Ok(None);
        }

        if self.verbose {
            eprintln!("[CACHE] Loaded {} releases from cache (age: {}s)", 
                cache_data.releases.len(), cache_data.age_seconds());
        }

        Ok(Some(cache_data))
    }

    /// Save releases to cache
    pub fn save_cache(&self, releases: &[Release], etag: Option<String>) -> Result<()> {
        self.ensure_cache_directory()?;

        let cache_data = if let Some(custom_ttl) = self.custom_ttl {
            let mut data = CacheData::new(releases.to_vec(), etag);
            data.ttl_seconds = custom_ttl.as_secs();
            data
        } else {
            CacheData::new(releases.to_vec(), etag)
        };

        let content = serde_json::to_string_pretty(&cache_data)
            .map_err(|e| AppError::cache(format!("Failed to serialize cache data: {}", e)))?;

        if self.verbose {
            eprintln!("[CACHE] Saving {} releases to cache: {}", 
                cache_data.releases.len(), self.cache_path.display());
        }

        fs::write(&self.cache_path, content)
            .map_err(|e| AppError::cache(format!("Failed to write cache file '{}': {}", self.cache_path.display(), e)))?;

        if self.verbose {
            eprintln!("[CACHE] Cache saved successfully");
        }

        Ok(())
    }

    /// Get cached releases if available and valid
    pub fn get_cached_releases(&self) -> Result<Option<Vec<Release>>> {
        match self.load_cache()? {
            Some(cache_data) => Ok(Some(cache_data.releases)),
            None => Ok(None),
        }
    }

    /// Get cached ETag if available and valid
    pub fn get_cached_etag(&self) -> Result<Option<String>> {
        match self.load_cache()? {
            Some(cache_data) => Ok(cache_data.etag),
            None => Ok(None),
        }
    }

    /// Check if cache is valid without loading data
    pub fn is_cache_valid(&self) -> bool {
        match self.load_cache() {
            Ok(Some(_)) => true,
            _ => false,
        }
    }

    /// Clear/invalidate the cache
    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_path.exists() {
            if self.verbose {
                eprintln!("[CACHE] Clearing cache: {}", self.cache_path.display());
            }
            fs::remove_file(&self.cache_path)
                .map_err(|e| AppError::cache(format!("Failed to remove cache file '{}': {}", self.cache_path.display(), e)))?;
        }
        Ok(())
    }

    /// Get cache file size in bytes
    pub fn get_cache_size(&self) -> Result<u64> {
        if !self.cache_path.exists() {
            return Ok(0);
        }

        let metadata = fs::metadata(&self.cache_path)
            .map_err(|e| AppError::cache(format!("Failed to get cache metadata '{}': {}", self.cache_path.display(), e)))?;

        Ok(metadata.len())
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> Result<CacheStats> {
        let exists = self.cache_path.exists();
        let size = if exists { self.get_cache_size()? } else { 0 };
        
        let (valid, age_seconds, release_count, etag) = if let Ok(Some(cache_data)) = self.load_cache() {
            (
                cache_data.is_valid(),
                Some(cache_data.age_seconds()),
                Some(cache_data.releases.len()),
                cache_data.etag,
            )
        } else {
            (false, None, None, None)
        };

        Ok(CacheStats {
            exists,
            valid,
            size_bytes: size,
            age_seconds,
            release_count,
            etag,
            path: self.cache_path.clone(),
        })
    }

    /// Perform cache maintenance (cleanup old or corrupted cache)
    pub fn maintain_cache(&self) -> Result<()> {
        if !self.cache_path.exists() {
            return Ok(());
        }

        // Try to load cache to check validity
        match self.load_cache() {
            Ok(Some(_)) => {
                if self.verbose {
                    eprintln!("[CACHE] Cache is valid, no maintenance needed");
                }
                Ok(())
            }
            Ok(None) => {
                // Cache is invalid or expired, remove it
                if self.verbose {
                    eprintln!("[CACHE] Cache is invalid/expired, performing cleanup");
                }
                self.clear_cache()
            }
            Err(_) => {
                // Cache is corrupted, remove it
                if self.verbose {
                    eprintln!("[CACHE] Cache is corrupted, performing cleanup");
                }
                self.clear_cache()
            }
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback cache manager if default creation fails
            Self {
                cache_path: PathBuf::from("releases_cache.json"),
                verbose: false,
                custom_ttl: None,
            }
        })
    }
}

/// Cache statistics information
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Whether cache file exists
    pub exists: bool,
    /// Whether cache is valid (not expired)
    pub valid: bool,
    /// Cache file size in bytes
    pub size_bytes: u64,
    /// Cache age in seconds (if available)
    pub age_seconds: Option<u64>,
    /// Number of releases in cache (if available)
    pub release_count: Option<usize>,
    /// Cached ETag (if available)
    pub etag: Option<String>,
    /// Cache file path
    pub path: PathBuf,
}

impl CacheStats {
    /// Format cache statistics for display
    pub fn format_stats(&self, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            use colored::Colorize;
            output.push_str(&format!("📁 Cache Path: {}\n", self.path.display().to_string().cyan()));
            output.push_str(&format!("📊 Status: {}\n", 
                if self.valid { "Valid".green() } else if self.exists { "Invalid/Expired".yellow() } else { "Not Found".red() }
            ));
            
            if self.exists {
                output.push_str(&format!("💾 Size: {} bytes\n", self.size_bytes.to_string().blue()));
                if let Some(count) = self.release_count {
                    output.push_str(&format!("📦 Releases: {}\n", count.to_string().blue()));
                }
                if let Some(age) = self.age_seconds {
                    output.push_str(&format!("⏰ Age: {}s\n", age.to_string().blue()));
                }
                if let Some(ref etag) = self.etag {
                    output.push_str(&format!("🏷️  ETag: {}\n", etag.cyan()));
                }
            }
        } else {
            output.push_str(&format!("Cache Path: {}\n", self.path.display()));
            output.push_str(&format!("Status: {}\n", 
                if self.valid { "Valid" } else if self.exists { "Invalid/Expired" } else { "Not Found" }
            ));
            
            if self.exists {
                output.push_str(&format!("Size: {} bytes\n", self.size_bytes));
                if let Some(count) = self.release_count {
                    output.push_str(&format!("Releases: {}\n", count));
                }
                if let Some(age) = self.age_seconds {
                    output.push_str(&format!("Age: {}s\n", age));
                }
                if let Some(ref etag) = self.etag {
                    output.push_str(&format!("ETag: {}\n", etag));
                }
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tempfile::TempDir;

    fn create_test_release(tag: &str) -> Release {
        Release::new(
            tag.to_string(),
            format!("Release {}", tag),
            "2024-01-01T00:00:00Z".to_string(),
            format!("https://github.com/test/repo/releases/tag/{}", tag),
            vec![],
            false,
        )
    }

    #[test]
    fn test_cache_data_creation() {
        let releases = vec![
            create_test_release("v1.0.0"),
            create_test_release("v1.1.0"),
        ];

        let cache_data = CacheData::new(releases.clone(), Some("etag123".to_string()));
        
        assert_eq!(cache_data.releases.len(), 2);
        assert_eq!(cache_data.etag, Some("etag123".to_string()));
        assert_eq!(cache_data.version, 1);
        assert!(cache_data.is_valid());
        assert!(cache_data.is_compatible());
    }

    #[test]
    fn test_cache_data_expiration() {
        let releases = vec![create_test_release("v1.0.0")];
        let mut cache_data = CacheData::new(releases, None);
        
        // Set expiration to 1 second
        cache_data.ttl_seconds = 1;
        cache_data.created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - 2; // 2 seconds ago
        
        assert!(!cache_data.is_valid());
        assert!(cache_data.age_seconds() >= 2);
    }

    #[test]
    fn test_cache_data_max_releases() {
        let releases: Vec<Release> = (0..100)
            .map(|i| create_test_release(&format!("v1.{}.0", i)))
            .collect();
        
        let cache_data = CacheData::new(releases, None);
        assert_eq!(cache_data.releases.len(), MAX_CACHED_RELEASES);
    }

    #[test]
    fn test_cache_manager_creation() {
        let cache_manager = CacheManager::new();
        assert!(cache_manager.is_ok());
        
        let cache_manager = cache_manager.unwrap();
        assert!(!cache_manager.verbose);
        assert!(cache_manager.custom_ttl.is_none());
    }

    #[test]
    fn test_cache_manager_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");
        let custom_ttl = Duration::from_secs(3600);
        
        let cache_manager = CacheManager::with_config(
            Some(cache_path.clone()),
            true,
            Some(custom_ttl),
        ).unwrap();
        
        assert_eq!(cache_manager.cache_path, cache_path);
        assert!(cache_manager.verbose);
        assert_eq!(cache_manager.custom_ttl, Some(custom_ttl));
    }

    #[test]
    fn test_cache_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");
        
        let cache_manager = CacheManager::with_config(
            Some(cache_path.clone()),
            false,
            None,
        ).unwrap();
        
        let releases = vec![
            create_test_release("v1.0.0"),
            create_test_release("v1.1.0"),
        ];
        
        // Save cache
        cache_manager.save_cache(&releases, Some("etag123".to_string())).unwrap();
        assert!(cache_path.exists());
        
        // Load cache
        let loaded_cache = cache_manager.load_cache().unwrap();
        assert!(loaded_cache.is_some());
        
        let cache_data = loaded_cache.unwrap();
        assert_eq!(cache_data.releases.len(), 2);
        assert_eq!(cache_data.etag, Some("etag123".to_string()));
        assert!(cache_data.is_valid());
    }

    #[test]
    fn test_cache_get_methods() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");
        
        let cache_manager = CacheManager::with_config(
            Some(cache_path),
            false,
            None,
        ).unwrap();
        
        let releases = vec![create_test_release("v1.0.0")];
        cache_manager.save_cache(&releases, Some("etag456".to_string())).unwrap();
        
        // Test get_cached_releases
        let cached_releases = cache_manager.get_cached_releases().unwrap();
        assert!(cached_releases.is_some());
        assert_eq!(cached_releases.unwrap().len(), 1);
        
        // Test get_cached_etag
        let cached_etag = cache_manager.get_cached_etag().unwrap();
        assert_eq!(cached_etag, Some("etag456".to_string()));
        
        // Test is_cache_valid
        assert!(cache_manager.is_cache_valid());
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");
        
        let cache_manager = CacheManager::with_config(
            Some(cache_path.clone()),
            false,
            None,
        ).unwrap();
        
        let releases = vec![create_test_release("v1.0.0")];
        cache_manager.save_cache(&releases, None).unwrap();
        assert!(cache_path.exists());
        
        // Clear cache
        cache_manager.clear_cache().unwrap();
        assert!(!cache_path.exists());
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");
        
        let cache_manager = CacheManager::with_config(
            Some(cache_path.clone()),
            false,
            None,
        ).unwrap();
        
        // Test stats with no cache
        let stats = cache_manager.get_cache_stats().unwrap();
        assert!(!stats.exists);
        assert!(!stats.valid);
        assert_eq!(stats.size_bytes, 0);
        
        // Create cache and test stats
        let releases = vec![create_test_release("v1.0.0")];
        cache_manager.save_cache(&releases, Some("etag789".to_string())).unwrap();
        
        let stats = cache_manager.get_cache_stats().unwrap();
        assert!(stats.exists);
        assert!(stats.valid);
        assert!(stats.size_bytes > 0);
        assert_eq!(stats.release_count, Some(1));
        assert_eq!(stats.etag, Some("etag789".to_string()));
        assert!(stats.age_seconds.is_some());
        
        // Test formatted stats
        let formatted = stats.format_stats(false);
        assert!(formatted.contains("Cache Path:"));
        assert!(formatted.contains("Status: Valid"));
        assert!(formatted.contains("Size:"));
        assert!(formatted.contains("Releases: 1"));
    }

    #[test]
    fn test_cache_maintenance() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");
        
        let cache_manager = CacheManager::with_config(
            Some(cache_path.clone()),
            false,
            Some(Duration::from_secs(1)), // 1 second TTL
        ).unwrap();
        
        let releases = vec![create_test_release("v1.0.0")];
        cache_manager.save_cache(&releases, None).unwrap();
        assert!(cache_path.exists());
        
        // Wait for cache to expire
        thread::sleep(Duration::from_secs(2));
        
        // Maintenance should remove expired cache
        cache_manager.maintain_cache().unwrap();
        assert!(!cache_path.exists());
    }

    #[test]
    fn test_corrupted_cache_handling() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");
        
        let cache_manager = CacheManager::with_config(
            Some(cache_path.clone()),
            false,
            None,
        ).unwrap();
        
        // Write invalid JSON to cache file
        fs::write(&cache_path, "invalid json content").unwrap();
        assert!(cache_path.exists());
        
        // Loading should return None for corrupted cache
        let result = cache_manager.load_cache();
        assert!(result.is_err());
        
        // Maintenance should clean up corrupted cache
        cache_manager.maintain_cache().unwrap();
        assert!(!cache_path.exists());
    }

    #[test]
    fn test_cache_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let nested_cache_path = temp_dir.path().join("deep").join("nested").join("cache.json");
        
        let cache_manager = CacheManager::with_config(
            Some(nested_cache_path.clone()),
            false,
            None,
        ).unwrap();
        
        // Parent directories should not exist initially
        assert!(!nested_cache_path.parent().unwrap().exists());
        
        let releases = vec![create_test_release("v1.0.0")];
        cache_manager.save_cache(&releases, None).unwrap();
        
        // Directories should be created and cache should exist
        assert!(nested_cache_path.exists());
        assert!(nested_cache_path.parent().unwrap().exists());
    }

    #[test]
    fn test_default_cache_path() {
        let default_path = CacheManager::get_default_cache_path();
        assert!(default_path.is_ok());
        
        let path = default_path.unwrap();
        // Should contain the application name
        assert!(path.to_string_lossy().contains("network-latency-tester"));
        assert!(path.to_string_lossy().contains("releases.json"));
    }
}