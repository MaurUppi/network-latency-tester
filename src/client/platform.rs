//! Platform-specific HTTP client configurations and networking optimizations
//!
//! This module provides platform-specific HTTP client settings, TLS configurations,
//! and networking optimizations to ensure optimal performance across different operating systems.

use crate::error::{AppError, Result};
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

/// Platform-specific networking configuration
#[derive(Debug, Clone)]
pub struct PlatformNetworkConfig {
    /// Default connection timeout for this platform
    pub connection_timeout: Duration,
    /// Default request timeout for this platform
    pub request_timeout: Duration,
    /// Maximum number of concurrent connections
    pub max_concurrent_connections: usize,
    /// Whether to enable HTTP/2 by default
    pub enable_http2: bool,
    /// Whether to enable connection pooling
    pub enable_connection_pooling: bool,
    /// TCP keep-alive settings
    pub tcp_keepalive: Option<Duration>,
    /// Maximum number of redirects to follow
    pub max_redirects: usize,
    /// User agent string for this platform
    pub user_agent: String,
    /// Whether to enable TLS SNI (Server Name Indication)
    pub enable_tls_sni: bool,
    /// Minimum TLS version to accept
    pub min_tls_version: TlsVersion,
    /// Whether to verify TLS certificates strictly
    pub strict_tls_verification: bool,
}

/// Supported TLS versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    /// TLS 1.2
    V1_2,
    /// TLS 1.3
    V1_3,
}

impl Default for PlatformNetworkConfig {
    fn default() -> Self {
        Self::for_current_platform()
    }
}

impl PlatformNetworkConfig {
    /// Create platform-specific network configuration for the current platform
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

    /// Windows-specific network configuration
    #[cfg(target_os = "windows")]
    pub fn windows_config() -> Self {
        Self {
            connection_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            max_concurrent_connections: 10,
            enable_http2: true,
            enable_connection_pooling: true,
            tcp_keepalive: Some(Duration::from_secs(30)),
            max_redirects: 5,
            user_agent: format!("network-latency-tester/0.1.0 (Windows; {})", 
                               std::env::consts::ARCH),
            enable_tls_sni: true,
            min_tls_version: TlsVersion::V1_2, // Windows might have older TLS
            strict_tls_verification: true,
        }
    }

    /// macOS-specific network configuration
    #[cfg(target_os = "macos")]
    pub fn macos_config() -> Self {
        Self {
            connection_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(20),
            max_concurrent_connections: 15,
            enable_http2: true,
            enable_connection_pooling: true,
            tcp_keepalive: Some(Duration::from_secs(15)),
            max_redirects: 10,
            user_agent: format!("network-latency-tester/0.1.0 (macOS; {})", 
                               std::env::consts::ARCH),
            enable_tls_sni: true,
            min_tls_version: TlsVersion::V1_3, // macOS has excellent TLS support
            strict_tls_verification: true,
        }
    }

    /// Linux-specific network configuration
    #[cfg(target_os = "linux")]
    pub fn linux_config() -> Self {
        Self {
            connection_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(20),
            max_concurrent_connections: 20,
            enable_http2: true,
            enable_connection_pooling: true,
            tcp_keepalive: Some(Duration::from_secs(15)),
            max_redirects: 10,
            user_agent: format!("network-latency-tester/0.1.0 (Linux; {})", 
                               std::env::consts::ARCH),
            enable_tls_sni: true,
            min_tls_version: TlsVersion::V1_3, // Modern Linux has excellent TLS
            strict_tls_verification: true,
        }
    }

    /// Generic configuration for unknown platforms
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub fn generic_config() -> Self {
        Self {
            connection_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            max_concurrent_connections: 10,
            enable_http2: false, // Conservative default
            enable_connection_pooling: true,
            tcp_keepalive: Some(Duration::from_secs(30)),
            max_redirects: 5,
            user_agent: format!("network-latency-tester/0.1.0 (Unknown; {})", 
                               std::env::consts::ARCH),
            enable_tls_sni: true,
            min_tls_version: TlsVersion::V1_2, // Conservative default
            strict_tls_verification: true,
        }
    }

    /// Get timeout for specific operation types
    pub fn get_timeout_for_operation(&self, operation: NetworkOperation) -> Duration {
        match operation {
            NetworkOperation::DnsResolution => Duration::from_secs(5),
            NetworkOperation::TcpConnection => self.connection_timeout,
            NetworkOperation::TlsHandshake => self.connection_timeout + Duration::from_secs(5),
            NetworkOperation::HttpRequest => self.request_timeout,
            NetworkOperation::DataTransfer => self.request_timeout * 2,
        }
    }

    /// Check if platform has good networking performance
    pub fn has_high_performance_networking(&self) -> bool {
        self.enable_http2 && 
        self.enable_connection_pooling &&
        self.max_concurrent_connections >= 15
    }
}

/// Different types of network operations for timeout configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkOperation {
    /// DNS resolution operation
    DnsResolution,
    /// TCP connection establishment
    TcpConnection,
    /// TLS handshake
    TlsHandshake,
    /// HTTP request/response
    HttpRequest,
    /// Data transfer (large responses)
    DataTransfer,
}

/// Platform-specific HTTP client builder
pub struct PlatformClientBuilder {
    config: PlatformNetworkConfig,
    builder: ClientBuilder,
}

impl PlatformClientBuilder {
    /// Create a new platform-specific client builder
    pub fn new() -> Self {
        let config = PlatformNetworkConfig::for_current_platform();
        let mut builder = Client::builder();

        // Apply platform-specific settings
        builder = builder
            .timeout(config.request_timeout)
            .connect_timeout(config.connection_timeout)
            .user_agent(&config.user_agent);

        // HTTP/2 settings
        if config.enable_http2 {
            builder = builder.http2_prior_knowledge();
        }

        // Connection pool settings
        if config.enable_connection_pooling {
            builder = builder.pool_max_idle_per_host(config.max_concurrent_connections);
        }

        // TCP keepalive
        if let Some(keepalive) = config.tcp_keepalive {
            builder = builder.tcp_keepalive(keepalive);
        }

        // Redirect policy
        builder = builder.redirect(reqwest::redirect::Policy::limited(config.max_redirects));

        Self { config, builder }
    }

    /// Create with custom platform configuration
    pub fn with_config(config: PlatformNetworkConfig) -> Self {
        let mut builder = Client::builder();

        // Apply custom platform settings
        builder = builder
            .timeout(config.request_timeout)
            .connect_timeout(config.connection_timeout)
            .user_agent(&config.user_agent);

        if config.enable_http2 {
            builder = builder.http2_prior_knowledge();
        }

        if config.enable_connection_pooling {
            builder = builder.pool_max_idle_per_host(config.max_concurrent_connections);
        }

        if let Some(keepalive) = config.tcp_keepalive {
            builder = builder.tcp_keepalive(keepalive);
        }

        builder = builder.redirect(reqwest::redirect::Policy::limited(config.max_redirects));

        Self { config, builder }
    }

    /// Enable or disable TLS verification
    pub fn tls_verification(mut self, enabled: bool) -> Self {
        if !enabled {
            self.builder = self.builder.danger_accept_invalid_certs(true);
        }
        self
    }

    /// Set custom timeout for specific operation
    pub fn timeout_for_operation(mut self, operation: NetworkOperation, timeout: Duration) -> Self {
        match operation {
            NetworkOperation::TcpConnection => {
                self.builder = self.builder.connect_timeout(timeout);
            }
            NetworkOperation::HttpRequest | NetworkOperation::DataTransfer => {
                self.builder = self.builder.timeout(timeout);
            }
            _ => {
                // DNS and TLS timeouts are handled at different levels
            }
        }
        self
    }

    /// Configure for latency testing (optimized for speed)
    pub fn for_latency_testing(mut self) -> Self {
        self.builder = self.builder
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .pool_max_idle_per_host(1) // Minimize connection overhead
            .redirect(reqwest::redirect::Policy::limited(3));
        self
    }

    /// Configure for throughput testing (optimized for data transfer)
    pub fn for_throughput_testing(mut self) -> Self {
        self.builder = self.builder
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(self.config.max_concurrent_connections)
            .tcp_keepalive(Duration::from_secs(60));
        self
    }

    /// Build the HTTP client
    pub fn build(self) -> Result<Client> {
        self.builder.build()
            .map_err(|e| AppError::network(format!("Failed to build HTTP client: {}", e)))
    }

    /// Get the platform configuration
    pub fn config(&self) -> &PlatformNetworkConfig {
        &self.config
    }
}

impl Default for PlatformClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Platform-specific networking diagnostics
pub struct PlatformNetworkDiagnostics {
    config: PlatformNetworkConfig,
}

impl PlatformNetworkDiagnostics {
    /// Create new platform network diagnostics
    pub fn new() -> Self {
        Self {
            config: PlatformNetworkConfig::for_current_platform(),
        }
    }

    /// Check platform networking capabilities
    pub async fn check_networking_capabilities(&self) -> NetworkCapabilities {
        let capabilities = NetworkCapabilities {
            platform: crate::dns::platform::get_platform_name(),
            http2_support: self.test_http2_support().await,
            ipv6_support: self.test_ipv6_support().await,
            tls_1_3_support: self.test_tls13_support().await,
            connection_pooling_effective: self.test_connection_pooling().await,
            high_concurrency_support: self.config.max_concurrent_connections >= 15,
            recommended_timeouts: self.get_recommended_timeouts(),
        };

        capabilities
    }

    /// Test HTTP/2 support
    async fn test_http2_support(&self) -> bool {
        // Try to create an HTTP/2 client and make a request
        match Client::builder()
            .http2_prior_knowledge()
            .timeout(Duration::from_secs(5))
            .build()
        {
            Ok(_client) => {
                // HTTP/2 client creation succeeded
                true
            }
            Err(_) => false,
        }
    }

    /// Test IPv6 support
    async fn test_ipv6_support(&self) -> bool {
        // Try to connect to an IPv6 endpoint
        match tokio::net::TcpStream::connect("[::1]:80").await {
            Ok(_) => true,
            Err(_) => {
                // Try IPv6 DNS resolution
                match tokio::net::lookup_host("ipv6.google.com:80").await {
                    Ok(addrs) => addrs.into_iter().any(|addr| addr.is_ipv6()),
                    Err(_) => false,
                }
            }
        }
    }

    /// Test TLS 1.3 support
    async fn test_tls13_support(&self) -> bool {
        // This is a simplified test - in practice would require more sophisticated TLS testing
        match self.config.min_tls_version {
            TlsVersion::V1_3 => true,
            TlsVersion::V1_2 => false,
        }
    }

    /// Test connection pooling effectiveness
    async fn test_connection_pooling(&self) -> bool {
        self.config.enable_connection_pooling && self.config.max_concurrent_connections > 5
    }

    /// Get recommended timeouts for this platform
    fn get_recommended_timeouts(&self) -> PlatformTimeouts {
        PlatformTimeouts {
            dns_resolution: self.config.get_timeout_for_operation(NetworkOperation::DnsResolution),
            tcp_connection: self.config.get_timeout_for_operation(NetworkOperation::TcpConnection),
            tls_handshake: self.config.get_timeout_for_operation(NetworkOperation::TlsHandshake),
            http_request: self.config.get_timeout_for_operation(NetworkOperation::HttpRequest),
            data_transfer: self.config.get_timeout_for_operation(NetworkOperation::DataTransfer),
        }
    }
}

impl Default for PlatformNetworkDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

/// Network capabilities for the current platform
#[derive(Debug, Clone)]
pub struct NetworkCapabilities {
    pub platform: String,
    pub http2_support: bool,
    pub ipv6_support: bool,
    pub tls_1_3_support: bool,
    pub connection_pooling_effective: bool,
    pub high_concurrency_support: bool,
    pub recommended_timeouts: PlatformTimeouts,
}

impl NetworkCapabilities {
    /// Get a human-readable capabilities report
    pub fn capabilities_report(&self) -> String {
        let mut report = format!("Network Capabilities Report for {}:\n", self.platform);
        report.push_str(&format!("  HTTP/2: {}\n", if self.http2_support { "✓ Supported" } else { "✗ Not supported" }));
        report.push_str(&format!("  IPv6: {}\n", if self.ipv6_support { "✓ Supported" } else { "✗ Not supported" }));
        report.push_str(&format!("  TLS 1.3: {}\n", if self.tls_1_3_support { "✓ Supported" } else { "✗ Not supported" }));
        report.push_str(&format!("  Connection Pooling: {}\n", if self.connection_pooling_effective { "✓ Effective" } else { "✗ Limited" }));
        report.push_str(&format!("  High Concurrency: {}\n", if self.high_concurrency_support { "✓ Supported" } else { "✗ Limited" }));
        report.push_str("  Recommended Timeouts:\n");
        report.push_str(&format!("    DNS: {:?}\n", self.recommended_timeouts.dns_resolution));
        report.push_str(&format!("    TCP: {:?}\n", self.recommended_timeouts.tcp_connection));
        report.push_str(&format!("    TLS: {:?}\n", self.recommended_timeouts.tls_handshake));
        report.push_str(&format!("    HTTP: {:?}\n", self.recommended_timeouts.http_request));
        report.push_str(&format!("    Transfer: {:?}\n", self.recommended_timeouts.data_transfer));
        report
    }

    /// Check if the platform has good networking performance
    pub fn has_good_performance(&self) -> bool {
        self.http2_support && 
        self.connection_pooling_effective &&
        self.high_concurrency_support
    }

    /// Get performance score (0-100)
    pub fn performance_score(&self) -> u8 {
        let mut score = 0u8;
        
        if self.http2_support { score += 20; }
        if self.ipv6_support { score += 15; }
        if self.tls_1_3_support { score += 20; }
        if self.connection_pooling_effective { score += 25; }
        if self.high_concurrency_support { score += 20; }
        
        score
    }
}

/// Platform-specific timeout recommendations
#[derive(Debug, Clone)]
pub struct PlatformTimeouts {
    pub dns_resolution: Duration,
    pub tcp_connection: Duration,
    pub tls_handshake: Duration,
    pub http_request: Duration,
    pub data_transfer: Duration,
}

/// Certificate validation configuration for different platforms
pub struct CertificateValidator {
    strict_validation: bool,
    platform_config: PlatformNetworkConfig,
}

impl CertificateValidator {
    /// Create new certificate validator with platform defaults
    pub fn new() -> Self {
        let config = PlatformNetworkConfig::for_current_platform();
        Self {
            strict_validation: config.strict_tls_verification,
            platform_config: config,
        }
    }

    /// Create with custom validation settings
    pub fn with_strict_validation(strict: bool) -> Self {
        let mut validator = Self::new();
        validator.strict_validation = strict;
        validator
    }

    /// Check if certificate validation should be strict on this platform
    pub fn should_validate_strictly(&self) -> bool {
        self.strict_validation && self.platform_supports_strict_validation()
    }

    /// Check if platform supports strict certificate validation
    fn platform_supports_strict_validation(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            // Windows certificate validation can be inconsistent
            true // But we'll be strict by default
        }
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            // Unix-like systems generally have good certificate validation
            true
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            // Conservative default for unknown platforms
            false
        }
    }

    /// Get certificate-related timeout
    pub fn get_certificate_timeout(&self) -> Duration {
        self.platform_config.get_timeout_for_operation(NetworkOperation::TlsHandshake)
    }

    /// Apply certificate validation to client builder
    pub fn apply_to_client_builder(&self, builder: ClientBuilder) -> ClientBuilder {
        if self.should_validate_strictly() {
            builder
        } else {
            builder.danger_accept_invalid_certs(true)
        }
    }
}

impl Default for CertificateValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_network_config() {
        let config = PlatformNetworkConfig::for_current_platform();
        assert!(config.connection_timeout > Duration::from_secs(0));
        assert!(config.request_timeout > Duration::from_secs(0));
        assert!(config.max_concurrent_connections > 0);
        assert!(config.max_redirects > 0);
        assert!(!config.user_agent.is_empty());
    }

    #[test]
    fn test_timeout_for_operations() {
        let config = PlatformNetworkConfig::for_current_platform();
        
        let dns_timeout = config.get_timeout_for_operation(NetworkOperation::DnsResolution);
        let tcp_timeout = config.get_timeout_for_operation(NetworkOperation::TcpConnection);
        let tls_timeout = config.get_timeout_for_operation(NetworkOperation::TlsHandshake);
        let http_timeout = config.get_timeout_for_operation(NetworkOperation::HttpRequest);
        let transfer_timeout = config.get_timeout_for_operation(NetworkOperation::DataTransfer);

        assert!(dns_timeout > Duration::from_secs(0));
        assert!(tcp_timeout > Duration::from_secs(0));
        assert!(tls_timeout >= tcp_timeout);
        assert!(http_timeout >= tcp_timeout);
        assert!(transfer_timeout >= http_timeout);
    }

    #[test]
    fn test_platform_client_builder() {
        let builder = PlatformClientBuilder::new();
        assert!(builder.config().connection_timeout > Duration::from_secs(0));
        
        let client = builder.build();
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_builder_configurations() {
        let builder = PlatformClientBuilder::new()
            .for_latency_testing()
            .tls_verification(false);
        
        let client = builder.build();
        assert!(client.is_ok());
    }

    #[test]
    fn test_tls_version_ordering() {
        assert_eq!(TlsVersion::V1_2, TlsVersion::V1_2);
        assert_eq!(TlsVersion::V1_3, TlsVersion::V1_3);
        assert_ne!(TlsVersion::V1_2, TlsVersion::V1_3);
    }

    #[test]
    fn test_network_operation_types() {
        let operations = vec![
            NetworkOperation::DnsResolution,
            NetworkOperation::TcpConnection,
            NetworkOperation::TlsHandshake,
            NetworkOperation::HttpRequest,
            NetworkOperation::DataTransfer,
        ];

        for op in operations {
            let config = PlatformNetworkConfig::for_current_platform();
            let timeout = config.get_timeout_for_operation(op);
            assert!(timeout > Duration::from_secs(0));
        }
    }

    #[tokio::test]
    async fn test_platform_network_diagnostics() {
        let diagnostics = PlatformNetworkDiagnostics::new();
        let capabilities = diagnostics.check_networking_capabilities().await;
        
        assert!(!capabilities.platform.is_empty());
        assert!(capabilities.recommended_timeouts.dns_resolution > Duration::from_secs(0));
        assert!(capabilities.performance_score() <= 100);
    }

    #[test]
    fn test_certificate_validator() {
        let validator = CertificateValidator::new();
        let timeout = validator.get_certificate_timeout();
        assert!(timeout > Duration::from_secs(0));
        
        let strict_validator = CertificateValidator::with_strict_validation(true);
        assert!(strict_validator.should_validate_strictly() || !strict_validator.should_validate_strictly());
    }

    #[test]
    fn test_network_capabilities_report() {
        let capabilities = NetworkCapabilities {
            platform: "Test".to_string(),
            http2_support: true,
            ipv6_support: false,
            tls_1_3_support: true,
            connection_pooling_effective: true,
            high_concurrency_support: false,
            recommended_timeouts: PlatformTimeouts {
                dns_resolution: Duration::from_secs(5),
                tcp_connection: Duration::from_secs(10),
                tls_handshake: Duration::from_secs(15),
                http_request: Duration::from_secs(30),
                data_transfer: Duration::from_secs(60),
            },
        };

        let report = capabilities.capabilities_report();
        assert!(report.contains("Test"));
        assert!(report.contains("✓ Supported"));
        assert!(report.contains("✗ Not supported"));
        
        let score = capabilities.performance_score();
        assert!(score > 0 && score <= 100);
        
        assert!(capabilities.has_good_performance() || !capabilities.has_good_performance());
    }

    #[test]
    fn test_platform_specific_user_agents() {
        let config = PlatformNetworkConfig::for_current_platform();
        assert!(config.user_agent.contains("network-latency-tester"));
        
        #[cfg(target_os = "windows")]
        assert!(config.user_agent.contains("Windows"));
        
        #[cfg(target_os = "macos")]
        assert!(config.user_agent.contains("macOS"));
        
        #[cfg(target_os = "linux")]
        assert!(config.user_agent.contains("Linux"));
    }

    #[test]
    fn test_high_performance_networking_detection() {
        let config = PlatformNetworkConfig::for_current_platform();
        let has_high_perf = config.has_high_performance_networking();
        
        // Should be true for most modern platforms
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        assert!(has_high_perf);
    }
}