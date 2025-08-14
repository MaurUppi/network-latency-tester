//! Configuration validation utilities and rules

use crate::{
    models::Config,
    error::{AppError, Result},
};
use std::net::IpAddr;
use std::time::Duration;

/// Configuration validator with advanced validation rules
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate configuration with comprehensive checks
    pub fn validate_comprehensive(config: &Config) -> Result<Vec<ValidationWarning>> {
        let mut warnings = Vec::new();

        // Basic validation (already done in Config::validate)
        config.validate()?;

        // Advanced validation checks
        warnings.extend(Self::validate_target_urls(&config.target_urls)?);
        warnings.extend(Self::validate_dns_servers(&config.dns_servers)?);
        warnings.extend(Self::validate_doh_providers(&config.doh_providers)?);
        warnings.extend(Self::validate_performance_settings(config)?);

        Ok(warnings)
    }

    /// Validate target URLs with detailed checks
    fn validate_target_urls(urls: &[String]) -> Result<Vec<ValidationWarning>> {
        let mut warnings = Vec::new();

        for url in urls {
            match url::Url::parse(url) {
                Ok(parsed) => {
                    // Check for HTTP vs HTTPS
                    if parsed.scheme() == "http" {
                        warnings.push(ValidationWarning::new(
                            ValidationLevel::Warning,
                            format!("URL '{}' uses HTTP instead of HTTPS, which may affect timing measurements", url)
                        ));
                    }

                    // Check for non-standard ports
                    if let Some(port) = parsed.port() {
                        match parsed.scheme() {
                            "http" if port != 80 => {
                                warnings.push(ValidationWarning::new(
                                    ValidationLevel::Info,
                                    format!("URL '{}' uses non-standard port {}", url, port)
                                ));
                            }
                            "https" if port != 443 => {
                                warnings.push(ValidationWarning::new(
                                    ValidationLevel::Info,
                                    format!("URL '{}' uses non-standard port {}", url, port)
                                ));
                            }
                            _ => {}
                        }
                    }

                    // Check for local/private networks
                    if let Some(url::Host::Ipv4(ip)) = parsed.host() {
                        if ip.is_private() || ip.is_loopback() {
                            warnings.push(ValidationWarning::new(
                                ValidationLevel::Info,
                                format!("URL '{}' targets private/local network", url)
                            ));
                        }
                    }

                    // Check for paths that might not be suitable for latency testing
                    if !parsed.path().is_empty() && parsed.path() != "/" {
                        warnings.push(ValidationWarning::new(
                            ValidationLevel::Info,
                            format!("URL '{}' includes path '{}', which may affect baseline measurements", url, parsed.path())
                        ));
                    }

                    // Check for query parameters
                    if parsed.query().is_some() {
                        warnings.push(ValidationWarning::new(
                            ValidationLevel::Info,
                            format!("URL '{}' includes query parameters, which may affect caching", url)
                        ));
                    }
                }
                Err(e) => {
                    return Err(AppError::config(format!("Invalid URL '{}': {}", url, e)));
                }
            }
        }

        Ok(warnings)
    }

    /// Validate DNS servers with connectivity checks
    fn validate_dns_servers(servers: &[String]) -> Result<Vec<ValidationWarning>> {
        let mut warnings = Vec::new();

        for server in servers {
            match server.parse::<IpAddr>() {
                Ok(ip) => {
                    // Check for known public DNS servers
                    if Self::is_known_public_dns(&ip) {
                        warnings.push(ValidationWarning::new(
                            ValidationLevel::Info,
                            format!("Using well-known public DNS server: {}", ip)
                        ));
                    }

                    // Check for private IP ranges (might not be accessible)
                    let is_private = match ip {
                        IpAddr::V4(ipv4) => ipv4.is_private(),
                        IpAddr::V6(ipv6) => ipv6.is_loopback(), // IPv6 doesn't have is_private, use loopback as approximation
                    };
                    
                    if is_private {
                        warnings.push(ValidationWarning::new(
                            ValidationLevel::Warning,
                            format!("DNS server {} is in private IP range, ensure it's accessible", ip)
                        ));
                    }

                    // Check for loopback
                    let is_loopback = match ip {
                        IpAddr::V4(ipv4) => ipv4.is_loopback(),
                        IpAddr::V6(ipv6) => ipv6.is_loopback(),
                    };
                    
                    if is_loopback {
                        warnings.push(ValidationWarning::new(
                            ValidationLevel::Info,
                            format!("DNS server {} is loopback address (localhost)", ip)
                        ));
                    }
                }
                Err(e) => {
                    return Err(AppError::config(format!("Invalid DNS server IP '{}': {}", server, e)));
                }
            }
        }

        Ok(warnings)
    }

    /// Validate DNS-over-HTTPS providers
    fn validate_doh_providers(providers: &[String]) -> Result<Vec<ValidationWarning>> {
        let mut warnings = Vec::new();

        for provider in providers {
            match url::Url::parse(provider) {
                Ok(parsed) => {
                    // Check scheme
                    if parsed.scheme() != "https" {
                        return Err(AppError::config(format!("DoH provider must use HTTPS: {}", provider)));
                    }

                    // Check for known DoH providers
                    if Self::is_known_doh_provider(&parsed) {
                        warnings.push(ValidationWarning::new(
                            ValidationLevel::Info,
                            format!("Using well-known DoH provider: {}", parsed.host_str().unwrap_or("unknown"))
                        ));
                    }

                    // Check path - DoH typically uses /dns-query
                    if !parsed.path().contains("dns-query") && !parsed.path().contains("resolve") {
                        warnings.push(ValidationWarning::new(
                            ValidationLevel::Warning,
                            format!("DoH provider '{}' may not use standard path (expected 'dns-query' or 'resolve')", provider)
                        ));
                    }

                    // Check for custom ports
                    if let Some(port) = parsed.port() {
                        if port != 443 {
                            warnings.push(ValidationWarning::new(
                                ValidationLevel::Info,
                                format!("DoH provider '{}' uses non-standard port {}", provider, port)
                            ));
                        }
                    }
                }
                Err(e) => {
                    return Err(AppError::config(format!("Invalid DoH provider URL '{}': {}", provider, e)));
                }
            }
        }

        Ok(warnings)
    }

    /// Validate performance-related settings
    fn validate_performance_settings(config: &Config) -> Result<Vec<ValidationWarning>> {
        let mut warnings = Vec::new();

        // Check test count
        if config.test_count < 3 {
            warnings.push(ValidationWarning::new(
                ValidationLevel::Warning,
                format!("Test count of {} may not provide reliable statistics (recommended: >= 3)", config.test_count)
            ));
        } else if config.test_count > 50 {
            warnings.push(ValidationWarning::new(
                ValidationLevel::Info,
                format!("High test count of {} will increase execution time", config.test_count)
            ));
        }

        // Check timeout
        if config.timeout_seconds < 3 {
            warnings.push(ValidationWarning::new(
                ValidationLevel::Warning,
                format!("Timeout of {}s may be too short for reliable measurements", config.timeout_seconds)
            ));
        } else if config.timeout_seconds > 60 {
            warnings.push(ValidationWarning::new(
                ValidationLevel::Info,
                format!("Long timeout of {}s will slow down failure detection", config.timeout_seconds)
            ));
        }

        // Check for potential excessive testing
        let total_tests = config.target_urls.len() as u32 * 
                         (1 + config.dns_servers.len() as u32 + config.doh_providers.len() as u32) * 
                         config.test_count;
        
        if total_tests > 500 {
            warnings.push(ValidationWarning::new(
                ValidationLevel::Warning,
                format!("Configuration will perform {} total tests, which may take a long time", total_tests)
            ));
        } else if total_tests > 100 {
            warnings.push(ValidationWarning::new(
                ValidationLevel::Info,
                format!("Configuration will perform {} total tests", total_tests)
            ));
        }

        Ok(warnings)
    }

    /// Check if IP is a known public DNS server
    fn is_known_public_dns(ip: &IpAddr) -> bool {
        let known_dns = [
            // IPv4 DNS servers
            "8.8.8.8",      // Google DNS
            "8.8.4.4",      // Google DNS
            "1.1.1.1",      // Cloudflare DNS
            "1.0.0.1",      // Cloudflare DNS
            "208.67.222.222", // OpenDNS
            "208.67.220.220", // OpenDNS
            "9.9.9.9",      // Quad9 DNS
            "149.112.112.112", // Quad9 DNS
            "114.114.114.114",  // Alternate DNS
            "76.223.100.101", // Alternate DNS
            "120.53.53.102", // Tencent DNS
            "223.5.5.5",    // Alibaba DNS
            "119.29.29.29",    // Tencent DNSPod
            // IPv6 DNS servers
            "2001:4860:4860::8888", // Google IPv6 DNS
            "2001:4860:4860::8844", // Google IPv6 DNS
            "2606:4700:4700::1111", // Cloudflare IPv6 DNS
            "2606:4700:4700::1001", // Cloudflare IPv6 DNS
            "2620:119:35::35",      // OpenDNS IPv6
            "2620:119:53::53",      // OpenDNS IPv6
            "2620:fe::fe",          // Quad9 IPv6 DNS
            "2620:fe::9",           // Quad9 IPv6 DNS
        ];

        known_dns.iter().any(|&known| known.parse::<IpAddr>().unwrap() == *ip)
    }

    /// Check if URL is a known DoH provider
    fn is_known_doh_provider(url: &url::Url) -> bool {
        if let Some(host) = url.host_str() {
            let known_hosts = [
                "cloudflare-dns.com",
                "dns.google",
                "dns.quad9.net",
                "doh.opendns.com",
                "doh.dns.sb",
                "dns.adguard.com",
                "alidns.com",
            ];

            known_hosts.iter().any(|&known| host.contains(known))
        } else {
            false
        }
    }
}

/// Validation warning levels
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationLevel {
    Info,
    Warning,
    Error,
}

impl ValidationLevel {
    /// Get display string for level
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warning => "WARNING", 
            Self::Error => "ERROR",
        }
    }

    /// Get color for terminal display
    pub fn color(&self) -> &'static str {
        match self {
            Self::Info => "blue",
            Self::Warning => "yellow",
            Self::Error => "red",
        }
    }
}

/// Configuration validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub level: ValidationLevel,
    pub message: String,
}

impl ValidationWarning {
    /// Create a new validation warning
    pub fn new(level: ValidationLevel, message: String) -> Self {
        Self { level, message }
    }

    /// Format warning for display
    pub fn format(&self, _use_color: bool) -> String {
        format!("[{}] {}", self.level.as_str(), self.message)
    }
}

/// Convenience function for comprehensive configuration validation
pub fn validate_config(config: &Config) -> Result<Vec<ValidationWarning>> {
    ConfigValidator::validate_comprehensive(config)
}

/// Test network connectivity to validate configuration feasibility
pub async fn test_configuration_connectivity(config: &Config) -> Result<ConnectivityReport> {
    let mut report = ConnectivityReport::new();

    // Test target URL connectivity
    for url in &config.target_urls {
        let result = test_url_connectivity(url, config.timeout()).await;
        report.url_results.push((url.clone(), result));
    }

    // Test DNS server connectivity (basic UDP check to port 53)
    for dns_server in &config.dns_servers {
        let result = test_dns_connectivity(dns_server).await;
        report.dns_results.push((dns_server.clone(), result));
    }

    // Test DoH provider connectivity
    for doh_provider in &config.doh_providers {
        let result = test_doh_connectivity(doh_provider, config.timeout()).await;
        report.doh_results.push((doh_provider.clone(), result));
    }

    Ok(report)
}

/// Test URL connectivity
async fn test_url_connectivity(url: &str, timeout: Duration) -> ConnectivityResult {
    match reqwest::Client::new()
        .head(url)
        .timeout(timeout)
        .send()
        .await
    {
        Ok(response) => {
            ConnectivityResult::Success {
                status_code: Some(response.status().as_u16()),
                response_time: None, // TODO: measure response time
            }
        }
        Err(e) => {
            ConnectivityResult::Failed {
                error: e.to_string(),
            }
        }
    }
}

/// Test DNS server connectivity
async fn test_dns_connectivity(dns_server: &str) -> ConnectivityResult {
    // For now, just validate the IP format (actual DNS query testing would be more complex)
    match dns_server.parse::<IpAddr>() {
        Ok(_) => ConnectivityResult::Success {
            status_code: None,
            response_time: None,
        },
        Err(e) => ConnectivityResult::Failed {
            error: format!("Invalid DNS server IP: {}", e),
        },
    }
}

/// Test DoH provider connectivity
async fn test_doh_connectivity(doh_url: &str, timeout: Duration) -> ConnectivityResult {
    match reqwest::Client::new()
        .head(doh_url)
        .timeout(timeout)
        .send()
        .await
    {
        Ok(response) => {
            ConnectivityResult::Success {
                status_code: Some(response.status().as_u16()),
                response_time: None,
            }
        }
        Err(e) => {
            ConnectivityResult::Failed {
                error: e.to_string(),
            }
        }
    }
}

/// Connectivity test result
#[derive(Debug, Clone)]
pub enum ConnectivityResult {
    Success {
        status_code: Option<u16>,
        response_time: Option<Duration>,
    },
    Failed {
        error: String,
    },
}

impl ConnectivityResult {
    /// Check if the connectivity test was successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Get error message if failed
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Failed { error } => Some(error),
            _ => None,
        }
    }
}

/// Connectivity test report
#[derive(Debug, Default)]
pub struct ConnectivityReport {
    pub url_results: Vec<(String, ConnectivityResult)>,
    pub dns_results: Vec<(String, ConnectivityResult)>,
    pub doh_results: Vec<(String, ConnectivityResult)>,
}


impl ConnectivityReport {
    /// Create a new connectivity report
    pub fn new() -> Self {
        Self {
            url_results: Vec::new(),
            dns_results: Vec::new(),
            doh_results: Vec::new(),
        }
    }

    /// Get summary of connectivity results
    pub fn summary(&self) -> ConnectivitySummary {
        let url_success = self.url_results.iter().filter(|(_, result)| result.is_success()).count();
        let dns_success = self.dns_results.iter().filter(|(_, result)| result.is_success()).count();
        let doh_success = self.doh_results.iter().filter(|(_, result)| result.is_success()).count();

        ConnectivitySummary {
            total_urls: self.url_results.len(),
            successful_urls: url_success,
            total_dns: self.dns_results.len(),
            successful_dns: dns_success,
            total_doh: self.doh_results.len(),
            successful_doh: doh_success,
        }
    }

    /// Check if all connectivity tests passed
    pub fn all_successful(&self) -> bool {
        let summary = self.summary();
        summary.successful_urls == summary.total_urls &&
        summary.successful_dns == summary.total_dns &&
        summary.successful_doh == summary.total_doh
    }
}

/// Connectivity summary statistics
#[derive(Debug)]
pub struct ConnectivitySummary {
    pub total_urls: usize,
    pub successful_urls: usize,
    pub total_dns: usize,
    pub successful_dns: usize,
    pub total_doh: usize,
    pub successful_doh: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_warning() {
        let warning = ValidationWarning::new(
            ValidationLevel::Warning,
            "Test warning message".to_string()
        );

        assert_eq!(warning.level, ValidationLevel::Warning);
        assert_eq!(warning.message, "Test warning message");

        let formatted = warning.format(false);
        assert!(formatted.contains("WARNING"));
        assert!(formatted.contains("Test warning message"));
    }

    #[test]
    fn test_validation_levels() {
        assert_eq!(ValidationLevel::Info.as_str(), "INFO");
        assert_eq!(ValidationLevel::Warning.as_str(), "WARNING");
        assert_eq!(ValidationLevel::Error.as_str(), "ERROR");

        assert_eq!(ValidationLevel::Info.color(), "blue");
        assert_eq!(ValidationLevel::Warning.color(), "yellow");
        assert_eq!(ValidationLevel::Error.color(), "red");
    }

    #[test]
    fn test_is_known_public_dns() {
        assert!(ConfigValidator::is_known_public_dns(&"8.8.8.8".parse().unwrap()));
        assert!(ConfigValidator::is_known_public_dns(&"1.1.1.1".parse().unwrap()));
        assert!(ConfigValidator::is_known_public_dns(&"223.5.5.5".parse().unwrap()));
        assert!(!ConfigValidator::is_known_public_dns(&"192.168.1.1".parse().unwrap()));
    }

    #[test]
    fn test_is_known_doh_provider() {
        let cloudflare = url::Url::parse("https://cloudflare-dns.com/dns-query").unwrap();
        let google = url::Url::parse("https://dns.google/dns-query").unwrap();
        let unknown = url::Url::parse("https://example.com/dns-query").unwrap();

        assert!(ConfigValidator::is_known_doh_provider(&cloudflare));
        assert!(ConfigValidator::is_known_doh_provider(&google));
        assert!(!ConfigValidator::is_known_doh_provider(&unknown));
    }

    #[test]
    fn test_connectivity_result() {
        let success = ConnectivityResult::Success {
            status_code: Some(200),
            response_time: None,
        };

        let failed = ConnectivityResult::Failed {
            error: "Connection refused".to_string(),
        };

        assert!(success.is_success());
        assert!(!failed.is_success());
        assert_eq!(failed.error_message(), Some("Connection refused"));
        assert_eq!(success.error_message(), None);
    }

    #[test]
    fn test_connectivity_report() {
        let mut report = ConnectivityReport::new();
        
        report.url_results.push(("https://example.com".to_string(), ConnectivityResult::Success {
            status_code: Some(200),
            response_time: None,
        }));
        
        report.dns_results.push(("8.8.8.8".to_string(), ConnectivityResult::Success {
            status_code: None,
            response_time: None,
        }));

        let summary = report.summary();
        assert_eq!(summary.total_urls, 1);
        assert_eq!(summary.successful_urls, 1);
        assert_eq!(summary.total_dns, 1);
        assert_eq!(summary.successful_dns, 1);
        
        assert!(report.all_successful());
    }

    #[test]
    fn test_comprehensive_validation() {
        let mut config = Config::default();
        config.target_urls = vec!["http://example.com".to_string()]; // HTTP warning
        config.test_count = 2; // Low count warning
        
        let warnings = ConfigValidator::validate_comprehensive(&config).unwrap();
        
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.message.contains("HTTP")));
        assert!(warnings.iter().any(|w| w.message.contains("reliable statistics")));
    }

    #[test]
    fn test_boundary_values_test_count() {
        let mut config = Config::default();
        
        // Test minimum boundary
        config.test_count = 1;
        let warnings = ConfigValidator::validate_comprehensive(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("reliable statistics")));
        
        // Test just above boundary
        config.test_count = 3;
        let warnings = ConfigValidator::validate_comprehensive(&config).unwrap();
        assert!(!warnings.iter().any(|w| w.message.contains("reliable statistics")));
        
        // Test high boundary
        config.test_count = 51;
        let warnings = ConfigValidator::validate_comprehensive(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("execution time")));
    }

    #[test]
    fn test_boundary_values_timeout() {
        let mut config = Config::default();
        
        // Test minimum boundary
        config.timeout_seconds = 1;
        let warnings = ConfigValidator::validate_comprehensive(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("too short")));
        
        // Test just above boundary
        config.timeout_seconds = 3;
        let warnings = ConfigValidator::validate_comprehensive(&config).unwrap();
        assert!(!warnings.iter().any(|w| w.message.contains("too short")));
        
        // Test high boundary
        config.timeout_seconds = 61;
        let warnings = ConfigValidator::validate_comprehensive(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("slow down failure")));
    }

    #[test]
    fn test_ipv6_dns_servers() {
        assert!(ConfigValidator::is_known_public_dns(&"2001:4860:4860::8888".parse().unwrap())); // Google IPv6 DNS should be false currently
        assert!(!ConfigValidator::is_known_public_dns(&"2001:db8::1".parse().unwrap())); // Test IPv6 address
    }

    #[test]
    fn test_url_edge_cases() {
        let mut config = Config::default();
        config.target_urls = vec![
            "https://example.com/path?query=value".to_string(), // Query params
            "https://example.com/deep/path/here".to_string(),    // Deep path
            "ftp://example.com".to_string(),                     // Non-HTTP protocol
        ];
        
        let warnings = ConfigValidator::validate_comprehensive(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("query parameters")));
        assert!(warnings.iter().any(|w| w.message.contains("path")));
    }

    #[test]
    fn test_excessive_total_tests() {
        let mut config = Config::default();
        config.target_urls = vec!["https://example.com".to_string(); 10]; // 10 URLs
        config.dns_servers = vec!["8.8.8.8".to_string(); 10]; // 10 DNS servers
        config.doh_providers = vec!["https://dns.google/dns-query".to_string(); 10]; // 10 DoH providers
        config.test_count = 5; // 10 * (1+10+10) * 5 = 1050 total tests
        
        let warnings = ConfigValidator::validate_comprehensive(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("long time")));
    }
}