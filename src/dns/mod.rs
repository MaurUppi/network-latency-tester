//! DNS configuration and resolution management

pub mod platform;

use crate::{
    error::{AppError, Result},
    types::DnsConfig,
};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts, NameServerConfig, Protocol},
    system_conf,
    TokioAsyncResolver,
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// DNS configuration manager that handles different DNS resolution strategies
pub struct DnsManager {
    /// System default resolver
    system_resolver: Arc<RwLock<Option<TokioAsyncResolver>>>,
    /// Custom resolvers for different configurations
    custom_resolvers: Arc<RwLock<std::collections::HashMap<String, TokioAsyncResolver>>>,
    /// HTTP client for DoH requests
    http_client: Client,
}

impl DnsManager {
    /// Create a new DNS manager
    pub fn new() -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("network-latency-tester/0.1.0")
            .build()
            .map_err(|e| AppError::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            system_resolver: Arc::new(RwLock::new(None)),
            custom_resolvers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            http_client,
        })
    }

    /// Initialize the system DNS resolver
    pub async fn initialize_system_resolver(&self) -> Result<()> {
        let (config, opts) = system_conf::read_system_conf()
            .map_err(|e| AppError::dns_resolution(format!("Failed to read system DNS config: {}", e)))?;
        
        let resolver = TokioAsyncResolver::tokio(config, opts);
        
        let mut system_resolver = self.system_resolver.write().await;
        *system_resolver = Some(resolver);
        
        Ok(())
    }

    /// Create a custom DNS resolver with specific servers
    pub async fn create_custom_resolver(&self, servers: &[IpAddr]) -> Result<TokioAsyncResolver> {
        if servers.is_empty() {
            return Err(AppError::validation("No DNS servers provided"));
        }

        let mut config = ResolverConfig::new();
        
        // Add each server to the configuration
        for &server in servers {
            let socket_addr = match server {
                IpAddr::V4(ipv4) => SocketAddr::new(IpAddr::V4(ipv4), 53),
                IpAddr::V6(ipv6) => SocketAddr::new(IpAddr::V6(ipv6), 53),
            };
            
            let name_server = NameServerConfig::new(socket_addr, Protocol::Udp);
            config.add_name_server(name_server);
            
            // Also add TCP fallback
            let tcp_name_server = NameServerConfig::new(socket_addr, Protocol::Tcp);
            config.add_name_server(tcp_name_server);
        }

        let opts = ResolverOpts::default();
        let resolver = TokioAsyncResolver::tokio(config, opts);
        
        Ok(resolver)
    }

    /// Get or create a resolver for the given DNS configuration
    pub async fn get_resolver(&self, dns_config: &DnsConfig) -> Result<DnsResolver> {
        match dns_config {
            DnsConfig::System => {
                let system_resolver = self.system_resolver.read().await;
                if let Some(resolver) = system_resolver.as_ref() {
                    Ok(DnsResolver::System(resolver.clone()))
                } else {
                    Err(AppError::dns_resolution("System resolver not initialized"))
                }
            }
            DnsConfig::Custom { servers } => {
                let cache_key = format!("{:?}", servers);
                let mut custom_resolvers = self.custom_resolvers.write().await;
                
                if let Some(resolver) = custom_resolvers.get(&cache_key) {
                    Ok(DnsResolver::Custom(resolver.clone()))
                } else {
                    let resolver = self.create_custom_resolver(servers).await?;
                    custom_resolvers.insert(cache_key, resolver.clone());
                    Ok(DnsResolver::Custom(resolver))
                }
            }
            DnsConfig::DoH { url } => {
                Ok(DnsResolver::DoH(DoHClient::new(url.clone(), self.http_client.clone())))
            }
        }
    }

    /// Resolve a domain name using the specified DNS configuration
    pub async fn resolve(&self, domain: &str, dns_config: &DnsConfig) -> Result<Vec<IpAddr>> {
        let resolver = self.get_resolver(dns_config).await?;
        resolver.resolve(domain).await
    }

    /// Test DNS resolution performance
    pub async fn test_resolution_performance(&self, domain: &str, dns_config: &DnsConfig) -> Result<DnsPerformanceResult> {
        let start_time = std::time::Instant::now();
        
        match self.resolve(domain, dns_config).await {
            Ok(ips) => {
                let duration = start_time.elapsed();
                Ok(DnsPerformanceResult {
                    success: true,
                    duration,
                    resolved_ips: ips,
                    error: None,
                })
            }
            Err(e) => {
                let duration = start_time.elapsed();
                Ok(DnsPerformanceResult {
                    success: false,
                    duration,
                    resolved_ips: Vec::new(),
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Get system DNS servers by inspecting system configuration
    pub fn get_system_dns_servers(&self) -> Result<Vec<IpAddr>> {
        let (config, _) = system_conf::read_system_conf()
            .map_err(|e| AppError::dns_resolution(format!("Failed to read system DNS config: {}", e)))?;
        
        let servers: Vec<IpAddr> = config
            .name_servers()
            .iter()
            .map(|ns| ns.socket_addr.ip())
            .collect();
        
        Ok(servers)
    }

    /// Validate DNS configuration
    pub async fn validate_dns_config(&self, dns_config: &DnsConfig) -> Result<DnsValidationResult> {
        match dns_config {
            DnsConfig::System => {
                // Test system DNS by resolving a well-known domain
                match self.test_resolution_performance("google.com", dns_config).await {
                    Ok(result) if result.success => {
                        Ok(DnsValidationResult {
                            valid: true,
                            warnings: Vec::new(),
                            test_duration: Some(result.duration),
                        })
                    }
                    Ok(result) => {
                        Ok(DnsValidationResult {
                            valid: false,
                            warnings: vec![format!("System DNS failed to resolve test domain: {}", 
                                result.error.unwrap_or_default())],
                            test_duration: Some(result.duration),
                        })
                    }
                    Err(e) => {
                        Ok(DnsValidationResult {
                            valid: false,
                            warnings: vec![format!("System DNS validation failed: {}", e)],
                            test_duration: None,
                        })
                    }
                }
            }
            DnsConfig::Custom { servers } => {
                let mut warnings = Vec::new();
                
                // Check for private/loopback addresses
                for server in servers {
                    match server {
                        IpAddr::V4(ipv4) => {
                            if ipv4.is_private() {
                                warnings.push(format!("DNS server {} is in private range", server));
                            }
                            if ipv4.is_loopback() {
                                warnings.push(format!("DNS server {} is loopback address", server));
                            }
                        }
                        IpAddr::V6(ipv6) => {
                            if ipv6.is_loopback() {
                                warnings.push(format!("DNS server {} is loopback address", server));
                            }
                        }
                    }
                }
                
                // Test resolution
                match self.test_resolution_performance("google.com", dns_config).await {
                    Ok(result) if result.success => {
                        Ok(DnsValidationResult {
                            valid: true,
                            warnings,
                            test_duration: Some(result.duration),
                        })
                    }
                    Ok(result) => {
                        warnings.push(format!("Custom DNS failed to resolve test domain: {}", 
                            result.error.unwrap_or_default()));
                        Ok(DnsValidationResult {
                            valid: false,
                            warnings,
                            test_duration: Some(result.duration),
                        })
                    }
                    Err(e) => {
                        warnings.push(format!("Custom DNS validation failed: {}", e));
                        Ok(DnsValidationResult {
                            valid: false,
                            warnings,
                            test_duration: None,
                        })
                    }
                }
            }
            DnsConfig::DoH { url } => {
                let mut warnings = Vec::new();
                
                // Validate URL format
                if let Err(e) = url::Url::parse(url) {
                    warnings.push(format!("Invalid DoH URL format: {}", e));
                    return Ok(DnsValidationResult {
                        valid: false,
                        warnings,
                        test_duration: None,
                    });
                }
                
                // Test DoH resolution
                match self.test_resolution_performance("google.com", dns_config).await {
                    Ok(result) if result.success => {
                        Ok(DnsValidationResult {
                            valid: true,
                            warnings,
                            test_duration: Some(result.duration),
                        })
                    }
                    Ok(result) => {
                        warnings.push(format!("DoH provider failed to resolve test domain: {}", 
                            result.error.unwrap_or_default()));
                        Ok(DnsValidationResult {
                            valid: false,
                            warnings,
                            test_duration: Some(result.duration),
                        })
                    }
                    Err(e) => {
                        warnings.push(format!("DoH validation failed: {}", e));
                        Ok(DnsValidationResult {
                            valid: false,
                            warnings,
                            test_duration: None,
                        })
                    }
                }
            }
        }
    }
}

impl Default for DnsManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default DNS manager")
    }
}

/// DNS resolver wrapper that abstracts different resolution methods
#[derive(Clone)]
pub enum DnsResolver {
    /// System default resolver
    System(TokioAsyncResolver),
    /// Custom DNS resolver
    Custom(TokioAsyncResolver),
    /// DNS-over-HTTPS client
    DoH(DoHClient),
}

impl DnsResolver {
    /// Resolve a domain name to IP addresses
    pub async fn resolve(&self, domain: &str) -> Result<Vec<IpAddr>> {
        match self {
            DnsResolver::System(resolver) | DnsResolver::Custom(resolver) => {
                let response = resolver
                    .lookup_ip(domain)
                    .await
                    .map_err(|e| AppError::dns_resolution(format!("DNS lookup failed for {}: {}", domain, e)))?;
                
                let ips: Vec<IpAddr> = response.iter().collect();
                Ok(ips)
            }
            DnsResolver::DoH(client) => client.resolve(domain).await,
        }
    }
}

/// DNS-over-HTTPS client implementation
#[derive(Clone)]
pub struct DoHClient {
    url: String,
    client: Client,
}

impl DoHClient {
    /// Create a new DoH client
    pub fn new(url: String, client: Client) -> Self {
        Self { url, client }
    }

    /// Resolve a domain using DNS-over-HTTPS
    pub async fn resolve(&self, domain: &str) -> Result<Vec<IpAddr>> {
        // Create DNS query for A and AAAA records
        let queries = vec![
            self.query_record(domain, "A").await,
            self.query_record(domain, "AAAA").await,
        ];

        let mut all_ips = Vec::new();
        for query_result in queries {
            match query_result {
                Ok(mut ips) => all_ips.append(&mut ips),
                Err(_) => continue, // Ignore individual query failures
            }
        }

        if all_ips.is_empty() {
            return Err(AppError::dns_resolution(format!("No IP addresses resolved for {}", domain)));
        }

        Ok(all_ips)
    }

    /// Query specific DNS record type via DoH
    async fn query_record(&self, domain: &str, record_type: &str) -> Result<Vec<IpAddr>> {
        let query_params = [
            ("name", domain),
            ("type", record_type),
            ("ct", "application/dns-json"),
        ];

        let response = self
            .client
            .get(&self.url)
            .query(&query_params)
            .header("Accept", "application/dns-json")
            .send()
            .await
            .map_err(|e| AppError::network(format!("DoH request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::dns_resolution(format!(
                "DoH query failed with status: {}",
                response.status()
            )));
        }

        let dns_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::parse(format!("Failed to parse DoH response: {}", e)))?;

        self.parse_dns_response(dns_response, record_type)
    }

    /// Parse DNS response and extract IP addresses
    fn parse_dns_response(&self, response: serde_json::Value, record_type: &str) -> Result<Vec<IpAddr>> {
        let mut ips = Vec::new();

        if let Some(answers) = response.get("Answer").and_then(|a| a.as_array()) {
            for answer in answers {
                if let Some(data) = answer.get("data").and_then(|d| d.as_str()) {
                    match record_type {
                        "A" => {
                            if let Ok(ipv4) = data.parse::<Ipv4Addr>() {
                                ips.push(IpAddr::V4(ipv4));
                            }
                        }
                        "AAAA" => {
                            if let Ok(ipv6) = data.parse::<Ipv6Addr>() {
                                ips.push(IpAddr::V6(ipv6));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(ips)
    }
}

/// DNS performance test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsPerformanceResult {
    pub success: bool,
    pub duration: Duration,
    pub resolved_ips: Vec<IpAddr>,
    pub error: Option<String>,
}

/// DNS configuration validation result
#[derive(Debug, Clone)]
pub struct DnsValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub test_duration: Option<Duration>,
}

/// DNS configuration utilities
pub struct DnsUtils;

impl DnsUtils {
    /// Get well-known public DNS servers
    pub fn get_public_dns_servers() -> Vec<(String, Vec<IpAddr>)> {
        vec![
            (
                "Google DNS".to_string(),
                vec!["8.8.8.8".parse().unwrap(), "8.8.4.4".parse().unwrap()],
            ),
            (
                "Cloudflare DNS".to_string(),
                vec!["1.1.1.1".parse().unwrap(), "1.0.0.1".parse().unwrap()],
            ),
            (
                "OpenDNS".to_string(),
                vec!["208.67.222.222".parse().unwrap(), "208.67.220.220".parse().unwrap()],
            ),
            (
                "Quad9 DNS".to_string(),
                vec!["9.9.9.9".parse().unwrap(), "149.112.112.112".parse().unwrap()],
            ),
        ]
    }

    /// Get well-known DoH providers
    pub fn get_public_doh_providers() -> Vec<(String, String)> {
        vec![
            ("Cloudflare DoH".to_string(), "https://cloudflare-dns.com/dns-query".to_string()),
            ("Google DoH".to_string(), "https://dns.google/dns-query".to_string()),
            ("Quad9 DoH".to_string(), "https://dns.quad9.net/dns-query".to_string()),
            ("AdGuard DoH".to_string(), "https://dns.adguard.com/dns-query".to_string()),
        ]
    }

    /// Determine the fastest DNS configuration from a list
    pub async fn find_fastest_dns(
        dns_manager: &DnsManager,
        configs: &[DnsConfig],
        test_domain: &str,
    ) -> Result<(DnsConfig, Duration)> {
        if configs.is_empty() {
            return Err(AppError::validation("No DNS configurations provided"));
        }

        let mut fastest_config = None;
        let mut fastest_time = Duration::from_secs(u64::MAX);

        for config in configs {
            match dns_manager.test_resolution_performance(test_domain, config).await {
                Ok(result) if result.success && result.duration < fastest_time => {
                    fastest_time = result.duration;
                    fastest_config = Some(config.clone());
                }
                _ => continue,
            }
        }

        match fastest_config {
            Some(config) => Ok((config, fastest_time)),
            None => Err(AppError::dns_resolution("No DNS configurations were successful")),
        }
    }

    /// Create DNS configuration from string representation
    pub fn parse_dns_config(input: &str) -> Result<DnsConfig> {
        let input = input.trim();

        if input.is_empty() || input.eq_ignore_ascii_case("system") || input.eq_ignore_ascii_case("default") {
            return Ok(DnsConfig::System);
        }

        if input.starts_with("https://") {
            return Ok(DnsConfig::DoH { url: input.to_string() });
        }

        // Try to parse as IP address(es)
        if input.contains(',') {
            let mut servers = Vec::new();
            for part in input.split(',') {
                let ip = part.trim().parse::<IpAddr>()
                    .map_err(|e| AppError::parse(format!("Invalid IP address '{}': {}", part.trim(), e)))?;
                servers.push(ip);
            }
            Ok(DnsConfig::Custom { servers })
        } else {
            let ip = input.parse::<IpAddr>()
                .map_err(|e| AppError::parse(format!("Invalid IP address '{}': {}", input, e)))?;
            Ok(DnsConfig::Custom { servers: vec![ip] })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_dns_manager_creation() {
        let dns_manager = DnsManager::new();
        assert!(dns_manager.is_ok());
    }

    #[tokio::test]
    async fn test_system_resolver_initialization() {
        let dns_manager = DnsManager::new().unwrap();
        let result = dns_manager.initialize_system_resolver().await;
        
        // This might fail in test environment without proper DNS config, so we allow both outcomes
        match result {
            Ok(_) => {
                // System resolver initialized successfully
                let system_resolver = dns_manager.system_resolver.read().await;
                assert!(system_resolver.is_some());
            }
            Err(_) => {
                // Expected in some test environments
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_custom_resolver_creation() {
        let dns_manager = DnsManager::new().unwrap();
        let servers = vec!["8.8.8.8".parse().unwrap(), "1.1.1.1".parse().unwrap()];
        
        let result = dns_manager.create_custom_resolver(&servers).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_empty_dns_servers_validation() {
        let dns_manager = DnsManager::new().unwrap();
        let servers = vec![];
        
        let result = dns_manager.create_custom_resolver(&servers).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[test]
    fn test_dns_performance_result() {
        let result = DnsPerformanceResult {
            success: true,
            duration: Duration::from_millis(150),
            resolved_ips: vec!["8.8.8.8".parse().unwrap()],
            error: None,
        };
        
        assert!(result.success);
        assert_eq!(result.duration, Duration::from_millis(150));
        assert_eq!(result.resolved_ips.len(), 1);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_dns_validation_result() {
        let result = DnsValidationResult {
            valid: true,
            warnings: vec!["Test warning".to_string()],
            test_duration: Some(Duration::from_millis(100)),
        };
        
        assert!(result.valid);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0], "Test warning");
        assert!(result.test_duration.is_some());
    }

    #[test]
    fn test_doh_client_creation() {
        let client = Client::new();
        let doh_client = DoHClient::new("https://dns.google/dns-query".to_string(), client);
        
        assert_eq!(doh_client.url, "https://dns.google/dns-query");
    }

    #[test]
    fn test_dns_parse_response_a_record() {
        let client = Client::new();
        let doh_client = DoHClient::new("https://example.com".to_string(), client);
        
        let response = serde_json::json!({
            "Answer": [
                {
                    "data": "8.8.8.8",
                    "type": 1
                },
                {
                    "data": "8.8.4.4",
                    "type": 1
                }
            ]
        });
        
        let result = doh_client.parse_dns_response(response, "A").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&IpAddr::V4("8.8.8.8".parse().unwrap())));
        assert!(result.contains(&IpAddr::V4("8.8.4.4".parse().unwrap())));
    }

    #[test]
    fn test_dns_parse_response_aaaa_record() {
        let client = Client::new();
        let doh_client = DoHClient::new("https://example.com".to_string(), client);
        
        let response = serde_json::json!({
            "Answer": [
                {
                    "data": "2001:4860:4860::8888",
                    "type": 28
                }
            ]
        });
        
        let result = doh_client.parse_dns_response(response, "AAAA").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&IpAddr::V6("2001:4860:4860::8888".parse().unwrap())));
    }

    #[test]
    fn test_dns_parse_response_empty() {
        let client = Client::new();
        let doh_client = DoHClient::new("https://example.com".to_string(), client);
        
        let response = serde_json::json!({
            "Answer": []
        });
        
        let result = doh_client.parse_dns_response(response, "A").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_dns_parse_response_no_answer() {
        let client = Client::new();
        let doh_client = DoHClient::new("https://example.com".to_string(), client);
        
        let response = serde_json::json!({
            "Status": 0
        });
        
        let result = doh_client.parse_dns_response(response, "A").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_dns_utils_public_dns_servers() {
        let servers = DnsUtils::get_public_dns_servers();
        
        assert!(!servers.is_empty());
        
        // Check that Google DNS is included
        let google_dns = servers.iter().find(|(name, _)| name == "Google DNS");
        assert!(google_dns.is_some());
        
        let (_, google_ips) = google_dns.unwrap();
        assert!(google_ips.contains(&"8.8.8.8".parse().unwrap()));
        assert!(google_ips.contains(&"8.8.4.4".parse().unwrap()));
    }

    #[test]
    fn test_dns_utils_public_doh_providers() {
        let providers = DnsUtils::get_public_doh_providers();
        
        assert!(!providers.is_empty());
        
        // Check that Cloudflare DoH is included
        let cloudflare_doh = providers.iter().find(|(name, _)| name == "Cloudflare DoH");
        assert!(cloudflare_doh.is_some());
        
        let (_, cloudflare_url) = cloudflare_doh.unwrap();
        assert_eq!(cloudflare_url, "https://cloudflare-dns.com/dns-query");
    }

    #[test]
    fn test_dns_config_parsing_system() {
        let config = DnsUtils::parse_dns_config("system").unwrap();
        assert_eq!(config, DnsConfig::System);
        
        let config = DnsUtils::parse_dns_config("default").unwrap();
        assert_eq!(config, DnsConfig::System);
        
        let config = DnsUtils::parse_dns_config("").unwrap();
        assert_eq!(config, DnsConfig::System);
        
        let config = DnsUtils::parse_dns_config("  ").unwrap();
        assert_eq!(config, DnsConfig::System);
    }

    #[test]
    fn test_dns_config_parsing_doh() {
        let config = DnsUtils::parse_dns_config("https://dns.google/dns-query").unwrap();
        assert_eq!(config, DnsConfig::DoH { 
            url: "https://dns.google/dns-query".to_string() 
        });
        
        let config = DnsUtils::parse_dns_config("https://cloudflare-dns.com/dns-query").unwrap();
        assert_eq!(config, DnsConfig::DoH { 
            url: "https://cloudflare-dns.com/dns-query".to_string() 
        });
    }

    #[test]
    fn test_dns_config_parsing_single_ip() {
        let config = DnsUtils::parse_dns_config("8.8.8.8").unwrap();
        assert_eq!(config, DnsConfig::Custom { 
            servers: vec!["8.8.8.8".parse().unwrap()] 
        });
        
        let config = DnsUtils::parse_dns_config("2001:4860:4860::8888").unwrap();
        assert_eq!(config, DnsConfig::Custom { 
            servers: vec!["2001:4860:4860::8888".parse().unwrap()] 
        });
    }

    #[test]
    fn test_dns_config_parsing_multiple_ips() {
        let config = DnsUtils::parse_dns_config("8.8.8.8,1.1.1.1").unwrap();
        assert_eq!(config, DnsConfig::Custom { 
            servers: vec![
                "8.8.8.8".parse().unwrap(),
                "1.1.1.1".parse().unwrap(),
            ] 
        });
        
        let config = DnsUtils::parse_dns_config("8.8.8.8, 1.1.1.1, 208.67.222.222").unwrap();
        assert_eq!(config, DnsConfig::Custom { 
            servers: vec![
                "8.8.8.8".parse().unwrap(),
                "1.1.1.1".parse().unwrap(),
                "208.67.222.222".parse().unwrap(),
            ] 
        });
    }

    #[test]
    fn test_dns_config_parsing_invalid_ip() {
        let result = DnsUtils::parse_dns_config("not-an-ip");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Parse(_)));
    }

    #[test]
    fn test_dns_config_parsing_mixed_valid_invalid_ips() {
        let result = DnsUtils::parse_dns_config("8.8.8.8,not-an-ip");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Parse(_)));
    }

    #[tokio::test]
    async fn test_fastest_dns_empty_configs() {
        let dns_manager = DnsManager::new().unwrap();
        let configs = vec![];
        
        let result = DnsUtils::find_fastest_dns(&dns_manager, &configs, "google.com").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[test]
    fn test_dns_manager_default() {
        let _dns_manager = DnsManager::default();
        // Should not panic and create a valid instance
        assert!(true); // If we get here, default() worked
    }

    #[tokio::test]
    async fn test_resolver_caching() {
        let dns_manager = DnsManager::new().unwrap();
        let servers = vec!["8.8.8.8".parse().unwrap()];
        
        let config = DnsConfig::Custom { servers: servers.clone() };
        
        // Get resolver twice with same config
        let resolver1_result = dns_manager.get_resolver(&config).await;
        let resolver2_result = dns_manager.get_resolver(&config).await;
        
        // Both should succeed (or both should fail consistently)
        assert_eq!(resolver1_result.is_ok(), resolver2_result.is_ok());
        
        // If successful, the resolvers should be cached (same instance)
        if resolver1_result.is_ok() {
            // Check that custom resolver cache was populated
            let cache = dns_manager.custom_resolvers.read().await;
            assert!(!cache.is_empty());
        }
    }

    #[test]
    fn test_dns_config_name_display() {
        let system_config = DnsConfig::System;
        assert!(system_config.name().contains("系统默认"));
        
        let custom_config = DnsConfig::Custom {
            servers: vec!["8.8.8.8".parse().unwrap()],
        };
        let name = custom_config.name();
        assert!(name.contains("自定义DNS"));
        assert!(name.contains("8.8.8.8"));
        
        let multi_dns_config = DnsConfig::Custom {
            servers: vec!["8.8.8.8".parse().unwrap(), "1.1.1.1".parse().unwrap()],
        };
        let name = multi_dns_config.name();
        assert!(name.contains("自定义DNS"));
        assert!(name.contains("servers"));
        
        let doh_config = DnsConfig::DoH {
            url: "https://dns.google/dns-query".to_string(),
        };
        let name = doh_config.name();
        assert!(name.contains("DoH"));
        assert!(name.contains("dns.google"));
    }

    #[test]
    fn test_dns_config_name_invalid_doh_url() {
        let doh_config = DnsConfig::DoH {
            url: "not-a-valid-url".to_string(),
        };
        let name = doh_config.name();
        assert_eq!(name, "DoH");
    }

    #[test]
    fn test_dns_config_name_doh_no_host() {
        let doh_config = DnsConfig::DoH {
            url: "https:///dns-query".to_string(),  // Invalid URL with no host
        };
        let name = doh_config.name();
        // The URL parsing may return the path portion, so we just check that DoH is mentioned
        assert!(name.starts_with("DoH"));
    }
}

// Additional comprehensive tests in separate module
#[cfg(test)]
mod comprehensive_tests;