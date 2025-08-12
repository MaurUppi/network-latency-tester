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

// Re-export commonly used types
pub use error::{AppError, Result};
pub use models::{Config, TimingMetrics, TestResult, Statistics};
pub use stats::{StatisticsEngine, StatisticalAnalysis, ExtendedStatistics, OptimizedStatisticsCalculator, RollingStats};
pub use diagnostics::{NetworkDiagnostics, DiagnosticReport, SystemHealth};
pub use output::{OutputFormatter, ColoredFormatter, PlainFormatter, OutputCoordinator, OutputFormatterFactory, VerboseTimingFormatter};

/// Application version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Default configuration values
pub mod defaults {
    use std::time::Duration;

    pub const DEFAULT_TEST_COUNT: u32 = 5;
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
    pub const DEFAULT_TARGET_URLS: &[&str] = &["https://as.ctok.ai"];
    pub const DEFAULT_DNS_SERVERS: &[&str] = &[
        "120.53.53.102",  // Tencent DNS
        "223.5.5.5",      // Alibaba DNS
        "223.6.6.6",      // Alibaba DNS Secondary
    ];
    pub const DEFAULT_DOH_PROVIDERS: &[&str] = &[
        "https://137618-io7m09tk35h1lurw.alidns.com/dns-query",  // Aliyun DoH
        "https://hk1.pro.xns.one/6EMqIkLe5E4/dns-query",         // NovaXNS
    ];
    pub const DEFAULT_ENABLE_COLOR: bool = true;
}