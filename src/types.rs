//! Type definitions and aliases

use std::net::IpAddr;
use std::time::Duration;
use serde::{Deserialize, Serialize};

// Re-export commonly used types
pub use crate::error::{AppError, Result};

/// DNS configuration variants supported by the application
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DnsConfig {
    /// Use system default DNS resolution
    System,
    /// Use custom DNS servers
    Custom { servers: Vec<IpAddr> },
    /// Use DNS-over-HTTPS with specified URL
    DoH { url: String },
}

impl DnsConfig {
    /// Get a human-readable name for this DNS configuration
    pub fn name(&self) -> String {
        match self {
            DnsConfig::System => "系统默认".to_string(),
            DnsConfig::Custom { servers } => {
                if servers.len() == 1 {
                    format!("自定义DNS ({})", servers[0])
                } else {
                    format!("自定义DNS ({} servers)", servers.len())
                }
            }
            DnsConfig::DoH { url } => {
                // Extract hostname from URL for display
                if let Ok(parsed) = url::Url::parse(url) {
                    if let Some(host) = parsed.host_str() {
                        format!("DoH ({})", host)
                    } else {
                        "DoH".to_string()
                    }
                } else {
                    "DoH".to_string()
                }
            }
        }
    }
}

/// Performance classification based on timing results
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceLevel {
    /// Good performance (< 1 second)
    Good,
    /// Moderate performance (1-3 seconds)
    Moderate,
    /// Poor performance (> 3 seconds)
    Poor,
}

impl PerformanceLevel {
    /// Classify performance based on total duration
    pub fn from_duration(duration: Duration) -> Self {
        let secs = duration.as_secs_f64();
        if secs < 1.0 {
            Self::Good
        } else if secs < 3.0 {
            Self::Moderate
        } else {
            Self::Poor
        }
    }
}

/// Test execution status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TestStatus {
    /// Test completed successfully
    Success,
    /// Test failed due to network error
    Failed,
    /// Test was skipped (e.g., unsupported DNS config)
    Skipped,
    /// Test timed out
    Timeout,
}