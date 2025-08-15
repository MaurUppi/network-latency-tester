//! GitHub REST API client for detailed release information
//!
//! This module provides a GitHub REST API client that serves as a fallback
//! data source when GitHub Atom feeds don't provide sufficient detail about
//! releases and their assets. It includes rate limit handling, error recovery,
//! and comprehensive release asset information.

use crate::{AppError, Result};
use crate::updater::types::{Release, ReleaseAsset};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default timeout for GitHub API requests
const GITHUB_API_TIMEOUT: Duration = Duration::from_secs(10);

/// GitHub API base URL
const GITHUB_API_BASE: &str = "https://api.github.com";

/// Rate limit information from GitHub API headers
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Remaining requests in current window
    pub remaining: u32,
    /// Rate limit reset time (Unix timestamp)
    pub reset: u64,
    /// Total rate limit for current window
    pub limit: u32,
}

/// GitHub API release response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubApiRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub published_at: Option<String>,
    pub html_url: String,
    pub assets: Vec<GitHubApiAsset>,
    pub prerelease: bool,
    pub draft: bool,
}

/// GitHub API asset response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubApiAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub content_type: String,
}

/// GitHub REST API client
pub struct GitHubApiClient {
    /// HTTP client for requests
    client: Client,
    /// Repository owner
    repo_owner: String,
    /// Repository name
    repo_name: String,
    /// Whether verbose logging is enabled
    verbose: bool,
    /// API token for authenticated requests (optional)
    api_token: Option<String>,
}

impl GitHubApiClient {
    /// Create a new GitHubApiClient with default settings
    pub fn new(repo_owner: String, repo_name: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(GITHUB_API_TIMEOUT)
            .user_agent(format!(
                "network-latency-tester/{} (https://github.com/MaurUppi/network-latency-tester)",
                crate::VERSION
            ))
            .build()
            .map_err(|e| AppError::update(format!("Failed to create GitHub API client: {}", e)))?;

        Ok(Self {
            client,
            repo_owner,
            repo_name,
            verbose: false,
            api_token: None,
        })
    }

    /// Create a new GitHubApiClient with configuration
    pub fn with_config(
        repo_owner: String,
        repo_name: String,
        verbose: bool,
        timeout: Option<Duration>,
        api_token: Option<String>,
    ) -> Result<Self> {
        let timeout = timeout.unwrap_or(GITHUB_API_TIMEOUT);
        
        let client = Client::builder()
            .timeout(timeout)
            .user_agent(format!(
                "network-latency-tester/{} (https://github.com/MaurUppi/network-latency-tester)",
                crate::VERSION
            ))
            .build()
            .map_err(|e| AppError::update(format!("Failed to create GitHub API client: {}", e)))?;

        Ok(Self {
            client,
            repo_owner,
            repo_name,
            verbose,
            api_token,
        })
    }

    /// Fetch releases from GitHub REST API
    pub async fn fetch_releases(&self) -> Result<Vec<Release>> {
        let url = format!("{}/repos/{}/{}/releases", GITHUB_API_BASE, self.repo_owner, self.repo_name);
        
        if self.verbose {
            eprintln!("[GITHUB] Fetching releases from: {}", url);
        }

        let response = self.make_api_request(&url).await?;
        
        // Check rate limit before processing response
        if let Some(rate_limit) = self.extract_rate_limit_info(&response) {
            if self.verbose {
                eprintln!("[GITHUB] Rate limit: {}/{} remaining, resets at {}", 
                    rate_limit.remaining, rate_limit.limit, rate_limit.reset);
            }
            
            if rate_limit.remaining < 10 {
                eprintln!("[GITHUB] WARNING: GitHub API rate limit low ({} remaining)", rate_limit.remaining);
            }
        }

        // Parse response
        let github_releases: Vec<GitHubApiRelease> = response
            .json()
            .await
            .map_err(|e| AppError::update(format!("Failed to parse GitHub API response: {}", e)))?;

        if self.verbose {
            eprintln!("[GITHUB] Parsed {} releases from API", github_releases.len());
        }

        // Convert GitHub API releases to our Release format
        let releases: Vec<Release> = github_releases
            .into_iter()
            .filter(|r| !r.draft) // Filter out draft releases
            .map(|github_release| self.convert_github_release_to_release(github_release))
            .collect();

        if self.verbose {
            eprintln!("[GITHUB] Converted {} releases (excluding drafts)", releases.len());
        }

        Ok(releases)
    }

    /// Fetch a specific number of recent releases
    pub async fn fetch_recent_releases(&self, limit: usize) -> Result<Vec<Release>> {
        let url = format!("{}/repos/{}/{}/releases?per_page={}", 
            GITHUB_API_BASE, self.repo_owner, self.repo_name, limit.min(100));
        
        if self.verbose {
            eprintln!("[GITHUB] Fetching {} recent releases from: {}", limit, url);
        }

        let response = self.make_api_request(&url).await?;
        
        let github_releases: Vec<GitHubApiRelease> = response
            .json()
            .await
            .map_err(|e| AppError::update(format!("Failed to parse GitHub API response: {}", e)))?;

        let releases: Vec<Release> = github_releases
            .into_iter()
            .filter(|r| !r.draft)
            .take(limit)
            .map(|github_release| self.convert_github_release_to_release(github_release))
            .collect();

        if self.verbose {
            eprintln!("[GITHUB] Retrieved {} recent releases", releases.len());
        }

        Ok(releases)
    }

    /// Find a specific release by tag name
    pub async fn find_release_by_tag(&self, tag_name: &str) -> Result<Option<Release>> {
        let url = format!("{}/repos/{}/{}/releases/tags/{}", 
            GITHUB_API_BASE, self.repo_owner, self.repo_name, tag_name);
        
        if self.verbose {
            eprintln!("[GITHUB] Fetching release for tag '{}': {}", tag_name, url);
        }

        match self.make_api_request(&url).await {
            Ok(response) => {
                let github_release: GitHubApiRelease = response
                    .json()
                    .await
                    .map_err(|e| AppError::update(format!("Failed to parse GitHub API response: {}", e)))?;

                if github_release.draft {
                    if self.verbose {
                        eprintln!("[GITHUB] Release '{}' is a draft, skipping", tag_name);
                    }
                    return Ok(None);
                }

                let release = self.convert_github_release_to_release(github_release);
                if self.verbose {
                    eprintln!("[GITHUB] Found release for tag '{}'", tag_name);
                }
                Ok(Some(release))
            }
            Err(e) => {
                // Check if it's a 404 (not found) error
                if e.to_string().contains("404") {
                    if self.verbose {
                        eprintln!("[GITHUB] Release not found for tag '{}'", tag_name);
                    }
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Get detailed asset information for a release
    pub async fn get_release_assets(&self, tag_name: &str) -> Result<Vec<ReleaseAsset>> {
        match self.find_release_by_tag(tag_name).await? {
            Some(release) => Ok(release.assets),
            None => Ok(vec![]),
        }
    }

    /// Check GitHub API availability and rate limits
    pub async fn check_api_availability(&self) -> Result<ApiAvailability> {
        let url = format!("{}/repos/{}/{}", GITHUB_API_BASE, self.repo_owner, self.repo_name);
        
        if self.verbose {
            eprintln!("[GITHUB] Checking API availability: {}", url);
        }

        match self.make_api_request(&url).await {
            Ok(response) => {
                let rate_limit = self.extract_rate_limit_info(&response);
                
                if self.verbose {
                    if let Some(ref limit_info) = rate_limit {
                        eprintln!("[GITHUB] API available, rate limit: {}/{}", 
                            limit_info.remaining, limit_info.limit);
                    } else {
                        eprintln!("[GITHUB] API available, no rate limit info");
                    }
                }

                Ok(ApiAvailability {
                    available: true,
                    rate_limit,
                    error_message: None,
                })
            }
            Err(e) => {
                if self.verbose {
                    eprintln!("[GITHUB] API check failed: {}", e);
                }
                Ok(ApiAvailability {
                    available: false,
                    rate_limit: None,
                    error_message: Some(e.to_string()),
                })
            }
        }
    }

    /// Make an authenticated API request with rate limit handling
    async fn make_api_request(&self, url: &str) -> Result<Response> {
        let mut request = self.client.get(url);

        // Add authentication header if token is available
        if let Some(ref token) = self.api_token {
            request = request.header("Authorization", format!("token {}", token));
        }

        // Add GitHub API version header
        request = request.header("Accept", "application/vnd.github.v3+json");

        let response = request
            .send()
            .await
            .map_err(|e| AppError::update(format!("GitHub API request failed for '{}': {}", url, e)))?;

        // Handle rate limiting
        if response.status() == StatusCode::FORBIDDEN {
            if let Some(rate_limit) = self.extract_rate_limit_info(&response) {
                if rate_limit.remaining == 0 {
                    return Err(AppError::update(format!(
                        "GitHub API rate limit exceeded. Limit resets at Unix timestamp: {}",
                        rate_limit.reset
                    )));
                }
            }
        }

        // Check for other error status codes
        if !response.status().is_success() {
            return Err(AppError::update(format!(
                "GitHub API request failed: HTTP {} for '{}'",
                response.status(),
                url
            )));
        }

        Ok(response)
    }

    /// Extract rate limit information from response headers
    fn extract_rate_limit_info(&self, response: &Response) -> Option<RateLimitInfo> {
        let headers = response.headers();
        
        let remaining = headers
            .get("x-ratelimit-remaining")?
            .to_str().ok()?
            .parse::<u32>().ok()?;
            
        let reset = headers
            .get("x-ratelimit-reset")?
            .to_str().ok()?
            .parse::<u64>().ok()?;
            
        let limit = headers
            .get("x-ratelimit-limit")?
            .to_str().ok()?
            .parse::<u32>().ok()?;

        Some(RateLimitInfo {
            remaining,
            reset,
            limit,
        })
    }

    /// Convert GitHub API release to our Release format
    fn convert_github_release_to_release(&self, github_release: GitHubApiRelease) -> Release {
        let assets: Vec<ReleaseAsset> = github_release.assets
            .into_iter()
            .map(|asset| ReleaseAsset {
                name: asset.name,
                browser_download_url: asset.browser_download_url,
                size: asset.size,
                content_type: asset.content_type,
            })
            .collect();

        Release::new(
            github_release.tag_name,
            github_release.name.unwrap_or_else(|| "Unnamed Release".to_string()),
            github_release.published_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            github_release.html_url,
            assets,
            github_release.prerelease,
        )
    }
}

impl Default for GitHubApiClient {
    fn default() -> Self {
        Self::new(
            "MaurUppi".to_string(),
            "network-latency-tester".to_string(),
        ).unwrap_or_else(|_| {
            // Fallback client if default creation fails
            Self {
                client: Client::new(),
                repo_owner: "MaurUppi".to_string(),
                repo_name: "network-latency-tester".to_string(),
                verbose: false,
                api_token: None,
            }
        })
    }
}

/// API availability information
#[derive(Debug, Clone)]
pub struct ApiAvailability {
    /// Whether the API is available
    pub available: bool,
    /// Current rate limit information
    pub rate_limit: Option<RateLimitInfo>,
    /// Error message if unavailable
    pub error_message: Option<String>,
}

impl ApiAvailability {
    /// Format availability information for display
    pub fn format_availability(&self, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            use colored::Colorize;
            output.push_str(&format!("🌐 GitHub API: {}\n", 
                if self.available { "Available".green() } else { "Unavailable".red() }
            ));
            
            if let Some(ref rate_limit) = self.rate_limit {
                output.push_str(&format!("📊 Rate Limit: {}/{}\n", 
                    rate_limit.remaining.to_string().blue(),
                    rate_limit.limit.to_string().blue()
                ));
                output.push_str(&format!("🔄 Resets: {}\n", 
                    rate_limit.reset.to_string().yellow()
                ));
            }
            
            if let Some(ref error) = self.error_message {
                output.push_str(&format!("❌ Error: {}\n", error.red()));
            }
        } else {
            output.push_str(&format!("GitHub API: {}\n", 
                if self.available { "Available" } else { "Unavailable" }
            ));
            
            if let Some(ref rate_limit) = self.rate_limit {
                output.push_str(&format!("Rate Limit: {}/{}\n", 
                    rate_limit.remaining, rate_limit.limit
                ));
                output.push_str(&format!("Resets: {}\n", rate_limit.reset));
            }
            
            if let Some(ref error) = self.error_message {
                output.push_str(&format!("Error: {}\n", error));
            }
        }
        
        output
    }
}

/// GitHub API client statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubApiStats {
    /// Number of releases fetched
    pub releases_count: usize,
    /// Number of API requests made
    pub requests_made: u32,
    /// Whether API is available
    pub api_available: bool,
    /// Current rate limit information
    pub rate_limit_remaining: Option<u32>,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
}

impl GitHubApiStats {
    /// Create new API statistics
    pub fn new(releases_count: usize, api_available: bool) -> Self {
        Self {
            releases_count,
            requests_made: 1,
            api_available,
            rate_limit_remaining: None,
            response_time_ms: None,
        }
    }

    /// Format statistics for display
    pub fn format_stats(&self, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            use colored::Colorize;
            output.push_str(&format!("🔗 GitHub API Status: {}\n", 
                if self.api_available { "Available".green() } else { "Unavailable".red() }
            ));
            output.push_str(&format!("📦 Releases Fetched: {}\n", 
                self.releases_count.to_string().blue()
            ));
            output.push_str(&format!("📡 Requests Made: {}\n", 
                self.requests_made.to_string().blue()
            ));
            
            if let Some(remaining) = self.rate_limit_remaining {
                output.push_str(&format!("⏱️  Rate Limit Remaining: {}\n", 
                    remaining.to_string().yellow()
                ));
            }
            
            if let Some(response_time) = self.response_time_ms {
                output.push_str(&format!("⚡ Response Time: {}ms\n", 
                    response_time.to_string().yellow()
                ));
            }
        } else {
            output.push_str(&format!("GitHub API Status: {}\n", 
                if self.api_available { "Available" } else { "Unavailable" }
            ));
            output.push_str(&format!("Releases Fetched: {}\n", self.releases_count));
            output.push_str(&format!("Requests Made: {}\n", self.requests_made));
            
            if let Some(remaining) = self.rate_limit_remaining {
                output.push_str(&format!("Rate Limit Remaining: {}\n", remaining));
            }
            
            if let Some(response_time) = self.response_time_ms {
                output.push_str(&format!("Response Time: {}ms\n", response_time));
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_client() -> GitHubApiClient {
        GitHubApiClient::with_config(
            "test".to_string(),
            "repo".to_string(),
            true,
            Some(Duration::from_secs(1)),
            None,
        ).unwrap()
    }

    #[test]
    fn test_github_api_client_creation() {
        let client = GitHubApiClient::new("owner".to_string(), "repo".to_string());
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.repo_owner, "owner");
        assert_eq!(client.repo_name, "repo");
        assert!(!client.verbose);
        assert!(client.api_token.is_none());
    }

    #[test]
    fn test_github_api_client_with_config() {
        let client = create_test_client();
        assert_eq!(client.repo_owner, "test");
        assert_eq!(client.repo_name, "repo");
        assert!(client.verbose);
        assert!(client.api_token.is_none());
    }

    #[test]
    fn test_default_client() {
        let client = GitHubApiClient::default();
        assert_eq!(client.repo_owner, "MaurUppi");
        assert_eq!(client.repo_name, "network-latency-tester");
        assert!(!client.verbose);
    }

    #[test]
    fn test_convert_github_release_to_release() {
        let client = create_test_client();
        
        let github_asset = GitHubApiAsset {
            name: "binary.tar.gz".to_string(),
            browser_download_url: "https://github.com/owner/repo/releases/download/v1.0.0/binary.tar.gz".to_string(),
            size: 1024,
            content_type: "application/gzip".to_string(),
        };
        
        let github_release = GitHubApiRelease {
            tag_name: "v1.0.0".to_string(),
            name: Some("Release 1.0.0".to_string()),
            published_at: Some("2024-01-01T00:00:00Z".to_string()),
            html_url: "https://github.com/owner/repo/releases/tag/v1.0.0".to_string(),
            assets: vec![github_asset],
            prerelease: false,
            draft: false,
        };
        
        let release = client.convert_github_release_to_release(github_release);
        
        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.name, "Release 1.0.0");
        assert_eq!(release.published_at, "2024-01-01T00:00:00Z");
        assert_eq!(release.html_url, "https://github.com/owner/repo/releases/tag/v1.0.0");
        assert_eq!(release.assets.len(), 1);
        assert!(!release.prerelease);
        
        let asset = &release.assets[0];
        assert_eq!(asset.name, "binary.tar.gz");
        assert_eq!(asset.size, 1024);
        assert_eq!(asset.content_type, "application/gzip");
    }

    #[test]
    fn test_rate_limit_info() {
        let rate_limit = RateLimitInfo {
            remaining: 4999,
            reset: 1640995200,
            limit: 5000,
        };
        
        assert_eq!(rate_limit.remaining, 4999);
        assert_eq!(rate_limit.reset, 1640995200);
        assert_eq!(rate_limit.limit, 5000);
    }

    #[test]
    fn test_api_availability() {
        let availability = ApiAvailability {
            available: true,
            rate_limit: Some(RateLimitInfo {
                remaining: 1000,
                reset: 1640995200,
                limit: 5000,
            }),
            error_message: None,
        };
        
        assert!(availability.available);
        assert!(availability.rate_limit.is_some());
        assert!(availability.error_message.is_none());
        
        let formatted = availability.format_availability(false);
        assert!(formatted.contains("GitHub API: Available"));
        assert!(formatted.contains("Rate Limit: 1000/5000"));
    }

    #[test]
    fn test_github_api_stats() {
        let stats = GitHubApiStats::new(10, true);
        
        assert_eq!(stats.releases_count, 10);
        assert_eq!(stats.requests_made, 1);
        assert!(stats.api_available);
        assert!(stats.rate_limit_remaining.is_none());
        
        let formatted = stats.format_stats(false);
        assert!(formatted.contains("GitHub API Status: Available"));
        assert!(formatted.contains("Releases Fetched: 10"));
        assert!(formatted.contains("Requests Made: 1"));
    }

    // Note: More comprehensive tests involving actual HTTP requests would require
    // a test helper that mocks the GitHub API responses. For now, we focus on
    // testing the core client functionality and data conversion logic.
    
    // Integration tests that make actual HTTP requests would be in a separate
    // integration test file to avoid network dependencies in unit tests.
}