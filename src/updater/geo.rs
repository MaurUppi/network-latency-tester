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
    fn test_location_parsing() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test China mainland detection
        assert_eq!(
            detector.parse_location("当前 IP：1.2.3.4 来自于：中国 北京市"),
            GeographicRegion::ChinaMainland
        );
        
        assert_eq!(
            detector.parse_location("Current IP: 1.2.3.4 Location: China Beijing"),
            GeographicRegion::ChinaMainland
        );
        
        // Test global detection
        assert_eq!(
            detector.parse_location("Current IP: 1.2.3.4 Location: United States"),
            GeographicRegion::Global
        );
        
        assert_eq!(
            detector.parse_location("IP: 1.2.3.4 Japan Tokyo"),
            GeographicRegion::Global
        );
    }

    #[test]
    fn test_url_acceleration() {
        let detector = GeographicDetector::new().unwrap();
        
        // Test no acceleration for global region
        let global_url = detector.get_accelerated_url(
            "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz",
            &GeographicRegion::Global
        );
        assert_eq!(global_url, "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz");
        
        // Test acceleration for China mainland
        let china_url = detector.get_accelerated_url(
            "https://github.com/user/repo/releases/download/v1.0.0/file.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert!(china_url.contains("ghproxy.com"));
        assert!(china_url.contains("github.com"));
        
        // Test raw.githubusercontent.com acceleration
        let raw_url = detector.get_accelerated_url(
            "https://raw.githubusercontent.com/user/repo/main/file.txt",
            &GeographicRegion::ChinaMainland
        );
        assert!(raw_url.contains("ghproxy.com"));
        
        // Test non-GitHub URL (no acceleration)
        let other_url = detector.get_accelerated_url(
            "https://example.com/file.tar.gz",
            &GeographicRegion::ChinaMainland
        );
        assert_eq!(other_url, "https://example.com/file.tar.gz");
    }

    #[test]
    fn test_ghproxy_acceleration() {
        let detector = GeographicDetector::new().unwrap();
        
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
        
        let other_url = "https://example.com/file.tar.gz";
        let accelerated = detector.try_ghproxy_acceleration(other_url);
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

    #[tokio::test]
    async fn test_detect_region_fallback() {
        // This test verifies that detection gracefully falls back to Global
        // even if the detector encounters errors
        let detector = GeographicDetector::new().unwrap();
        
        // The actual detection might succeed or fail depending on network
        // but it should always return a valid GeographicRegion
        let region = detector.detect_region().await;
        assert!(matches!(region, GeographicRegion::ChinaMainland | GeographicRegion::Global));
    }
}