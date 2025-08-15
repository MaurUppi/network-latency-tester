//! Geographic detection for optimized download URLs
//!
//! This module provides IP-based location detection to determine if users
//! are in regions that require download acceleration (specifically China mainland).
//! Uses myip.ipip.net for fast, reliable geographic detection.

use crate::{AppError, Result};
use crate::updater::types::GeographicRegion;
use reqwest::Client;
use std::time::Duration;

/// Default timeout for geographic detection requests
const GEO_DETECTION_TIMEOUT: Duration = Duration::from_secs(3);

/// IP detection service URL (ipip.net provides fast, reliable service)
const IP_DETECTION_URL: &str = "https://myip.ipip.net";

/// Geographic detector for IP-based location detection
pub struct GeographicDetector {
    client: Client,
}

impl GeographicDetector {
    /// Create a new GeographicDetector
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(GEO_DETECTION_TIMEOUT)
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(|e| AppError::geographic(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client })
    }

    /// Detect user's geographic region based on IP address
    /// 
    /// Returns GeographicRegion::ChinaMainland if user is in China mainland,
    /// GeographicRegion::Global otherwise. Falls back to Global on any error
    /// to ensure download functionality is never blocked.
    pub async fn detect_region(&self) -> GeographicRegion {
        match self.perform_detection().await {
            Ok(region) => region,
            Err(e) => {
                // Log error but continue with global fallback
                eprintln!("Geographic detection failed: {}. Using global downloads.", e);
                GeographicRegion::Global
            }
        }
    }

    /// Perform the actual IP detection and region determination
    async fn perform_detection(&self) -> Result<GeographicRegion> {
        let response = self.client
            .get(IP_DETECTION_URL)
            .send()
            .await
            .map_err(|e| AppError::geographic(format!("IP detection request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::geographic(format!(
                "IP detection service returned error: {}",
                response.status()
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AppError::geographic(format!("Failed to read IP detection response: {}", e)))?;

        Ok(self.parse_location(&response_text))
    }

    /// Parse location information from IP detection response
    /// 
    /// The myip.ipip.net service returns location info in Chinese format.
    /// We look for keywords that indicate China mainland location.
    fn parse_location(&self, response: &str) -> GeographicRegion {
        let response_lower = response.to_lowercase();
        
        // Check for China mainland indicators
        // myip.ipip.net returns text like "当前 IP：xxx.xxx.xxx.xxx 来自于：中国 北京市"
        if response_lower.contains("中国") || 
           response_lower.contains("china") ||
           response_lower.contains("beijing") ||
           response_lower.contains("shanghai") ||
           response_lower.contains("guangzhou") ||
           response_lower.contains("shenzhen") {
            GeographicRegion::ChinaMainland
        } else {
            GeographicRegion::Global
        }
    }

    /// Get accelerated download URL for China mainland users
    /// 
    /// Converts GitHub URLs to accelerated alternatives using mirror services.
    /// This is a simple URL transformation - if it fails, original URL is returned.
    pub fn get_accelerated_url(&self, original_url: &str, region: &GeographicRegion) -> String {
        if !region.needs_acceleration() {
            return original_url.to_string();
        }

        // For China mainland users, use GitHub proxy services
        // These are common, reliable proxy services for GitHub content
        if original_url.contains("github.com") || original_url.contains("githubusercontent.com") {
            // Try ghproxy.com - a reliable GitHub proxy service
            if let Some(accelerated) = self.try_ghproxy_acceleration(original_url) {
                return accelerated;
            }
            
            // Fallback: try githubusercontent acceleration
            if let Some(accelerated) = self.try_githubusercontent_acceleration(original_url) {
                return accelerated;
            }
        }

        // If acceleration fails, return original URL
        original_url.to_string()
    }

    /// Try acceleration using ghproxy.com service
    fn try_ghproxy_acceleration(&self, url: &str) -> Option<String> {
        if url.starts_with("https://github.com/") || url.starts_with("https://raw.githubusercontent.com/") {
            Some(format!("https://ghproxy.com/{}", url))
        } else {
            None
        }
    }

    /// Try acceleration for raw.githubusercontent.com URLs
    fn try_githubusercontent_acceleration(&self, url: &str) -> Option<String> {
        if url.starts_with("https://raw.githubusercontent.com/") {
            // Replace with a China-friendly CDN if available
            Some(url.replace("raw.githubusercontent.com", "raw.githubusercontents.com"))
        } else {
            None
        }
    }
}

impl Default for GeographicDetector {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // If client creation fails, create a minimal fallback
            Self {
                client: Client::new(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geographic_detector_creation() {
        let detector = GeographicDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_location_parsing_comprehensive() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test China mainland detection - Chinese responses
        assert_eq!(
            detector.parse_location("当前 IP：1.2.3.4 来自于：中国 北京市"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("当前 IP：220.181.57.217 来自于：中国 上海市"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("IP地址：114.114.114.114 地区：中国 广东省 深圳市"),
            GeographicRegion::ChinaMainland
        );
        
        // Test China mainland detection - English responses
        assert_eq!(
            detector.parse_location("Current IP: 1.2.3.4 Location: China Beijing"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("IP: 220.181.57.217 Country: China City: Shanghai"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("Your IP is located in Beijing, China"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("Location: Guangzhou, China mainland"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("City: Shenzhen Country: China"),
            GeographicRegion::ChinaMainland
        );
        
        // Test global detection - various countries
        assert_eq!(
            detector.parse_location("Current IP: 8.8.8.8 Location: United States"),
            GeographicRegion::Global
        );
        
        assert_eq!(
            detector.parse_location("IP: 1.2.3.4 Japan Tokyo"),
            GeographicRegion::Global
        );
        
        assert_eq!(
            detector.parse_location("Location: London, United Kingdom"),
            GeographicRegion::Global
        );
        
        assert_eq!(
            detector.parse_location("Your IP is from Germany, Frankfurt"),
            GeographicRegion::Global
        );
        
        assert_eq!(
            detector.parse_location("IP: 203.0.113.1 Country: Australia"),
            GeographicRegion::Global
        );
        
        // Test edge cases
        assert_eq!(
            detector.parse_location(""),
            GeographicRegion::Global
        );
        
        assert_eq!(
            detector.parse_location("Invalid response format"),
            GeographicRegion::Global
        );
        
        assert_eq!(
            detector.parse_location("IP: 127.0.0.1 Location: Unknown"),
            GeographicRegion::Global
        );
        
        // Test case sensitivity
        assert_eq!(
            detector.parse_location("CURRENT IP: 1.2.3.4 LOCATION: CHINA BEIJING"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("location: SHANGHAI, china"),
            GeographicRegion::ChinaMainland
        );
    }

    #[test]
    fn test_url_acceleration_comprehensive() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test no acceleration for global region
        let global_url = detector.get_accelerated_url(
            "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz",
            &GeographicRegion::Global
        );
        assert_eq!(global_url, "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz");
        
        // Test no acceleration for unknown region
        let unknown_url = detector.get_accelerated_url(
            "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz",
            &GeographicRegion::Unknown
        );
        assert_eq!(unknown_url, "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz");
        
        // Test acceleration for China mainland - GitHub releases
        let china_url = detector.get_accelerated_url(
            "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert!(china_url.contains("ghproxy.com"));
        assert!(china_url.contains("github.com"));
        assert_eq!(china_url, "https://ghproxy.com/https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz");
        
        // Test acceleration for China mainland - raw.githubusercontent.com
        let raw_url = detector.get_accelerated_url(
            "https://raw.githubusercontent.com/user/repo/main/file.txt",
            &GeographicRegion::ChinaMainland
        );
        assert!(raw_url.contains("ghproxy.com"));
        assert_eq!(raw_url, "https://ghproxy.com/https://raw.githubusercontent.com/user/repo/main/file.txt");
        
        // Test different GitHub URL patterns
        let archive_url = detector.get_accelerated_url(
            "https://github.com/user/repo/archive/v1.0.0.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert!(archive_url.contains("ghproxy.com"));
        
        let zipball_url = detector.get_accelerated_url(
            "https://github.com/user/repo/zipball/main",
            &GeographicRegion::ChinaMainland
        );
        assert!(zipball_url.contains("ghproxy.com"));
        
        // Test non-GitHub URLs (should not be accelerated)
        let example_url = detector.get_accelerated_url(
            "https://example.com/file.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert_eq!(example_url, "https://example.com/file.tar.gz");
        
        let gitlab_url = detector.get_accelerated_url(
            "https://gitlab.com/user/repo/-/archive/main/repo-main.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert_eq!(gitlab_url, "https://gitlab.com/user/repo/-/archive/main/repo-main.tar.gz");
        
        let bitbucket_url = detector.get_accelerated_url(
            "https://bitbucket.org/user/repo/downloads/file.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert_eq!(bitbucket_url, "https://bitbucket.org/user/repo/downloads/file.tar.gz");
        
        // Test edge cases
        let empty_url = detector.get_accelerated_url("", &GeographicRegion::ChinaMainland);
        assert_eq!(empty_url, "");
        
        let malformed_url = detector.get_accelerated_url("not-a-url", &GeographicRegion::ChinaMainland);
        assert_eq!(malformed_url, "not-a-url");
    }

    #[test]
    fn test_ghproxy_acceleration_edge_cases() {
        let detector = GeographicDetector::new().unwrap();
        
        // Valid GitHub URLs
        let github_url = "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz";
        let accelerated = detector.try_ghproxy_acceleration(github_url);
        assert_eq!(
            accelerated, 
            Some("https://ghproxy.com/https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz".to_string())
        );
        
        let raw_url = "https://raw.githubusercontent.com/user/repo/main/file.txt";
        let accelerated = detector.try_ghproxy_acceleration(raw_url);
        assert_eq!(
            accelerated,
            Some("https://ghproxy.com/https://raw.githubusercontent.com/user/repo/main/file.txt".to_string())
        );
        
        // Test various GitHub URL patterns
        let archive_url = "https://github.com/user/repo/archive/refs/tags/v1.0.0.tar.gz";
        let accelerated = detector.try_ghproxy_acceleration(archive_url);
        assert_eq!(
            accelerated,
            Some("https://ghproxy.com/https://github.com/user/repo/archive/refs/tags/v1.0.0.tar.gz".to_string())
        );
        
        // Test URLs that should not be accelerated
        let other_url = "https://example.com/file.tar.gz";
        let accelerated = detector.try_ghproxy_acceleration(other_url);
        assert_eq!(accelerated, None);
        
        let github_subdomain = "https://api.github.com/repos/user/repo/releases";
        let accelerated = detector.try_ghproxy_acceleration(github_subdomain);
        assert_eq!(accelerated, None);
        
        let http_github = "http://github.com/user/repo/releases/download/v1.0.0/file.tar.gz";
        let accelerated = detector.try_ghproxy_acceleration(http_github);
        assert_eq!(accelerated, None);
        
        // Test edge cases
        let empty_url = "";
        let accelerated = detector.try_ghproxy_acceleration(empty_url);
        assert_eq!(accelerated, None);
        
        let malformed_url = "not-a-url";
        let accelerated = detector.try_ghproxy_acceleration(malformed_url);
        assert_eq!(accelerated, None);
        
        let github_partial = "github.com/user/repo";
        let accelerated = detector.try_ghproxy_acceleration(github_partial);
        assert_eq!(accelerated, None);
    }

    #[test]
    fn test_githubusercontent_acceleration_edge_cases() {
        let detector = GeographicDetector::new().unwrap();
        
        // Valid raw.githubusercontent.com URLs
        let raw_url = "https://raw.githubusercontent.com/user/repo/main/README.md";
        let accelerated = detector.try_githubusercontent_acceleration(raw_url);
        assert_eq!(
            accelerated,
            Some("https://raw.githubusercontents.com/user/repo/main/README.md".to_string())
        );
        
        let raw_url_with_branch = "https://raw.githubusercontent.com/user/repo/feature-branch/config.json";
        let accelerated = detector.try_githubusercontent_acceleration(raw_url_with_branch);
        assert_eq!(
            accelerated,
            Some("https://raw.githubusercontents.com/user/repo/feature-branch/config.json".to_string())
        );
        
        // URLs that should not be accelerated
        let github_url = "https://github.com/user/repo/blob/main/README.md";
        let accelerated = detector.try_githubusercontent_acceleration(github_url);
        assert_eq!(accelerated, None);
        
        let other_url = "https://example.com/raw/file.txt";
        let accelerated = detector.try_githubusercontent_acceleration(other_url);
        assert_eq!(accelerated, None);
        
        let github_api = "https://api.github.com/repos/user/repo/contents/file.txt";
        let accelerated = detector.try_githubusercontent_acceleration(github_api);
        assert_eq!(accelerated, None);
        
        // Test edge cases
        let empty_url = "";
        let accelerated = detector.try_githubusercontent_acceleration(empty_url);
        assert_eq!(accelerated, None);
        
        let malformed_url = "not-a-url";
        let accelerated = detector.try_githubusercontent_acceleration(malformed_url);
        assert_eq!(accelerated, None);
        
        let partial_raw = "raw.githubusercontent.com/user/repo/main/file.txt";
        let accelerated = detector.try_githubusercontent_acceleration(partial_raw);
        assert_eq!(accelerated, None);
    }

    #[test]
    fn test_default_creation() {
        let detector = GeographicDetector::default();
        // Should not panic and should be usable
        let result = detector.parse_location("test location");
        assert_eq!(result, GeographicRegion::Global);
    }

    #[test]
    fn test_region_needs_acceleration() {
        assert!(GeographicRegion::ChinaMainland.needs_acceleration());
        assert!(!GeographicRegion::Global.needs_acceleration());
        assert!(!GeographicRegion::Unknown.needs_acceleration());
    }

    #[test]
    fn test_detector_client_configuration() {
        let detector = GeographicDetector::new().unwrap();
        
        // Verify that client was created successfully
        // We can't directly test timeout configuration without making actual requests,
        // but we can verify the detector was created properly
        
        // Test that location parsing works (indicating client is functional)
        let result = detector.parse_location("Test location parsing works");
        assert_eq!(result, GeographicRegion::Global);
        
        // Test that URL acceleration works (indicating client methods are accessible)
        let accelerated = detector.get_accelerated_url(
            "https://github.com/test/test", 
            &GeographicRegion::ChinaMainland
        );
        assert!(accelerated.contains("ghproxy.com"));
    }

    #[tokio::test]
    async fn test_detect_region_graceful_fallback() {
        // This test verifies that detection gracefully falls back to Global
        // even if the detector encounters errors
        let detector = GeographicDetector::new().unwrap();
        
        // The actual detection might succeed or fail depending on network
        // but it should always return a valid GeographicRegion and never panic
        let region = detector.detect_region().await;
        assert!(matches!(region, GeographicRegion::ChinaMainland | GeographicRegion::Global));
    }

    #[tokio::test]
    async fn test_detect_region_error_handling() {
        // Create a detector with a very short timeout to simulate timeout conditions
        let short_timeout_client = Client::builder()
            .timeout(Duration::from_millis(1)) // Extremely short timeout
            .build()
            .unwrap();
        
        let detector = GeographicDetector {
            client: short_timeout_client,
        };
        
        // This should timeout quickly and fall back to Global
        let region = detector.detect_region().await;
        assert_eq!(region, GeographicRegion::Global);
    }

    #[tokio::test]
    async fn test_perform_detection_timeout_simulation() {
        // Test timeout handling by creating a client with minimal timeout
        let minimal_timeout_client = Client::builder()
            .timeout(Duration::from_millis(5)) // Very short timeout to force failure
            .build()
            .unwrap();
        
        let detector = GeographicDetector {
            client: minimal_timeout_client,
        };
        
        // This should fail due to timeout and return an error
        let result = detector.perform_detection().await;
        
        // The result should be an error (timeout or connection failure)
        assert!(result.is_err());
        if let Err(e) = result {
            let error_str = e.to_string();
            // Should be a geographic error containing timeout or request failure info
            assert!(error_str.contains("IP detection request failed") || 
                   error_str.contains("Geographic"));
        }
    }

    // ========== COMPREHENSIVE EDGE CASE TESTS ==========

    #[test]
    fn test_location_parsing_special_characters() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test various Unicode and special characters  
        assert_eq!(
            detector.parse_location("IP：192.168.1.1 地区：中国 北京市"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("Location: 中国大陆 上海市 浦东新区"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("Your IP: 114.114.114.114\nCountry: 中国\nCity: 深圳"),
            GeographicRegion::ChinaMainland
        );
        
        // Test with extra whitespace and formatting
        assert_eq!(
            detector.parse_location("   当前 IP：   1.2.3.4   来自于：   中国   北京市   "),
            GeographicRegion::ChinaMainland
        );
        
        // Test mixed case and languages
        assert_eq!(
            detector.parse_location("Location: CHINA beijing 中国"),
            GeographicRegion::ChinaMainland
        );
    }

    #[test]
    fn test_url_acceleration_protocol_variations() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test HTTPS URLs (should be accelerated)
        let https_url = detector.get_accelerated_url(
            "https://github.com/user/repo/releases/download/v1.0.0/file.zip",
            &GeographicRegion::ChinaMainland
        );
        assert!(https_url.contains("ghproxy.com"));
        
        // Test HTTP URLs (should not be accelerated by ghproxy)
        let http_url = detector.get_accelerated_url(
            "http://github.com/user/repo/releases/download/v1.0.0/file.zip",
            &GeographicRegion::ChinaMainland
        );
        assert_eq!(http_url, "http://github.com/user/repo/releases/download/v1.0.0/file.zip");
        
        // Test URLs with ports (should not be accelerated due to exact matching)
        let port_url = detector.get_accelerated_url(
            "https://github.com:443/user/repo/releases/download/v1.0.0/file.zip",
            &GeographicRegion::ChinaMainland
        );
        assert_eq!(port_url, "https://github.com:443/user/repo/releases/download/v1.0.0/file.zip");
        
        // Test URLs with query parameters
        let query_url = detector.get_accelerated_url(
            "https://github.com/user/repo/releases/download/v1.0.0/file.zip?dl=1",
            &GeographicRegion::ChinaMainland
        );
        assert!(query_url.contains("ghproxy.com"));
        
        // Test URLs with fragments
        let fragment_url = detector.get_accelerated_url(
            "https://github.com/user/repo/releases/download/v1.0.0/file.zip#readme",
            &GeographicRegion::ChinaMainland
        );
        assert!(fragment_url.contains("ghproxy.com"));
    }

    #[test]
    fn test_url_acceleration_path_variations() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test different GitHub path patterns
        let release_url = detector.get_accelerated_url(
            "https://github.com/owner/repo/releases/download/v1.2.3/binary-linux-x64.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert!(release_url.contains("ghproxy.com"));
        
        let archive_url = detector.get_accelerated_url(
            "https://github.com/owner/repo/archive/refs/heads/main.zip",
            &GeographicRegion::ChinaMainland
        );
        assert!(archive_url.contains("ghproxy.com"));
        
        let tarball_url = detector.get_accelerated_url(
            "https://github.com/owner/repo/tarball/v1.0.0",
            &GeographicRegion::ChinaMainland
        );
        assert!(tarball_url.contains("ghproxy.com"));
        
        // Test raw.githubusercontent.com variations
        let raw_main_url = detector.get_accelerated_url(
            "https://raw.githubusercontent.com/owner/repo/main/README.md",
            &GeographicRegion::ChinaMainland
        );
        assert!(raw_main_url.contains("ghproxy.com"));
        
        let raw_commit_url = detector.get_accelerated_url(
            "https://raw.githubusercontent.com/owner/repo/abc123def456/config.yaml",
            &GeographicRegion::ChinaMainland
        );
        assert!(raw_commit_url.contains("ghproxy.com"));
        
        let raw_nested_url = detector.get_accelerated_url(
            "https://raw.githubusercontent.com/owner/repo/main/docs/guide/setup.md",
            &GeographicRegion::ChinaMainland
        );
        assert!(raw_nested_url.contains("ghproxy.com"));
    }

    #[test]
    fn test_error_conditions_and_edge_cases() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test extremely long URLs
        let long_url = format!("https://github.com/user/repo/releases/download/v1.0.0/{}.tar.gz", "a".repeat(1000));
        let accelerated = detector.get_accelerated_url(&long_url, &GeographicRegion::ChinaMainland);
        assert!(accelerated.contains("ghproxy.com"));
        assert!(accelerated.len() > long_url.len()); // Should be longer due to proxy prefix
        
        // Test URLs with special characters in path
        let special_char_url = detector.get_accelerated_url(
            "https://github.com/user/repo/releases/download/v1.0.0/file-name_with.special%20chars.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert!(special_char_url.contains("ghproxy.com"));
        
        // Test malformed but partially valid URLs
        let partial_url = detector.get_accelerated_url(
            "github.com/user/repo/releases/download/v1.0.0/file.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert_eq!(partial_url, "github.com/user/repo/releases/download/v1.0.0/file.tar.gz"); // No acceleration
        
        // Test empty and null-like inputs
        assert_eq!(detector.parse_location(""), GeographicRegion::Global);
        assert_eq!(detector.parse_location(" "), GeographicRegion::Global);
        assert_eq!(detector.parse_location("\n\t\r"), GeographicRegion::Global);
        
        let empty_accelerated = detector.get_accelerated_url("", &GeographicRegion::ChinaMainland);
        assert_eq!(empty_accelerated, "");
        
        let whitespace_accelerated = detector.get_accelerated_url("   ", &GeographicRegion::ChinaMainland);
        assert_eq!(whitespace_accelerated, "   ");
    }

    #[test]
    fn test_region_behavior_consistency() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test that the same input always produces the same output
        let test_responses = vec![
            "当前 IP：1.2.3.4 来自于：中国 北京市",
            "Current IP: 8.8.8.8 Location: United States",
            "IP: 1.2.3.4 Japan Tokyo",
            "",
            "Invalid response",
        ];
        
        for response in &test_responses {
            let result1 = detector.parse_location(response);
            let result2 = detector.parse_location(response);
            assert_eq!(result1, result2, "Inconsistent parsing for: {}", response);
        }
        
        // Test URL acceleration consistency
        let test_urls = vec![
            "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz",
            "https://raw.githubusercontent.com/user/repo/main/file.txt",
            "https://example.com/file.tar.gz",
            "",
        ];
        
        for url in &test_urls {
            let result1 = detector.get_accelerated_url(url, &GeographicRegion::ChinaMainland);
            let result2 = detector.get_accelerated_url(url, &GeographicRegion::ChinaMainland);
            assert_eq!(result1, result2, "Inconsistent acceleration for: {}", url);
            
            let global_result1 = detector.get_accelerated_url(url, &GeographicRegion::Global);
            let global_result2 = detector.get_accelerated_url(url, &GeographicRegion::Global);
            assert_eq!(global_result1, global_result2, "Inconsistent global handling for: {}", url);
        }
    }

    #[tokio::test]
    async fn test_concurrent_region_detection() {
        // Test that concurrent detection calls work correctly
        
        // Launch multiple concurrent detection tasks
        let tasks = (0..5).map(|_| {
            let detector_clone = GeographicDetector::new().unwrap();
            tokio::spawn(async move {
                detector_clone.detect_region().await
            })
        }).collect::<Vec<_>>();
        
        // Wait for all tasks to complete
        let mut results = Vec::new();
        for task in tasks {
            results.push(task.await);
        }
        
        // All should succeed and return valid regions
        for result in results {
            let region = result.unwrap(); // Task should not panic
            assert!(matches!(region, GeographicRegion::ChinaMainland | GeographicRegion::Global));
        }
    }

    #[test]
    fn test_detector_memory_efficiency() {
        // Test that creating multiple detectors doesn't cause issues
        let detectors: Vec<GeographicDetector> = (0..100)
            .map(|_| GeographicDetector::new().unwrap())
            .collect();
        
        // All detectors should work correctly
        for (i, detector) in detectors.iter().enumerate() {
            let result = detector.parse_location(&format!("Test location {}", i));
            assert_eq!(result, GeographicRegion::Global);
            
            let accelerated = detector.get_accelerated_url(
                "https://github.com/test/test", 
                &GeographicRegion::ChinaMainland
            );
            assert!(accelerated.contains("ghproxy.com"));
        }
    }

    // ========== MOCK-BASED TESTS FOR EXTERNAL SERVICE SIMULATION ==========

    #[test]
    fn test_mock_response_scenarios() {
        let detector = GeographicDetector::new().unwrap();
        
        // Simulate various response scenarios that might come from myip.ipip.net
        let mock_responses = vec![
            // Chinese responses (should detect China mainland)
            ("当前 IP：220.181.57.217 来自于：中国 北京市 联通", GeographicRegion::ChinaMainland),
            ("当前 IP：114.114.114.114 来自于：中国 上海市 电信", GeographicRegion::ChinaMainland),
            ("当前 IP：183.232.231.172 来自于：中国 广东省 深圳市 移动", GeographicRegion::ChinaMainland),
            ("IP地址：202.96.209.133 归属地：中国 江苏省 南京市", GeographicRegion::ChinaMainland),
            
            // English responses (should detect China mainland if contains China)
            ("Current IP: 220.181.57.217 Location: Beijing, China", GeographicRegion::ChinaMainland),
            ("Your IP 114.114.114.114 is located in Shanghai, China", GeographicRegion::ChinaMainland),
            ("IP: 183.232.231.172 Country: China City: Guangzhou", GeographicRegion::ChinaMainland),
            
            // Global responses (should detect Global)
            ("Current IP: 8.8.8.8 Location: Mountain View, United States", GeographicRegion::Global),
            ("Your IP 1.1.1.1 is located in Australia", GeographicRegion::Global),
            ("IP: 208.67.222.222 Country: United States", GeographicRegion::Global),
            ("Current IP: 77.88.8.8 Location: Netherlands", GeographicRegion::Global),
            
            // Edge case responses (should default to Global)
            ("Service temporarily unavailable", GeographicRegion::Global),
            ("Error: Unable to determine location", GeographicRegion::Global),
            ("Invalid request format", GeographicRegion::Global),
            ("", GeographicRegion::Global),
            ("    ", GeographicRegion::Global),
            ("NULL", GeographicRegion::Global),
            ("404 Not Found", GeographicRegion::Global),
            
            // Malformed but parseable responses
            ("IP地址解析失败，返回默认位置：中国", GeographicRegion::ChinaMainland),
            ("Location service error, detected: China Beijing", GeographicRegion::ChinaMainland),
            ("Partial response: ...China...", GeographicRegion::ChinaMainland),
        ];
        
        for (mock_response, expected_region) in mock_responses {
            let result = detector.parse_location(mock_response);
            assert_eq!(
                result, 
                expected_region,
                "Failed to parse response: '{}' - expected {:?}, got {:?}",
                mock_response, expected_region, result
            );
        }
    }

    #[test]
    fn test_http_error_simulation() {
        // Test various HTTP error scenarios that could occur
        let detector = GeographicDetector::new().unwrap();
        
        // These would simulate HTTP error responses that the real service might return
        let error_scenarios = vec![
            "HTTP 500 Internal Server Error",
            "HTTP 503 Service Unavailable", 
            "HTTP 429 Too Many Requests",
            "HTTP 404 Not Found",
            "Connection timed out",
            "DNS resolution failed",
            "Network unreachable",
        ];
        
        // Since we can't easily mock HTTP responses in unit tests without additional dependencies,
        // we test the parse_location logic that would handle such error responses
        for error_response in error_scenarios {
            let result = detector.parse_location(error_response);
            // All error responses should fall back to Global
            assert_eq!(result, GeographicRegion::Global, 
                      "Error response '{}' should fallback to Global", error_response);
        }
    }

    #[tokio::test] 
    async fn test_timeout_behavior_simulation() {
        // Test timeout handling by using a client with aggressive timeout
        let timeout_detector = GeographicDetector {
            client: Client::builder()
                .timeout(Duration::from_millis(1)) // 1ms timeout - should fail quickly
                .build()
                .unwrap(),
        };
        
        // This should timeout and gracefully fall back to Global
        let start_time = std::time::Instant::now();
        let region = timeout_detector.detect_region().await;
        let elapsed = start_time.elapsed();
        
        // Should return Global due to timeout
        assert_eq!(region, GeographicRegion::Global);
        
        // Should complete relatively quickly (within reasonable bounds)
        // Allow some extra time for system scheduling and error handling
        assert!(elapsed < Duration::from_millis(5000), 
               "Detection took too long: {:?}", elapsed);
    }

    #[test]
    fn test_real_myip_ipip_net_response_format() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test the exact response format from user's curl command
        let real_response = "当前 IP：113.74.8.52  来自于：中国 广东 珠海  电信";
        let result = detector.parse_location(real_response);
        assert_eq!(result, GeographicRegion::ChinaMainland, 
                  "Real myip.ipip.net response should be parsed as China mainland");
        
        // Test other similar real formats
        let guangzhou_response = "当前 IP：14.215.177.38  来自于：中国 广东 广州  联通";
        assert_eq!(detector.parse_location(guangzhou_response), GeographicRegion::ChinaMainland);
        
        let beijing_response = "当前 IP：220.181.57.217  来自于：中国 北京  电信";
        assert_eq!(detector.parse_location(beijing_response), GeographicRegion::ChinaMainland);
        
        let shanghai_response = "当前 IP：101.95.46.178  来自于：中国 上海  移动";
        assert_eq!(detector.parse_location(shanghai_response), GeographicRegion::ChinaMainland);
        
        // Test non-China response for comparison
        let us_response = "当前 IP：8.8.8.8  来自于：美国 加利福尼亚州 山景城  谷歌公司";
        assert_eq!(detector.parse_location(us_response), GeographicRegion::Global);
    }

    #[test]
    fn test_fallback_acceleration_logic() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test the fallback chain for URL acceleration
        let test_urls = vec![
            // Should use ghproxy for GitHub URLs
            "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz",
            "https://github.com/user/repo/archive/main.zip",
            
            // Should use ghproxy for raw.githubusercontent.com
            "https://raw.githubusercontent.com/user/repo/main/file.txt",
            "https://raw.githubusercontent.com/user/repo/branch/config.json",
        ];
        
        for url in test_urls {
            let accelerated = detector.get_accelerated_url(url, &GeographicRegion::ChinaMainland);
            
            // Should be accelerated with ghproxy
            assert!(accelerated.contains("ghproxy.com"), 
                   "URL '{}' should be accelerated with ghproxy, got '{}'", url, accelerated);
            
            // Should contain the original URL
            assert!(accelerated.contains(url),
                   "Accelerated URL '{}' should contain original URL '{}'", accelerated, url);
        }
        
        // Test URLs that should NOT be accelerated
        let non_github_urls = vec![
            "https://gitlab.com/user/repo/-/archive/main/repo-main.tar.gz",
            "https://bitbucket.org/user/repo/downloads/file.tar.gz", 
            "https://example.com/downloads/file.tar.gz",
            "https://sourceforge.net/projects/project/files/file.tar.gz",
        ];
        
        for url in non_github_urls {
            let accelerated = detector.get_accelerated_url(url, &GeographicRegion::ChinaMainland);
            
            // Should NOT be accelerated (should return original URL)
            assert_eq!(accelerated, url,
                      "Non-GitHub URL '{}' should not be accelerated", url);
        }
    }
}