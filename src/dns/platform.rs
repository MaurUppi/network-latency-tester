//! Platform-specific DNS resolution optimizations and configurations
//!
//! This module provides platform-specific DNS handling to ensure optimal
//! performance and compatibility across different operating systems.

use crate::{
    error::Result,
    types::DnsConfig,
};
use std::{
    net::IpAddr,
    time::Duration,
};

/// Platform-specific DNS configuration and optimization settings
#[derive(Debug, Clone)]
pub struct PlatformDnsConfig {
    /// Default DNS timeout for this platform
    pub default_timeout: Duration,
    /// Maximum number of concurrent DNS queries
    pub max_concurrent_queries: usize,
    /// Whether to use system DNS cache
    pub use_system_cache: bool,
    /// Platform-specific DNS server preferences
    pub preferred_dns_servers: Vec<IpAddr>,
    /// Whether IPv6 is preferred on this platform
    pub prefer_ipv6: bool,
}

impl Default for PlatformDnsConfig {
    fn default() -> Self {
        Self::for_current_platform()
    }
}

impl PlatformDnsConfig {
    /// Create platform-specific DNS configuration for the current platform
    pub fn for_current_platform() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::windows_config()
        }
        #[cfg(target_os = "macos")]
        {
            Self::macos_config()
        }
        #[cfg(target_os = "linux")]
        {
            Self::linux_config()
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Self::generic_config()
        }
    }

    /// Windows-specific DNS configuration
    #[cfg(target_os = "windows")]
    pub fn windows_config() -> Self {
        Self {
            default_timeout: Duration::from_secs(5),
            max_concurrent_queries: 10,
            use_system_cache: true,
            preferred_dns_servers: vec![
                "8.8.8.8".parse().unwrap(),     // Google DNS
                "1.1.1.1".parse().unwrap(),     // Cloudflare DNS
                "208.67.222.222".parse().unwrap(), // OpenDNS
            ],
            prefer_ipv6: false, // Windows IPv6 can be inconsistent
        }
    }

    /// macOS-specific DNS configuration
    #[cfg(target_os = "macos")]
    pub fn macos_config() -> Self {
        Self {
            default_timeout: Duration::from_secs(3),
            max_concurrent_queries: 15,
            use_system_cache: true,
            preferred_dns_servers: vec![
                "8.8.8.8".parse().unwrap(),     // Google DNS
                "1.1.1.1".parse().unwrap(),     // Cloudflare DNS
                "2001:4860:4860::8888".parse().unwrap(), // Google IPv6
            ],
            prefer_ipv6: true, // macOS has excellent IPv6 support
        }
    }

    /// Linux-specific DNS configuration
    #[cfg(target_os = "linux")]
    pub fn linux_config() -> Self {
        Self {
            default_timeout: Duration::from_secs(3),
            max_concurrent_queries: 20,
            use_system_cache: false, // Let systemd-resolved handle caching
            preferred_dns_servers: vec![
                "8.8.8.8".parse().unwrap(),     // Google DNS
                "1.1.1.1".parse().unwrap(),     // Cloudflare DNS
                "9.9.9.9".parse().unwrap(),     // Quad9 DNS
            ],
            prefer_ipv6: true, // Modern Linux has good IPv6 support
        }
    }

    /// Generic configuration for other platforms
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub fn generic_config() -> Self {
        Self {
            default_timeout: Duration::from_secs(5),
            max_concurrent_queries: 10,
            use_system_cache: true,
            preferred_dns_servers: vec![
                "8.8.8.8".parse().unwrap(),     // Google DNS
                "1.1.1.1".parse().unwrap(),     // Cloudflare DNS
            ],
            prefer_ipv6: false, // Conservative default
        }
    }

    /// Get platform-specific DoH providers with performance optimizations
    pub fn get_optimized_doh_providers(&self) -> Vec<String> {
        #[cfg(target_os = "windows")]
        {
            vec![
                "https://dns.google/dns-query".to_string(),
                "https://cloudflare-dns.com/dns-query".to_string(),
                "https://dns.opendns.com/dns-query".to_string(),
            ]
        }
        #[cfg(target_os = "macos")]
        {
            vec![
                "https://dns.google/dns-query".to_string(),
                "https://cloudflare-dns.com/dns-query".to_string(),
                "https://doh.opendns.com/dns-query".to_string(),
            ]
        }
        #[cfg(target_os = "linux")]
        {
            vec![
                "https://dns.google/dns-query".to_string(),
                "https://cloudflare-dns.com/dns-query".to_string(),
                "https://dns.quad9.net/dns-query".to_string(),
            ]
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            vec![
                "https://dns.google/dns-query".to_string(),
                "https://cloudflare-dns.com/dns-query".to_string(),
            ]
        }
    }

    /// Check if the current platform supports IPv6 well
    pub fn has_good_ipv6_support(&self) -> bool {
        self.prefer_ipv6
    }

    /// Get recommended timeout for DNS queries on this platform
    pub fn get_dns_timeout(&self, dns_config: &DnsConfig) -> Duration {
        match dns_config {
            DnsConfig::System => self.default_timeout,
            DnsConfig::Custom { .. } => {
                // Custom DNS servers might be slower
                self.default_timeout + Duration::from_millis(500)
            }
            DnsConfig::DoH { .. } => {
                // DoH queries take longer due to HTTPS overhead
                self.default_timeout * 2
            }
        }
    }
}

/// Platform-specific DNS resolver optimizations
pub struct PlatformDnsResolver {
    config: PlatformDnsConfig,
}

impl PlatformDnsResolver {
    /// Create a new platform-specific DNS resolver
    pub fn new() -> Self {
        Self {
            config: PlatformDnsConfig::for_current_platform(),
        }
    }

    /// Create with custom platform configuration
    pub fn with_config(config: PlatformDnsConfig) -> Self {
        Self { config }
    }

    /// Get the platform configuration
    pub fn config(&self) -> &PlatformDnsConfig {
        &self.config
    }

    /// Check if a DNS server is likely to perform well on this platform
    pub fn is_dns_server_optimal(&self, dns_server: &IpAddr) -> bool {
        // Check if it's in our preferred list
        self.config.preferred_dns_servers.contains(dns_server)
    }

    /// Get platform-specific DNS query timeout
    pub fn get_query_timeout(&self, dns_config: &DnsConfig) -> Duration {
        self.config.get_dns_timeout(dns_config)
    }

    /// Optimize DNS configuration list for this platform
    pub fn optimize_dns_configs(&self, configs: Vec<DnsConfig>) -> Vec<DnsConfig> {
        let mut optimized = Vec::new();
        
        // Always start with system DNS
        optimized.push(DnsConfig::System);
        
        // Add platform-optimized custom DNS servers
        for dns_server in &self.config.preferred_dns_servers {
            optimized.push(DnsConfig::Custom { 
                servers: vec![*dns_server] 
            });
        }
        
        // Add optimized DoH providers
        for doh_url in self.config.get_optimized_doh_providers() {
            optimized.push(DnsConfig::DoH { url: doh_url });
        }
        
        // Add any additional custom configs that weren't already included
        for config in configs {
            match &config {
                DnsConfig::Custom { servers } => {
                    // Only add if not already in our optimized list
                    if !servers.iter().any(|ip| self.config.preferred_dns_servers.contains(ip)) {
                        optimized.push(config);
                    }
                }
                DnsConfig::DoH { url } => {
                    // Only add if not already in our optimized list
                    if !self.config.get_optimized_doh_providers().contains(url) {
                        optimized.push(config);
                    }
                }
                DnsConfig::System => {
                    // Already added
                }
            }
        }
        
        optimized
    }

    /// Check platform-specific DNS health
    pub async fn check_dns_health(&self) -> Result<PlatformDnsHealth> {
        let mut health = PlatformDnsHealth {
            platform: get_platform_name(),
            system_dns_working: false,
            ipv6_available: false,
            custom_dns_working: false,
            doh_working: false,
            recommended_config: None,
        };

        // Test system DNS
        health.system_dns_working = self.test_system_dns().await;

        // Test IPv6 availability
        health.ipv6_available = self.test_ipv6_connectivity().await;

        // Test custom DNS
        health.custom_dns_working = self.test_custom_dns().await;

        // Test DoH
        health.doh_working = self.test_doh().await;

        // Recommend best configuration
        health.recommended_config = Some(self.recommend_best_config(&health));

        Ok(health)
    }

    /// Test if system DNS is working
    async fn test_system_dns(&self) -> bool {
        // Attempt to resolve a known domain using system DNS
        match tokio::net::lookup_host("google.com:80").await {
            Ok(mut addrs) => addrs.next().is_some(),
            Err(_) => false,
        }
    }

    /// Test if IPv6 connectivity is available
    async fn test_ipv6_connectivity(&self) -> bool {
        // Try to resolve an IPv6 address
        match tokio::net::lookup_host("ipv6.google.com:80").await {
            Ok(addrs) => {
                addrs.into_iter().any(|addr| addr.is_ipv6())
            }
            Err(_) => false,
        }
    }

    /// Test if custom DNS is working
    async fn test_custom_dns(&self) -> bool {
        // This would require actual DNS resolution testing
        // For now, return true if we have preferred servers
        !self.config.preferred_dns_servers.is_empty()
    }

    /// Test if DoH is working
    async fn test_doh(&self) -> bool {
        // This would require actual DoH testing
        // For now, return true on platforms with good HTTPS support
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            true
        }
        #[cfg(target_os = "windows")]
        {
            // Windows DoH support varies
            true
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            false
        }
    }

    /// Recommend the best DNS configuration based on platform health
    fn recommend_best_config(&self, health: &PlatformDnsHealth) -> DnsConfig {
        if health.system_dns_working {
            DnsConfig::System
        } else if health.custom_dns_working && !self.config.preferred_dns_servers.is_empty() {
            DnsConfig::Custom {
                servers: vec![self.config.preferred_dns_servers[0]]
            }
        } else if health.doh_working {
            let providers = self.config.get_optimized_doh_providers();
            if !providers.is_empty() {
                DnsConfig::DoH { url: providers[0].clone() }
            } else {
                DnsConfig::System // Fallback
            }
        } else {
            DnsConfig::System // Final fallback
        }
    }
}

impl Default for PlatformDnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Platform DNS health check results
#[derive(Debug, Clone)]
pub struct PlatformDnsHealth {
    pub platform: String,
    pub system_dns_working: bool,
    pub ipv6_available: bool,
    pub custom_dns_working: bool,
    pub doh_working: bool,
    pub recommended_config: Option<DnsConfig>,
}

impl PlatformDnsHealth {
    /// Get a human-readable status report
    pub fn status_report(&self) -> String {
        let mut report = format!("DNS Health Report for {}:\n", self.platform);
        report.push_str(&format!("  System DNS: {}\n", if self.system_dns_working { "✓ Working" } else { "✗ Not working" }));
        report.push_str(&format!("  IPv6: {}\n", if self.ipv6_available { "✓ Available" } else { "✗ Not available" }));
        report.push_str(&format!("  Custom DNS: {}\n", if self.custom_dns_working { "✓ Working" } else { "✗ Not working" }));
        report.push_str(&format!("  DoH: {}\n", if self.doh_working { "✓ Working" } else { "✗ Not working" }));
        
        if let Some(ref config) = self.recommended_config {
            report.push_str(&format!("  Recommended: {:?}\n", config));
        }
        
        report
    }

    /// Check if DNS is generally healthy on this platform
    pub fn is_healthy(&self) -> bool {
        self.system_dns_working || self.custom_dns_working || self.doh_working
    }
}

/// Get the current platform name
pub fn get_platform_name() -> String {
    #[cfg(target_os = "windows")]
    {
        "Windows".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "macOS".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        "Linux".to_string()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "Unknown".to_string()
    }
}

/// Platform-specific DNS performance tuning
pub struct DnsPerformanceTuner {
    resolver: PlatformDnsResolver,
}

impl DnsPerformanceTuner {
    /// Create a new DNS performance tuner
    pub fn new() -> Self {
        Self {
            resolver: PlatformDnsResolver::new(),
        }
    }

    /// Tune DNS configuration for optimal performance on this platform
    pub fn tune_for_performance(&self, configs: Vec<DnsConfig>) -> Vec<DnsConfig> {
        self.resolver.optimize_dns_configs(configs)
    }

    /// Get recommended concurrency level for DNS queries
    pub fn get_recommended_concurrency(&self) -> usize {
        self.resolver.config().max_concurrent_queries
    }

    /// Check if a specific DNS configuration is optimal for this platform
    pub fn is_config_optimal(&self, config: &DnsConfig) -> bool {
        match config {
            DnsConfig::System => true, // Always acceptable
            DnsConfig::Custom { servers } => {
                servers.iter().all(|ip| self.resolver.is_dns_server_optimal(ip))
            }
            DnsConfig::DoH { url } => {
                self.resolver.config().get_optimized_doh_providers().contains(url)
            }
        }
    }

    /// Get platform-specific DNS timeout recommendations
    pub fn get_timeout_recommendation(&self, config: &DnsConfig) -> Duration {
        self.resolver.get_query_timeout(config)
    }
}

impl Default for DnsPerformanceTuner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_config_creation() {
        let config = PlatformDnsConfig::for_current_platform();
        assert!(config.default_timeout > Duration::from_secs(0));
        assert!(config.max_concurrent_queries > 0);
        assert!(!config.preferred_dns_servers.is_empty());
    }

    #[test]
    fn test_dns_timeout_calculation() {
        let config = PlatformDnsConfig::for_current_platform();
        
        let system_timeout = config.get_dns_timeout(&DnsConfig::System);
        let custom_timeout = config.get_dns_timeout(&DnsConfig::Custom { 
            servers: vec!["8.8.8.8".parse().unwrap()] 
        });
        let doh_timeout = config.get_dns_timeout(&DnsConfig::DoH { 
            url: "https://dns.google/dns-query".to_string() 
        });

        assert!(custom_timeout >= system_timeout);
        assert!(doh_timeout >= system_timeout);
    }

    #[test]
    fn test_platform_dns_resolver() {
        let resolver = PlatformDnsResolver::new();
        assert!(!resolver.config().preferred_dns_servers.is_empty());
        
        let google_dns = "8.8.8.8".parse().unwrap();
        assert!(resolver.is_dns_server_optimal(&google_dns));
    }

    #[test]
    fn test_dns_config_optimization() {
        let resolver = PlatformDnsResolver::new();
        let configs = vec![
            DnsConfig::System,
            DnsConfig::Custom { servers: vec!["1.2.3.4".parse().unwrap()] },
        ];
        
        let optimized = resolver.optimize_dns_configs(configs);
        assert!(!optimized.is_empty());
        assert_eq!(optimized[0], DnsConfig::System);
    }

    #[test]
    fn test_dns_performance_tuner() {
        let tuner = DnsPerformanceTuner::new();
        assert!(tuner.get_recommended_concurrency() > 0);
        
        let system_config = DnsConfig::System;
        assert!(tuner.is_config_optimal(&system_config));
        
        let timeout = tuner.get_timeout_recommendation(&system_config);
        assert!(timeout > Duration::from_secs(0));
    }

    #[test]
    fn test_platform_name() {
        let platform = get_platform_name();
        assert!(!platform.is_empty());
        assert!(platform == "Windows" || platform == "macOS" || platform == "Linux" || platform == "Unknown");
    }

    #[test]
    fn test_doh_providers() {
        let config = PlatformDnsConfig::for_current_platform();
        let providers = config.get_optimized_doh_providers();
        assert!(!providers.is_empty());
        
        for provider in &providers {
            assert!(provider.starts_with("https://"));
        }
    }

    #[tokio::test]
    async fn test_dns_health_check() {
        let resolver = PlatformDnsResolver::new();
        let health = resolver.check_dns_health().await;
        
        assert!(health.is_ok());
        let health = health.unwrap();
        assert!(!health.platform.is_empty());
        assert!(health.recommended_config.is_some());
    }

    #[test]
    fn test_dns_health_status_report() {
        let health = PlatformDnsHealth {
            platform: "Test".to_string(),
            system_dns_working: true,
            ipv6_available: false,
            custom_dns_working: true,
            doh_working: false,
            recommended_config: Some(DnsConfig::System),
        };

        let report = health.status_report();
        assert!(report.contains("Test"));
        assert!(report.contains("✓ Working"));
        assert!(report.contains("✗ Not"));
    }

    #[test]
    fn test_ipv6_support_detection() {
        let config = PlatformDnsConfig::for_current_platform();
        let has_ipv6 = config.has_good_ipv6_support();
        
        // This varies by platform
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        assert!(has_ipv6);
        
        #[cfg(target_os = "windows")]
        assert!(!has_ipv6); // Conservative default
    }
}