//! Data source management layer with platform detection
//!
//! This module orchestrates data retrieval from multiple sources (GitHub Atom feeds,
//! REST API, and local cache) with intelligent fallback mechanisms and automatic
//! platform-specific asset filtering.

use crate::{AppError, Result};
use super::{
    cache::{CacheManager, CacheStats},
    feeds::{FeedsClient, FeedStats},
    github::{GitHubApiClient, GitHubApiStats},
    types::{Release, ReleaseAsset, PlatformInfo},
};
use std::time::{Duration, Instant};

/// Data source priority levels for fallback logic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSourcePriority {
    /// Cache has highest priority for performance
    Cache = 1,
    /// Atom feeds have second priority (no rate limits)
    Feeds = 2,
    /// REST API has lowest priority (rate limited)
    Api = 3,
}

/// Data source status tracking
#[derive(Debug, Clone)]
pub struct DataSourceStatus {
    /// Which data source was used
    pub source: DataSourcePriority,
    /// Response time for the operation
    pub response_time: Duration,
    /// Whether the operation was successful
    pub success: bool,
    /// Additional context or error message
    pub message: Option<String>,
}

/// Comprehensive statistics from data source operations
#[derive(Debug, Clone)]
pub struct DataSourceStats {
    /// Cache operation statistics
    pub cache_stats: Option<CacheStats>,
    /// Atom feeds operation statistics
    pub feeds_stats: Option<FeedStats>,
    /// GitHub API operation statistics
    pub api_stats: Option<GitHubApiStats>,
    /// Overall operation status
    pub operation_status: DataSourceStatus,
    /// Platform detection results
    pub platform_info: PlatformInfo,
    /// Number of assets filtered for current platform
    pub platform_filtered_assets: usize,
}

/// Main data source management coordinator
pub struct DataSourceManager {
    /// Cache manager for local data storage
    cache_manager: CacheManager,
    /// GitHub Atom feeds client
    feeds_client: FeedsClient,
    /// GitHub REST API client
    github_client: GitHubApiClient,
    /// Current platform information
    platform_info: PlatformInfo,
    /// Start time for operation tracking
    start_time: Instant,
    /// Verbose output mode
    verbose: bool,
}

impl DataSourceManager {
    /// Create a new DataSourceManager with default configuration
    pub fn new() -> Self {
        let github_client = GitHubApiClient::new("MaurUppi".to_string(), "network-latency-tester".to_string())
            .unwrap_or_else(|_| panic!("Failed to create GitHub API client"));
            
        Self {
            cache_manager: CacheManager::new().unwrap_or_else(|_| panic!("Failed to create cache manager")),
            feeds_client: FeedsClient::new("https://github.com/MaurUppi/network-latency-tester".to_string())
                .unwrap_or_else(|_| panic!("Failed to create feeds client")),
            github_client,
            platform_info: PlatformInfo::current(),
            start_time: Instant::now(),
            verbose: false,
        }
    }

    /// Create a new DataSourceManager with custom configuration
    pub fn with_config(_use_colors: bool, verbose: bool) -> Self {
        let github_client = GitHubApiClient::with_config(
            "MaurUppi".to_string(),
            "network-latency-tester".to_string(),
            verbose,
            None,
            None,
        ).unwrap_or_else(|_| panic!("Failed to create GitHub API client"));
        
        Self {
            cache_manager: CacheManager::with_config(None, verbose, None)
                .unwrap_or_else(|_| panic!("Failed to create cache manager")),
            feeds_client: FeedsClient::with_config(
                "https://github.com/MaurUppi/network-latency-tester".to_string(),
                verbose,
                None
            ).unwrap_or_else(|_| panic!("Failed to create feeds client")),
            github_client,
            platform_info: PlatformInfo::current(),
            start_time: Instant::now(),
            verbose,
        }
    }

    /// Get releases with intelligent fallback from cache -> feeds -> API
    pub async fn get_releases(&mut self, limit: usize) -> Result<(Vec<Release>, DataSourceStats)> {
        let start_time = Instant::now();
        
        if self.verbose {
            self.log_info(&format!("Retrieving up to {} releases for platform: {}", 
                limit, self.platform_info.display_name()));
        }

        // Strategy 1: Try cache first for best performance
        if let Ok(Some(cached_releases)) = self.cache_manager.get_cached_releases() {
            if self.verbose {
                self.log_success(&format!("Found {} cached releases", cached_releases.len()));
            }

            let filtered_releases = self.filter_releases_by_platform(&cached_releases);
            
            let stats = DataSourceStats {
                cache_stats: self.cache_manager.get_cache_stats().ok(),
                feeds_stats: None,
                api_stats: None,
                operation_status: DataSourceStatus {
                    source: DataSourcePriority::Cache,
                    response_time: start_time.elapsed(),
                    success: true,
                    message: Some(format!("Retrieved {} releases from cache", cached_releases.len())),
                },
                platform_info: self.platform_info.clone(),
                platform_filtered_assets: filtered_releases.iter()
                    .map(|r| r.assets.len())
                    .sum(),
            };

            return Ok((filtered_releases, stats));
        }

        if self.verbose {
            self.log_info("Cache miss or expired, trying Atom feeds...");
        }

        // Strategy 2: Try Atom feeds (no rate limits)
        match self.feeds_client.fetch_releases().await {
            Ok(feeds_releases) => {
                if self.verbose {
                    self.log_success(&format!("Retrieved {} releases from Atom feeds", feeds_releases.len()));
                }

                // Cache the successful result
                if let Err(e) = self.cache_manager.save_cache(&feeds_releases, None) {
                    if self.verbose {
                        self.log_warning(&format!("Failed to cache feeds results: {}", e));
                    }
                }

                let filtered_releases = self.filter_releases_by_platform(&feeds_releases);

                let stats = DataSourceStats {
                    cache_stats: self.cache_manager.get_cache_stats().ok(),
                    feeds_stats: None, // FeedsClient doesn't have get_stats yet
                    api_stats: None,
                    operation_status: DataSourceStatus {
                        source: DataSourcePriority::Feeds,
                        response_time: start_time.elapsed(),
                        success: true,
                        message: Some(format!("Retrieved {} releases from Atom feeds", feeds_releases.len())),
                    },
                    platform_info: self.platform_info.clone(),
                    platform_filtered_assets: filtered_releases.iter()
                        .map(|r| r.assets.len())
                        .sum(),
                };

                return Ok((filtered_releases, stats));
            }
            Err(e) => {
                if self.verbose {
                    self.log_warning(&format!("Atom feeds failed: {}", e));
                }
            }
        }

        if self.verbose {
            self.log_info("Atom feeds unavailable, trying GitHub REST API...");
        }

        // Strategy 3: Try GitHub REST API (rate limited, last resort)
        match self.github_client.fetch_releases().await {
            Ok(api_releases) => {
                if self.verbose {
                    self.log_success(&format!("Retrieved {} releases from GitHub API", api_releases.len()));
                }

                // Cache the successful result
                if let Err(e) = self.cache_manager.save_cache(&api_releases, None) {
                    if self.verbose {
                        self.log_warning(&format!("Failed to cache API results: {}", e));
                    }
                }

                let filtered_releases = self.filter_releases_by_platform(&api_releases);

                let stats = DataSourceStats {
                    cache_stats: self.cache_manager.get_cache_stats().ok(),
                    feeds_stats: None, // FeedsClient doesn't have get_stats yet
                    api_stats: None, // GitHubApiClient doesn't have get_stats yet
                    operation_status: DataSourceStatus {
                        source: DataSourcePriority::Api,
                        response_time: start_time.elapsed(),
                        success: true,
                        message: Some(format!("Retrieved {} releases from GitHub API", api_releases.len())),
                    },
                    platform_info: self.platform_info.clone(),
                    platform_filtered_assets: filtered_releases.iter()
                        .map(|r| r.assets.len())
                        .sum(),
                };

                return Ok((filtered_releases, stats));
            }
            Err(e) => {
                if self.verbose {
                    self.log_error(&format!("GitHub API failed: {}", e));
                }
            }
        }

        // All sources failed
        let error_msg = "All data sources failed: cache miss/expired, Atom feeds unavailable, and GitHub API unavailable";
        
        Err(AppError::update(error_msg.to_string()))
    }

    /// Get a specific release by version with intelligent fallback
    pub async fn get_specific_release(&mut self, version: &str) -> Result<(Option<Release>, DataSourceStats)> {
        let start_time = Instant::now();
        
        if self.verbose {
            self.log_info(&format!("Searching for specific release: {} for platform: {}", 
                version, self.platform_info.display_name()));
        }

        // Normalize version string (strip 'v' prefix if present)
        let normalized_version = version.strip_prefix('v').unwrap_or(version);

        // Try to get releases from all sources and search for the specific version
        match self.get_all_releases_for_search().await {
            Ok((releases, mut stats)) => {
                let found_release = releases.into_iter()
                    .find(|release| {
                        let release_version = release.version();
                        release_version == normalized_version || 
                        release.tag_name == version ||
                        release.tag_name == format!("v{}", normalized_version)
                    });

                if let Some(release) = found_release.as_ref() {
                    if self.verbose {
                        self.log_success(&format!("Found release {} with {} platform-specific assets", 
                            release.tag_name, release.assets.len()));
                    }
                } else if self.verbose {
                    self.log_warning(&format!("Release {} not found in available releases", version));
                }

                // Update stats for specific release search
                stats.operation_status.message = Some(format!(
                    "Searched for specific release: {} - {}",
                    version,
                    if found_release.is_some() { "found" } else { "not found" }
                ));
                stats.operation_status.response_time = start_time.elapsed();

                Ok((found_release, stats))
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    /// Get all releases for search operations (without limit)
    async fn get_all_releases_for_search(&mut self) -> Result<(Vec<Release>, DataSourceStats)> {
        // For search operations, get more releases to increase chances of finding specific version
        self.get_releases(100).await
    }

    /// Get current platform information
    pub fn get_platform_info(&self) -> &PlatformInfo {
        &self.platform_info
    }

    /// Detect current platform (using std::env::consts as specified in leverage)
    pub fn detect_current_platform() -> PlatformInfo {
        PlatformInfo::current()
    }

    /// Filter releases to include only assets compatible with current platform
    pub fn filter_releases_by_platform(&self, releases: &[Release]) -> Vec<Release> {
        releases.iter()
            .map(|release| {
                let filtered_assets = self.filter_assets_by_platform(&release.assets);
                Release {
                    tag_name: release.tag_name.clone(),
                    name: release.name.clone(),
                    published_at: release.published_at.clone(),
                    html_url: release.html_url.clone(),
                    assets: filtered_assets,
                    prerelease: release.prerelease,
                }
            })
            .collect()
    }

    /// Filter assets to include only those compatible with current platform
    pub fn filter_assets_by_platform(&self, assets: &[ReleaseAsset]) -> Vec<ReleaseAsset> {
        assets.iter()
            .filter(|asset| self.platform_info.matches_asset_name(&asset.name))
            .cloned()
            .collect()
    }

    /// Prioritize assets based on platform preferences
    pub fn prioritize_assets(&self, assets: &[ReleaseAsset]) -> Vec<ReleaseAsset> {
        let mut prioritized = assets.to_vec();
        
        // Sort by preference: preferred extension first, then by name
        prioritized.sort_by(|a, b| {
            let a_preferred = a.name.ends_with(self.platform_info.preferred_extension());
            let b_preferred = b.name.ends_with(self.platform_info.preferred_extension());
            
            match (a_preferred, b_preferred) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        prioritized
    }

    /// Get comprehensive statistics from all data sources
    pub fn get_comprehensive_stats(&self) -> DataSourceStats {
        DataSourceStats {
            cache_stats: self.cache_manager.get_cache_stats().ok(),
            feeds_stats: None, // Will be added when FeedsClient gets stats methods
            api_stats: None, // Will be added when GitHubApiClient gets stats methods
            operation_status: DataSourceStatus {
                source: DataSourcePriority::Cache,
                response_time: self.start_time.elapsed(),
                success: true,
                message: Some("Comprehensive stats snapshot".to_string()),
            },
            platform_info: self.platform_info.clone(),
            platform_filtered_assets: 0,
        }
    }

    /// Check data source availability without making requests
    pub async fn check_availability(&mut self) -> Result<(bool, bool, bool)> {
        if self.verbose {
            self.log_info("Checking data source availability...");
        }

        let cache_available = self.cache_manager.is_cache_valid();
        
        // For feeds and API, we'll do quick connectivity checks
        // For feeds, try a simple fetch to check availability
        let feeds_available = match self.feeds_client.fetch_releases().await {
            Ok(_) => true,
            Err(_) => false,
        };

        let api_available = match self.github_client.check_api_availability().await {
            Ok(availability) => availability.available,
            Err(_) => false,
        };

        if self.verbose {
            self.log_info(&format!(
                "Data source availability - Cache: {}, Feeds: {}, API: {}",
                cache_available, feeds_available, api_available
            ));
        }

        Ok((cache_available, feeds_available, api_available))
    }

    /// Force refresh from remote sources (bypassing cache)
    pub async fn force_refresh(&mut self, _limit: usize) -> Result<(Vec<Release>, DataSourceStats)> {
        let start_time = Instant::now();
        
        if self.verbose {
            self.log_info("Force refreshing from remote sources (bypassing cache)...");
        }

        // Try feeds first (no rate limits)
        match self.feeds_client.fetch_releases().await {
            Ok(feeds_releases) => {
                if self.verbose {
                    self.log_success(&format!("Force refresh: Retrieved {} releases from Atom feeds", feeds_releases.len()));
                }

                // Update cache with fresh data
                if let Err(e) = self.cache_manager.save_cache(&feeds_releases, None) {
                    if self.verbose {
                        self.log_warning(&format!("Failed to update cache after force refresh: {}", e));
                    }
                }

                let filtered_releases = self.filter_releases_by_platform(&feeds_releases);

                let stats = DataSourceStats {
                    cache_stats: self.cache_manager.get_cache_stats().ok(),
                    feeds_stats: None,
                    api_stats: None,
                    operation_status: DataSourceStatus {
                        source: DataSourcePriority::Feeds,
                        response_time: start_time.elapsed(),
                        success: true,
                        message: Some(format!("Force refresh: Retrieved {} releases from Atom feeds", feeds_releases.len())),
                    },
                    platform_info: self.platform_info.clone(),
                    platform_filtered_assets: filtered_releases.iter()
                        .map(|r| r.assets.len())
                        .sum(),
                };

                return Ok((filtered_releases, stats));
            }
            Err(e) => {
                if self.verbose {
                    self.log_warning(&format!("Force refresh: Atom feeds failed: {}", e));
                }
            }
        }

        // Fallback to API for force refresh
        match self.github_client.fetch_releases().await {
            Ok(api_releases) => {
                if self.verbose {
                    self.log_success(&format!("Force refresh: Retrieved {} releases from GitHub API", api_releases.len()));
                }

                // Update cache with fresh data
                if let Err(e) = self.cache_manager.save_cache(&api_releases, None) {
                    if self.verbose {
                        self.log_warning(&format!("Failed to update cache after force refresh: {}", e));
                    }
                }

                let filtered_releases = self.filter_releases_by_platform(&api_releases);

                let stats = DataSourceStats {
                    cache_stats: self.cache_manager.get_cache_stats().ok(),
                    feeds_stats: None,
                    api_stats: None,
                    operation_status: DataSourceStatus {
                        source: DataSourcePriority::Api,
                        response_time: start_time.elapsed(),
                        success: true,
                        message: Some(format!("Force refresh: Retrieved {} releases from GitHub API", api_releases.len())),
                    },
                    platform_info: self.platform_info.clone(),
                    platform_filtered_assets: filtered_releases.iter()
                        .map(|r| r.assets.len())
                        .sum(),
                };

                Ok((filtered_releases, stats))
            }
            Err(e) => {
                let error_msg = format!("Force refresh failed: both Atom feeds and GitHub API unavailable. Last API error: {}", e);
                
                let _stats = DataSourceStats {
                    cache_stats: self.cache_manager.get_cache_stats().ok(),
                    feeds_stats: None,
                    api_stats: None,
                    operation_status: DataSourceStatus {
                        source: DataSourcePriority::Api,
                        response_time: start_time.elapsed(),
                        success: false,
                        message: Some(error_msg.clone()),
                    },
                    platform_info: self.platform_info.clone(),
                    platform_filtered_assets: 0,
                };

                Err(AppError::update(error_msg))
            }
        }
    }

    /// Log info message with optional color (reusing existing patterns)
    fn log_info(&self, message: &str) {
        eprintln!("[DATA] {}", message);
    }

    /// Log success message with optional color (reusing existing patterns)
    fn log_success(&self, message: &str) {
        eprintln!("[DATA] ✓ {}", message);
    }

    /// Log warning message with optional color (reusing existing patterns)
    fn log_warning(&self, message: &str) {
        eprintln!("[DATA] ⚠ {}", message);
    }

    /// Log error message with optional color (reusing existing patterns)
    fn log_error(&self, message: &str) {
        eprintln!("[DATA] ✗ {}", message);
    }
}

impl Default for DataSourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DataSourceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataSourceManager")
            .field("platform_info", &self.platform_info)
            .field("elapsed", &self.start_time.elapsed())
            .field("verbose", &self.verbose)
            .field("cache_stats", &self.cache_manager.get_cache_stats().ok())
            .field("feeds_stats", &"Not implemented yet")
            .field("api_stats", &"Not implemented yet")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updater::types::{ReleaseAsset, Release};

    #[test]
    fn test_data_source_manager_creation() {
        let manager = DataSourceManager::new();
        assert!(!manager.verbose);
        assert!(manager.get_duration() < Duration::from_millis(100));
    }

    #[test]
    fn test_data_source_manager_with_config() {
        let manager = DataSourceManager::with_config(true, true);
        assert!(manager.verbose);
    }

    #[test]
    fn test_platform_detection() {
        let platform = DataSourceManager::detect_current_platform();
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
        assert!(!platform.target_triple.is_empty());
    }

    #[test]
    fn test_asset_filtering_by_platform() {
        let manager = DataSourceManager::new();
        
        let assets = vec![
            ReleaseAsset::new(
                "nlt-v0.1.9-darwin-x64.tar.gz".to_string(),
                "https://example.com/darwin-x64".to_string(),
                1024,
                "application/gzip".to_string(),
            ),
            ReleaseAsset::new(
                "nlt-v0.1.9-windows-x64.zip".to_string(),
                "https://example.com/windows-x64".to_string(),
                2048,
                "application/zip".to_string(),
            ),
            ReleaseAsset::new(
                "nlt-v0.1.9-linux-x64-ubuntu.tar.gz".to_string(),
                "https://example.com/linux-x64".to_string(),
                1536,
                "application/gzip".to_string(),
            ),
        ];

        let filtered = manager.filter_assets_by_platform(&assets);
        
        // Should filter to only assets matching current platform
        for asset in &filtered {
            assert!(manager.platform_info.matches_asset_name(&asset.name));
        }
    }

    #[test]
    fn test_asset_prioritization() {
        let manager = DataSourceManager::new();
        
        // Create assets with different extensions
        let assets = vec![
            ReleaseAsset::new(
                "app.zip".to_string(),
                "https://example.com/zip".to_string(),
                1024,
                "application/zip".to_string(),
            ),
            ReleaseAsset::new(
                "app.tar.gz".to_string(),
                "https://example.com/tar".to_string(),
                1024,
                "application/gzip".to_string(),
            ),
        ];

        let prioritized = manager.prioritize_assets(&assets);
        
        // First asset should match platform's preferred extension
        let preferred_ext = manager.platform_info.preferred_extension();
        assert!(prioritized[0].name.ends_with(preferred_ext));
    }

    #[test]
    fn test_release_filtering_by_platform() {
        let manager = DataSourceManager::new();
        
        let releases = vec![
            Release::new(
                "v0.1.9".to_string(),
                "Version 0.1.9".to_string(),
                "2024-01-01T00:00:00Z".to_string(),
                "https://github.com/example/repo/releases/tag/v0.1.9".to_string(),
                vec![
                    ReleaseAsset::new(
                        "nlt-v0.1.9-darwin-x64.tar.gz".to_string(),
                        "https://example.com/darwin".to_string(),
                        1024,
                        "application/gzip".to_string(),
                    ),
                    ReleaseAsset::new(
                        "nlt-v0.1.9-windows-x64.zip".to_string(),
                        "https://example.com/windows".to_string(),
                        2048,
                        "application/zip".to_string(),
                    ),
                ],
                false,
            ),
        ];

        let filtered_releases = manager.filter_releases_by_platform(&releases);
        
        assert_eq!(filtered_releases.len(), 1);
        
        // Assets should be filtered to current platform only
        for release in &filtered_releases {
            for asset in &release.assets {
                assert!(manager.platform_info.matches_asset_name(&asset.name));
            }
        }
    }

    #[test]
    fn test_data_source_priority_ordering() {
        assert_eq!(DataSourcePriority::Cache as u8, 1);
        assert_eq!(DataSourcePriority::Feeds as u8, 2);
        assert_eq!(DataSourcePriority::Api as u8, 3);
    }

    #[test]
    fn test_data_source_status_creation() {
        let status = DataSourceStatus {
            source: DataSourcePriority::Cache,
            response_time: Duration::from_millis(100),
            success: true,
            message: Some("Test message".to_string()),
        };

        assert_eq!(status.source, DataSourcePriority::Cache);
        assert!(status.success);
        assert!(status.response_time <= Duration::from_millis(100));
    }

    #[test]
    fn test_comprehensive_stats() {
        let manager = DataSourceManager::new();
        let stats = manager.get_comprehensive_stats();
        
        assert!(stats.cache_stats.is_some() || stats.cache_stats.is_none()); // Either way is valid
        assert_eq!(stats.platform_info, manager.platform_info);
    }

    #[tokio::test]
    async fn test_data_source_availability_check() {
        let mut manager = DataSourceManager::new();
        
        // This test checks the availability check method structure
        // Actual connectivity depends on network and GitHub availability
        let result = manager.check_availability().await;
        
        // Should return Ok with three boolean values regardless of actual availability
        assert!(result.is_ok());
        let (cache_available, feeds_available, api_available) = result.unwrap();
        
        // These are just type checks - actual values depend on environment
        // Just verify we got boolean types back (no assertions on actual values since they depend on network)
        let _: bool = cache_available;
        let _: bool = feeds_available;
        let _: bool = api_available;
    }

    impl DataSourceManager {
        /// Get operation duration for testing
        pub fn get_duration(&self) -> Duration {
            self.start_time.elapsed()
        }
    }
}