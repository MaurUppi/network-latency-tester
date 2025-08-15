//! GitHub Atom feeds client for version information retrieval
//!
//! This module provides functionality to parse GitHub releases from Atom feeds,
//! which offers unlimited access without API rate limits. It serves as the primary
//! data source for version information.

use crate::{AppError, Result};
use crate::updater::types::{Release, ReleaseAsset};
use feed_rs::parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default timeout for feed requests
const FEED_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// GitHub Atom feeds client
pub struct FeedsClient {
    /// HTTP client for requests
    client: Client,
    /// Base repository URL for constructing feed URLs
    repo_url: String,
    /// Whether verbose logging is enabled
    verbose: bool,
}

impl FeedsClient {
    /// Create a new FeedsClient with default settings
    pub fn new(repo_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(FEED_REQUEST_TIMEOUT)
            .user_agent(format!(
                "network-latency-tester/{} (https://github.com/MaurUppi/network-latency-tester)",
                crate::VERSION
            ))
            .build()
            .map_err(|e| AppError::update(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            repo_url,
            verbose: false,
        })
    }

    /// Create a new FeedsClient with configuration
    pub fn with_config(repo_url: String, verbose: bool, timeout: Option<Duration>) -> Result<Self> {
        let timeout = timeout.unwrap_or(FEED_REQUEST_TIMEOUT);
        
        let client = Client::builder()
            .timeout(timeout)
            .user_agent(format!(
                "network-latency-tester/{} (https://github.com/MaurUppi/network-latency-tester)",
                crate::VERSION
            ))
            .build()
            .map_err(|e| AppError::update(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            repo_url,
            verbose,
        })
    }

    /// Fetch and parse releases from GitHub Atom feeds
    pub async fn fetch_releases(&self) -> Result<Vec<Release>> {
        let feed_url = self.construct_feed_url();
        
        if self.verbose {
            eprintln!("[FEEDS] Fetching releases from: {}", feed_url);
        }

        // Fetch the Atom feed
        let response = self.client
            .get(&feed_url)
            .send()
            .await
            .map_err(|e| AppError::update(format!("Failed to fetch Atom feed from '{}': {}", feed_url, e)))?;

        // Check response status
        if !response.status().is_success() {
            return Err(AppError::update(format!(
                "Failed to fetch Atom feed: HTTP {} from '{}'",
                response.status(),
                feed_url
            )));
        }

        // Get response text
        let feed_content = response
            .text()
            .await
            .map_err(|e| AppError::update(format!("Failed to read Atom feed content: {}", e)))?;

        if self.verbose {
            eprintln!("[FEEDS] Received {} bytes of feed content", feed_content.len());
        }

        // Parse the Atom feed
        let feed = parser::parse(feed_content.as_bytes())
            .map_err(|e| AppError::update(format!("Failed to parse Atom feed: {}", e)))?;

        if self.verbose {
            eprintln!("[FEEDS] Parsed feed with {} entries", feed.entries.len());
        }

        // Convert feed entries to Release structs
        let releases: Vec<Release> = feed.entries
            .into_iter()
            .filter_map(|entry| self.convert_entry_to_release(entry))
            .collect();

        if self.verbose {
            eprintln!("[FEEDS] Converted {} feed entries to releases", releases.len());
        }

        Ok(releases)
    }

    /// Fetch a specific number of recent releases
    pub async fn fetch_recent_releases(&self, limit: usize) -> Result<Vec<Release>> {
        let mut releases = self.fetch_releases().await?;
        
        // Sort by published date (newest first) and take only the requested number
        releases.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        releases.truncate(limit);
        
        Ok(releases)
    }

    /// Search for a specific release by tag name
    pub async fn find_release_by_tag(&self, tag_name: &str) -> Result<Option<Release>> {
        let releases = self.fetch_releases().await?;
        
        // Normalize tag name (handle both "v1.0.0" and "1.0.0" formats)
        let normalized_target = self.normalize_tag_name(tag_name);
        
        for release in releases {
            let normalized_release_tag = self.normalize_tag_name(&release.tag_name);
            if normalized_release_tag == normalized_target {
                return Ok(Some(release));
            }
        }
        
        Ok(None)
    }

    /// Check if the feeds service is available
    pub async fn check_availability(&self) -> Result<bool> {
        let feed_url = self.construct_feed_url();
        
        if self.verbose {
            eprintln!("[FEEDS] Checking feed availability: {}", feed_url);
        }

        match self.client.head(&feed_url).send().await {
            Ok(response) => {
                let available = response.status().is_success();
                if self.verbose {
                    eprintln!("[FEEDS] Feed availability check: {} (HTTP {})", 
                        if available { "available" } else { "unavailable" },
                        response.status()
                    );
                }
                Ok(available)
            }
            Err(e) => {
                if self.verbose {
                    eprintln!("[FEEDS] Feed availability check failed: {}", e);
                }
                Ok(false)
            }
        }
    }

    /// Construct the Atom feed URL from repository URL
    fn construct_feed_url(&self) -> String {
        format!("{}/releases.atom", self.repo_url)
    }

    /// Convert a feed entry to a Release struct
    fn convert_entry_to_release(&self, entry: feed_rs::model::Entry) -> Option<Release> {
        // Extract tag name from entry ID or title
        let tag_name = self.extract_tag_name(&entry)?;
        
        // Use entry title or fallback to tag name
        let name = entry.title
            .as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_else(|| format!("Release {}", tag_name));

        // Format published date
        let published_at = entry.published
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| entry.updated.map(|dt| dt.to_rfc3339()).unwrap_or_default());

        // Get HTML URL from links
        let html_url = entry.links
            .iter()
            .find(|link| link.rel == Some("alternate".to_string()) || link.media_type == Some("text/html".to_string()))
            .map(|link| link.href.clone())
            .unwrap_or_else(|| format!("{}/releases/tag/{}", self.repo_url, tag_name));

        // Extract assets information from entry content (if available)
        let assets = self.extract_assets_from_entry(&entry, &tag_name);

        // Check if this is a pre-release based on tag name or content
        let prerelease = self.is_prerelease(&tag_name, &entry);

        Some(Release::new(
            tag_name,
            name,
            published_at,
            html_url,
            assets,
            prerelease,
        ))
    }

    /// Extract tag name from feed entry
    fn extract_tag_name(&self, entry: &feed_rs::model::Entry) -> Option<String> {
        // Try to extract from entry ID first (usually contains the tag)
        if let Some(tag) = entry.id.split('/').last() {
            if !tag.is_empty() {
                return Some(tag.to_string());
            }
        }

        // Try to extract from title
        if let Some(title) = &entry.title {
            let title_content = title.content.trim();
            // Look for version patterns in title
            if let Some(captures) = regex::Regex::new(r"(v?\d+\.\d+\.\d+(?:[-+][a-zA-Z0-9.-]*)?)")
                .ok()?
                .captures(title_content)
            {
                return Some(captures[1].to_string());
            }
        }

        // Try to extract from first link that looks like a release tag
        for link in &entry.links {
            if let Some(tag) = link.href.split('/').last() {
                if tag.starts_with('v') || tag.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    return Some(tag.to_string());
                }
            }
        }

        None
    }

    /// Extract release assets from entry content (GitHub feeds don't typically include asset details)
    fn extract_assets_from_entry(&self, _entry: &feed_rs::model::Entry, tag_name: &str) -> Vec<ReleaseAsset> {
        // GitHub Atom feeds don't include detailed asset information
        // We'll create placeholder assets that would need to be populated by the GitHub API client
        // For now, return empty vector as Atom feeds are primarily for basic release info
        let _ = tag_name; // Suppress unused parameter warning
        Vec::new()
    }

    /// Check if a release is a pre-release based on tag name or content
    fn is_prerelease(&self, tag_name: &str, entry: &feed_rs::model::Entry) -> bool {
        let tag_lower = tag_name.to_lowercase();
        
        // Check tag name for pre-release indicators
        if tag_lower.contains("alpha") || tag_lower.contains("beta") || 
           tag_lower.contains("rc") || tag_lower.contains("dev") ||
           tag_lower.contains("pre") {
            return true;
        }

        // Check entry content for pre-release indicators
        if let Some(content) = &entry.content {
            let content_lower = content.body.as_ref().map(|b| b.to_lowercase()).unwrap_or_default();
            if content_lower.contains("pre-release") || content_lower.contains("prerelease") {
                return true;
            }
        }

        // Check title for pre-release indicators
        if let Some(title) = &entry.title {
            let title_lower = title.content.to_lowercase();
            if title_lower.contains("pre-release") || title_lower.contains("prerelease") ||
               title_lower.contains("alpha") || title_lower.contains("beta") {
                return true;
            }
        }

        false
    }

    /// Normalize tag name for comparison (remove 'v' prefix)
    fn normalize_tag_name(&self, tag_name: &str) -> String {
        tag_name.trim().strip_prefix('v').unwrap_or(tag_name.trim()).to_string()
    }
}

impl Default for FeedsClient {
    fn default() -> Self {
        Self::new("https://github.com/MaurUppi/network-latency-tester".to_string())
            .unwrap_or_else(|_| {
                // Fallback client if default creation fails
                Self {
                    client: Client::new(),
                    repo_url: "https://github.com/MaurUppi/network-latency-tester".to_string(),
                    verbose: false,
                }
            })
    }
}

/// Feed client statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedStats {
    /// Number of releases fetched
    pub releases_count: usize,
    /// Feed URL that was accessed
    pub feed_url: String,
    /// Whether the feed was successfully parsed
    pub parse_success: bool,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    /// Feed size in bytes
    pub feed_size_bytes: Option<usize>,
}

impl FeedStats {
    /// Create new feed statistics
    pub fn new(releases_count: usize, feed_url: String, parse_success: bool) -> Self {
        Self {
            releases_count,
            feed_url,
            parse_success,
            response_time_ms: None,
            feed_size_bytes: None,
        }
    }

    /// Format statistics for display
    pub fn format_stats(&self, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            use colored::Colorize;
            output.push_str(&format!("📡 Feed URL: {}\n", self.feed_url.cyan()));
            output.push_str(&format!("📊 Status: {}\n", 
                if self.parse_success { "Success".green() } else { "Failed".red() }
            ));
            output.push_str(&format!("📦 Releases: {}\n", self.releases_count.to_string().blue()));
            
            if let Some(response_time) = self.response_time_ms {
                output.push_str(&format!("⏱️  Response Time: {}ms\n", response_time.to_string().yellow()));
            }
            
            if let Some(size) = self.feed_size_bytes {
                output.push_str(&format!("📏 Feed Size: {} bytes\n", size.to_string().blue()));
            }
        } else {
            output.push_str(&format!("Feed URL: {}\n", self.feed_url));
            output.push_str(&format!("Status: {}\n", 
                if self.parse_success { "Success" } else { "Failed" }
            ));
            output.push_str(&format!("Releases: {}\n", self.releases_count));
            
            if let Some(response_time) = self.response_time_ms {
                output.push_str(&format!("Response Time: {}ms\n", response_time));
            }
            
            if let Some(size) = self.feed_size_bytes {
                output.push_str(&format!("Feed Size: {} bytes\n", size));
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_client() -> FeedsClient {
        FeedsClient::with_config(
            "https://github.com/test/repo".to_string(),
            true,
            Some(Duration::from_secs(1)),
        ).unwrap()
    }

    #[test]
    fn test_feeds_client_creation() {
        let client = FeedsClient::new("https://github.com/test/repo".to_string());
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.repo_url, "https://github.com/test/repo");
        assert!(!client.verbose);
    }

    #[test]
    fn test_feeds_client_with_config() {
        let client = create_test_client();
        assert_eq!(client.repo_url, "https://github.com/test/repo");
        assert!(client.verbose);
    }

    #[test]
    fn test_construct_feed_url() {
        let client = create_test_client();
        let feed_url = client.construct_feed_url();
        assert_eq!(feed_url, "https://github.com/test/repo/releases.atom");
    }

    #[test]
    fn test_normalize_tag_name() {
        let client = create_test_client();
        
        assert_eq!(client.normalize_tag_name("v1.0.0"), "1.0.0");
        assert_eq!(client.normalize_tag_name("1.0.0"), "1.0.0");
        assert_eq!(client.normalize_tag_name("  v2.1.3  "), "2.1.3");
        assert_eq!(client.normalize_tag_name("  2.1.3  "), "2.1.3");
    }

    #[test]
    fn test_default_client() {
        let client = FeedsClient::default();
        assert_eq!(client.repo_url, "https://github.com/MaurUppi/network-latency-tester");
        assert!(!client.verbose);
    }

    #[test]
    fn test_feed_stats() {
        let stats = FeedStats::new(5, "https://example.com/feed".to_string(), true);
        
        assert_eq!(stats.releases_count, 5);
        assert_eq!(stats.feed_url, "https://example.com/feed");
        assert!(stats.parse_success);
        assert!(stats.response_time_ms.is_none());
        assert!(stats.feed_size_bytes.is_none());
        
        let formatted = stats.format_stats(false);
        assert!(formatted.contains("Feed URL: https://example.com/feed"));
        assert!(formatted.contains("Status: Success"));
        assert!(formatted.contains("Releases: 5"));
    }

    // Note: More comprehensive tests involving Entry creation and parsing would require
    // a test helper that creates valid feed_rs::model::Entry structs with proper field initialization.
    // For now, we focus on testing the core client functionality and public API.
    
    // Integration tests that make actual HTTP requests would be in a separate integration test file
    // to avoid network dependencies in unit tests.
}