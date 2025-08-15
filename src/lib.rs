//! Network Latency Tester
//! 
//! A high-performance network latency testing tool that measures connectivity
//! to configurable target URLs using various DNS configurations including
//! custom DNS servers and DNS-over-HTTPS providers.

pub mod app;
pub mod cli;
pub mod config;
pub mod client;
pub mod dns;
pub mod error;
pub mod logging;
pub mod stats;
pub mod diagnostics;
pub mod executor;
pub mod output;
pub mod models;
pub mod types;
pub mod updater;
pub mod utils;

// Re-export commonly used types
pub use error::{AppError, Result};
pub use models::{Config, TimingMetrics, TestResult, Statistics};
pub use stats::{StatisticsEngine, StatisticalAnalysis, ExtendedStatistics, OptimizedStatisticsCalculator, RollingStats};
pub use diagnostics::{NetworkDiagnostics, DiagnosticReport, SystemHealth};
pub use output::{OutputFormatter, ColoredFormatter, PlainFormatter, OutputCoordinator, OutputFormatterFactory, VerboseTimingFormatter};
pub use updater::{UpdateCoordinator, UpdateArgs, UpdateResult, UpdateMode, Version, Release, GeographicRegion, VersionManager, CacheManager, CacheStats, FeedsClient, FeedStats, GitHubApiClient, GitHubApiStats, ApiAvailability, RateLimitInfo};

/// Application version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Default configuration values
pub mod defaults {
    use std::time::Duration;

    pub const DEFAULT_TEST_COUNT: u32 = 5;
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
    pub const DEFAULT_TARGET_URLS: &[&str] = &["https://bing.com"];
    pub const DEFAULT_DNS_SERVERS: &[&str] = &[
        "114.114.114.114",  // 114 DNS
        "223.5.5.5",      // Alibaba DNS
        "119.29.29.29", /* Tencent DNSPod */
    ];
    pub const DEFAULT_DOH_PROVIDERS: &[&str] = &[
        "https://cloudflare-dns.com/dns-query", /* Cloudflare */
        "https://dns.google/dns-query",         // Google
    ];
    pub const DEFAULT_ENABLE_COLOR: bool = true;
}