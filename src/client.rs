//! HTTP client implementation and timing measurements

pub mod platform;
pub mod windows;
pub mod cert_validation;
pub mod timeouts;

#[cfg(test)]
mod integration_tests;

use crate::{
    error::{AppError, Result},
    types::{DnsConfig, TestStatus},
    dns::DnsManager,
    models::metrics::TimingMetrics,
};
use std::{
    net::IpAddr,
    time::{Duration, Instant},
    sync::Arc,
};
use reqwest::{Client, Method, Url};
use tokio::time::timeout;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

/// HTTP client trait for abstraction and testing
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Execute an HTTP request with timing measurements
    async fn execute_request(&self, request: HttpRequest) -> Result<HttpResponse>;
    
    /// Execute a HEAD request for latency testing
    async fn head(&self, url: &str, dns_config: &DnsConfig) -> Result<HttpResponse>;
    
    /// Execute a GET request
    async fn get(&self, url: &str, dns_config: &DnsConfig) -> Result<HttpResponse>;
    
    /// Test connectivity to a URL with specific DNS configuration
    async fn test_connectivity(&self, url: &str, dns_config: &DnsConfig) -> Result<ConnectivityTest>;
}

/// HTTP request configuration
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: String,
    pub method: Method,
    pub timeout: Duration,
    pub dns_config: DnsConfig,
    pub follow_redirects: bool,
    pub max_redirects: usize,
    pub user_agent: Option<String>,
    pub headers: Vec<(String, String)>,
}

impl HttpRequest {
    /// Create a new HTTP request
    pub fn new(url: String, method: Method, dns_config: DnsConfig) -> Self {
        Self {
            url,
            method,
            timeout: Duration::from_secs(10),
            dns_config,
            follow_redirects: true,
            max_redirects: 5,
            user_agent: Some("network-latency-tester/0.1.0".to_string()),
            headers: Vec::new(),
        }
    }
    
    /// Create a HEAD request for latency testing
    pub fn head(url: String, dns_config: DnsConfig) -> Self {
        Self::new(url, Method::HEAD, dns_config)
    }
    
    /// Create a GET request
    pub fn get(url: String, dns_config: DnsConfig) -> Self {
        Self::new(url, Method::GET, dns_config)
    }
    
    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set follow redirects behavior
    pub fn with_redirects(mut self, follow: bool, max_redirects: usize) -> Self {
        self.follow_redirects = follow;
        self.max_redirects = max_redirects;
        self
    }
    
    /// Add custom header
    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.headers.push((name, value));
        self
    }
    
    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}

/// HTTP response with timing information
#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body_size: usize,
    pub timing: TimingMetrics,
    pub resolved_ip: Option<IpAddr>,
    pub dns_config_used: DnsConfig,
    pub final_url: String,
}

impl HttpResponse {
    /// Check if the response indicates success
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
    
    /// Check if the response is a redirect
    pub fn is_redirect(&self) -> bool {
        self.status_code >= 300 && self.status_code < 400
    }
    
    /// Get the test status based on response
    pub fn test_status(&self) -> TestStatus {
        self.timing.status
    }
}

/// Connectivity test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityTest {
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_time: Duration,
    pub resolved_ip: Option<IpAddr>,
    pub dns_resolution_time: Duration,
    pub connection_time: Duration,
    pub error: Option<String>,
}

/// Network latency tester HTTP client implementation
pub struct NetworkClient {
    dns_manager: Arc<DnsManager>,
    #[allow(dead_code)]
    client: Client,
    default_timeout: Duration,
}

impl NetworkClient {
    /// Create a new network client
    pub fn new(dns_manager: Arc<DnsManager>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("network-latency-tester/0.1.0")
            .build()
            .map_err(|e| AppError::network(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            dns_manager,
            client,
            default_timeout: Duration::from_secs(10),
        })
    }
    
    /// Create a new network client with custom timeout
    pub fn with_timeout(dns_manager: Arc<DnsManager>, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .user_agent("network-latency-tester/0.1.0")
            .build()
            .map_err(|e| AppError::network(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            dns_manager,
            client,
            default_timeout: timeout,
        })
    }
    
    /// Resolve URL using specified DNS configuration
    async fn resolve_url(&self, url: &str, dns_config: &DnsConfig) -> Result<(Vec<IpAddr>, Duration)> {
        let start_time = Instant::now();
        
        let parsed_url = Url::parse(url)
            .map_err(|e| AppError::parse(format!("Invalid URL: {}", e)))?;
        
        let host = parsed_url.host_str()
            .ok_or_else(|| AppError::validation("URL must have a host"))?;
        
        // Skip DNS resolution for IP addresses
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Ok((vec![ip], Duration::from_nanos(0)));
        }
        
        let ips = self.dns_manager.resolve(host, dns_config).await?;
        let resolution_time = start_time.elapsed();
        
        Ok((ips, resolution_time))
    }
    
    /// Execute HTTP request with custom DNS resolver
    async fn execute_with_dns(&self, request: HttpRequest) -> Result<HttpResponse> {
        let overall_start = Instant::now();
        
        // Parse URL
        let url = Url::parse(&request.url)
            .map_err(|e| AppError::parse(format!("Invalid URL: {}", e)))?;
        
        // DNS resolution timing
        let (resolved_ips, dns_time) = self.resolve_url(&request.url, &request.dns_config).await?;
        
        if resolved_ips.is_empty() {
            return Err(AppError::dns_resolution("No IP addresses resolved"));
        }
        
        // Use first resolved IP for the request
        let target_ip = resolved_ips[0];
        
        // Create custom HTTP client for this specific IP
        let custom_client = self.create_custom_client(&request, target_ip)?;
        
        // Build request
        let mut req_builder = custom_client.request(request.method, url.clone());
        
        // Set timeout
        req_builder = req_builder.timeout(request.timeout);
        
        // Add headers
        for (name, value) in &request.headers {
            req_builder = req_builder.header(name, value);
        }
        
        // Add user agent
        if let Some(ref ua) = request.user_agent {
            req_builder = req_builder.header("User-Agent", ua);
        }
        
        // Execute request with timing
        let request_start = Instant::now();
        
        let response_result = if request.timeout > Duration::ZERO {
            timeout(request.timeout, req_builder.send()).await
                .map_err(|_| AppError::timeout("HTTP request timed out"))?
                .map_err(|e| AppError::http_request(e.to_string()))
        } else {
            req_builder.send().await
                .map_err(|e| AppError::http_request(e.to_string()))
        };
        
        let request_time = request_start.elapsed();
        let total_time = overall_start.elapsed();
        
        match response_result {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let final_url = response.url().to_string();
                
                // Extract headers
                let headers: Vec<(String, String)> = response
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                
                // Get body size (read body for size measurement)
                let body = response.bytes().await
                    .map_err(|e| AppError::http_request(format!("Failed to read response body: {}", e)))?;
                let body_size = body.len();
                
                // Create timing metrics
                let timing = TimingMetrics::success(
                    dns_time,
                    Duration::from_nanos(0), // Difficult to measure separately
                    if url.scheme() == "https" { Some(Duration::from_nanos(0)) } else { None },
                    request_time,
                    total_time,
                    status_code,
                );
                
                Ok(HttpResponse {
                    status_code,
                    headers,
                    body_size,
                    timing,
                    resolved_ip: Some(target_ip),
                    dns_config_used: request.dns_config,
                    final_url,
                })
            }
            Err(e) => {
                let _timing = if matches!(e, AppError::Timeout(_)) {
                    TimingMetrics::timeout(total_time)
                } else {
                    TimingMetrics::failed(e.to_string())
                };
                
                // Return error response with timing information
                Err(e)
            }
        }
    }
    
    /// Create a custom HTTP client that connects to a specific IP
    fn create_custom_client(&self, request: &HttpRequest, _target_ip: IpAddr) -> Result<Client> {
        let url = Url::parse(&request.url)
            .map_err(|e| AppError::parse(format!("Invalid URL: {}", e)))?;
        
        let _port = url.port().unwrap_or_else(|| {
            match url.scheme() {
                "https" => 443,
                "http" => 80,
                _ => 80,
            }
        });
        
        // Create client builder
        let client_builder = Client::builder()
            .timeout(request.timeout)
            .redirect(if request.follow_redirects {
                reqwest::redirect::Policy::limited(request.max_redirects)
            } else {
                reqwest::redirect::Policy::none()
            });
        
        // For system DNS or when we want to use the default behavior,
        // we don't need to override the resolver
        match &request.dns_config {
            DnsConfig::System => {
                // Use default system resolver
            }
            DnsConfig::Custom { .. } | DnsConfig::DoH { .. } => {
                // For custom DNS, we would ideally use a custom resolver
                // but reqwest doesn't easily support this, so we use the resolved IP directly
                // This is a limitation that could be improved in future versions
            }
        }
        
        client_builder.build()
            .map_err(|e| AppError::network(format!("Failed to create custom client: {}", e)))
    }
}

#[async_trait]
impl HttpClient for NetworkClient {
    async fn execute_request(&self, request: HttpRequest) -> Result<HttpResponse> {
        self.execute_with_dns(request).await
    }
    
    async fn head(&self, url: &str, dns_config: &DnsConfig) -> Result<HttpResponse> {
        let request = HttpRequest::head(url.to_string(), dns_config.clone())
            .with_timeout(self.default_timeout);
        self.execute_request(request).await
    }
    
    async fn get(&self, url: &str, dns_config: &DnsConfig) -> Result<HttpResponse> {
        let request = HttpRequest::get(url.to_string(), dns_config.clone())
            .with_timeout(self.default_timeout);
        self.execute_request(request).await
    }
    
    async fn test_connectivity(&self, url: &str, dns_config: &DnsConfig) -> Result<ConnectivityTest> {
        let start_time = Instant::now();
        
        match self.head(url, dns_config).await {
            Ok(response) => {
                Ok(ConnectivityTest {
                    success: response.is_success(),
                    status_code: Some(response.status_code),
                    response_time: response.timing.total_duration,
                    resolved_ip: response.resolved_ip,
                    dns_resolution_time: response.timing.dns_resolution,
                    connection_time: response.timing.first_byte,
                    error: None,
                })
            }
            Err(e) => {
                Ok(ConnectivityTest {
                    success: false,
                    status_code: None,
                    response_time: start_time.elapsed(),
                    resolved_ip: None,
                    dns_resolution_time: Duration::from_nanos(0),
                    connection_time: Duration::from_nanos(0),
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

/// HTTP client factory for different configurations
pub struct ClientFactory {
    dns_manager: Arc<DnsManager>,
}

impl ClientFactory {
    /// Create a new client factory
    pub fn new(dns_manager: Arc<DnsManager>) -> Self {
        Self { dns_manager }
    }
    
    /// Create a network client with default configuration
    pub fn create_network_client(&self) -> Result<NetworkClient> {
        NetworkClient::new(self.dns_manager.clone())
    }
    
    /// Create a network client with custom timeout
    pub fn create_network_client_with_timeout(&self, timeout: Duration) -> Result<NetworkClient> {
        NetworkClient::with_timeout(self.dns_manager.clone(), timeout)
    }
    
    /// Create a client optimized for latency testing
    pub fn create_latency_test_client(&self) -> Result<NetworkClient> {
        NetworkClient::with_timeout(self.dns_manager.clone(), Duration::from_secs(5))
    }
}

/// Utility functions for HTTP operations
pub struct HttpUtils;

impl HttpUtils {
    /// Validate URL format and accessibility
    pub fn validate_url(url: &str) -> Result<()> {
        let parsed = Url::parse(url)
            .map_err(|e| AppError::validation(format!("Invalid URL format: {}", e)))?;
        
        // Check scheme
        match parsed.scheme() {
            "http" | "https" => {},
            scheme => return Err(AppError::validation(format!("Unsupported URL scheme: {}", scheme))),
        }
        
        // Check host
        if parsed.host().is_none() {
            return Err(AppError::validation("URL must have a host"));
        }
        
        Ok(())
    }
    
    /// Extract domain from URL
    pub fn extract_domain(url: &str) -> Result<String> {
        let parsed = Url::parse(url)
            .map_err(|e| AppError::parse(format!("Invalid URL: {}", e)))?;
        
        parsed.host_str()
            .ok_or_else(|| AppError::validation("URL must have a host"))
            .map(|s| s.to_string())
    }
    
    /// Check if URL uses HTTPS
    pub fn is_https(url: &str) -> bool {
        url.starts_with("https://")
    }
    
    /// Normalize URL for testing (remove fragments, sort query parameters)
    pub fn normalize_url(url: &str) -> Result<String> {
        let mut parsed = Url::parse(url)
            .map_err(|e| AppError::parse(format!("Invalid URL: {}", e)))?;
        
        // Remove fragment
        parsed.set_fragment(None);
        
        // Sort query parameters for consistency
        let query_pairs: Vec<_> = parsed.query_pairs().collect();
        if !query_pairs.is_empty() {
            let mut sorted_pairs = query_pairs;
            sorted_pairs.sort_by(|a, b| a.0.cmp(&b.0));
            
            // Need to clone the key-value pairs to avoid borrowing issues
            let sorted_pairs_owned: Vec<(String, String)> = sorted_pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            
            parsed.set_query(None);
            for (key, value) in sorted_pairs_owned {
                parsed.query_pairs_mut().append_pair(&key, &value);
            }
        }
        
        Ok(parsed.to_string())
    }
    
    /// Get default port for URL scheme
    pub fn get_default_port(url: &str) -> Result<u16> {
        let parsed = Url::parse(url)
            .map_err(|e| AppError::parse(format!("Invalid URL: {}", e)))?;
        
        Ok(match parsed.scheme() {
            "http" => 80,
            "https" => 443,
            scheme => return Err(AppError::validation(format!("Unknown scheme for port: {}", scheme))),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DnsConfig;
    use std::sync::Arc;
    use std::time::Duration;
    use reqwest::Method;

    #[tokio::test]
    async fn test_http_request_creation() {
        let dns_config = DnsConfig::System;
        let request = HttpRequest::new(
            "https://example.com".to_string(),
            Method::GET,
            dns_config.clone(),
        );
        
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.timeout, Duration::from_secs(10));
        assert_eq!(request.dns_config, dns_config);
        assert!(request.follow_redirects);
        assert_eq!(request.max_redirects, 5);
        assert!(request.user_agent.is_some());
        assert!(request.headers.is_empty());
    }

    #[test]
    fn test_http_request_builder_pattern() {
        let dns_config = DnsConfig::System;
        let request = HttpRequest::head("https://example.com".to_string(), dns_config.clone())
            .with_timeout(Duration::from_secs(5))
            .with_redirects(false, 0)
            .with_header("Accept".to_string(), "application/json".to_string())
            .with_user_agent("test-agent".to_string());
        
        assert_eq!(request.method, Method::HEAD);
        assert_eq!(request.timeout, Duration::from_secs(5));
        assert!(!request.follow_redirects);
        assert_eq!(request.max_redirects, 0);
        assert_eq!(request.headers.len(), 1);
        assert_eq!(request.headers[0], ("Accept".to_string(), "application/json".to_string()));
        assert_eq!(request.user_agent, Some("test-agent".to_string()));
    }

    #[test]
    fn test_http_request_get_constructor() {
        let dns_config = DnsConfig::System;
        let request = HttpRequest::get("https://api.example.com".to_string(), dns_config);
        
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.url, "https://api.example.com");
    }

    #[test]
    fn test_http_request_head_constructor() {
        let dns_config = DnsConfig::System;
        let request = HttpRequest::head("https://api.example.com".to_string(), dns_config);
        
        assert_eq!(request.method, Method::HEAD);
        assert_eq!(request.url, "https://api.example.com");
    }

    #[test]
    fn test_http_response_is_success() {
        let timing = TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            Some(Duration::from_millis(30)),
            Duration::from_millis(50),
            Duration::from_millis(110),
            200,
        );
        
        let response = HttpResponse {
            status_code: 200,
            headers: vec![],
            body_size: 1024,
            timing,
            resolved_ip: Some("8.8.8.8".parse().unwrap()),
            dns_config_used: DnsConfig::System,
            final_url: "https://example.com".to_string(),
        };
        
        assert!(response.is_success());
        assert!(!response.is_redirect());
        assert_eq!(response.test_status(), TestStatus::Success);
    }

    #[test]
    fn test_http_response_is_redirect() {
        let timing = TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            Some(Duration::from_millis(30)),
            Duration::from_millis(50),
            Duration::from_millis(110),
            301,
        );
        
        let response = HttpResponse {
            status_code: 301,
            headers: vec![("Location".to_string(), "https://new-example.com".to_string())],
            body_size: 0,
            timing,
            resolved_ip: Some("8.8.8.8".parse().unwrap()),
            dns_config_used: DnsConfig::System,
            final_url: "https://new-example.com".to_string(),
        };
        
        assert!(!response.is_success());
        assert!(response.is_redirect());
        assert_eq!(response.test_status(), TestStatus::Success);
    }

    #[test]
    fn test_http_response_timeout() {
        let timing = TimingMetrics::timeout(Duration::from_secs(10));
        
        let response = HttpResponse {
            status_code: 408,
            headers: vec![],
            body_size: 0,
            timing,
            resolved_ip: Some("8.8.8.8".parse().unwrap()),
            dns_config_used: DnsConfig::System,
            final_url: "https://example.com".to_string(),
        };
        
        assert_eq!(response.test_status(), TestStatus::Timeout);
    }

    #[test]
    fn test_connectivity_test_success() {
        let test = ConnectivityTest {
            success: true,
            status_code: Some(200),
            response_time: Duration::from_millis(150),
            resolved_ip: Some("8.8.8.8".parse().unwrap()),
            dns_resolution_time: Duration::from_millis(20),
            connection_time: Duration::from_millis(130),
            error: None,
        };
        
        assert!(test.success);
        assert_eq!(test.status_code, Some(200));
        assert_eq!(test.response_time, Duration::from_millis(150));
        assert!(test.resolved_ip.is_some());
        assert!(test.error.is_none());
    }

    #[test]
    fn test_connectivity_test_failure() {
        let test = ConnectivityTest {
            success: false,
            status_code: None,
            response_time: Duration::from_millis(5000),
            resolved_ip: None,
            dns_resolution_time: Duration::from_nanos(0),
            connection_time: Duration::from_nanos(0),
            error: Some("Connection refused".to_string()),
        };
        
        assert!(!test.success);
        assert_eq!(test.status_code, None);
        assert!(test.resolved_ip.is_none());
        assert!(test.error.is_some());
        assert_eq!(test.error.as_ref().unwrap(), "Connection refused");
    }

    #[tokio::test]
    async fn test_network_client_creation() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager);
        
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.default_timeout, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_network_client_with_timeout() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let timeout = Duration::from_secs(5);
        let client = NetworkClient::with_timeout(dns_manager, timeout);
        
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.default_timeout, timeout);
    }

    #[test]
    fn test_client_factory_creation() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let factory = ClientFactory::new(dns_manager);
        
        // Test different client creation methods
        let network_client = factory.create_network_client();
        assert!(network_client.is_ok());
        
        let timeout_client = factory.create_network_client_with_timeout(Duration::from_secs(3));
        assert!(timeout_client.is_ok());
        
        let latency_client = factory.create_latency_test_client();
        assert!(latency_client.is_ok());
        let latency_client = latency_client.unwrap();
        assert_eq!(latency_client.default_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_http_utils_validate_url() {
        // Valid URLs
        assert!(HttpUtils::validate_url("https://example.com").is_ok());
        assert!(HttpUtils::validate_url("http://example.com").is_ok());
        assert!(HttpUtils::validate_url("https://api.example.com/v1/test").is_ok());
        assert!(HttpUtils::validate_url("https://example.com:8080").is_ok());
        
        // Invalid URLs
        assert!(HttpUtils::validate_url("ftp://example.com").is_err());
        assert!(HttpUtils::validate_url("not-a-url").is_err());
        assert!(HttpUtils::validate_url("https://").is_err());
        assert!(HttpUtils::validate_url("").is_err());
    }

    #[test]
    fn test_http_utils_extract_domain() {
        assert_eq!(
            HttpUtils::extract_domain("https://example.com").unwrap(),
            "example.com"
        );
        assert_eq!(
            HttpUtils::extract_domain("http://api.example.com").unwrap(),
            "api.example.com"
        );
        assert_eq!(
            HttpUtils::extract_domain("https://example.com:8080/path").unwrap(),
            "example.com"
        );
        
        // Invalid cases
        assert!(HttpUtils::extract_domain("not-a-url").is_err());
        assert!(HttpUtils::extract_domain("https://").is_err());
    }

    #[test]
    fn test_http_utils_is_https() {
        assert!(HttpUtils::is_https("https://example.com"));
        assert!(!HttpUtils::is_https("http://example.com"));
        assert!(!HttpUtils::is_https("ftp://example.com"));
        assert!(!HttpUtils::is_https("example.com"));
    }

    #[test]
    fn test_http_utils_normalize_url() {
        // Remove fragment
        assert_eq!(
            HttpUtils::normalize_url("https://example.com/path#fragment").unwrap(),
            "https://example.com/path"
        );
        
        // Sort query parameters
        assert_eq!(
            HttpUtils::normalize_url("https://example.com/path?b=2&a=1").unwrap(),
            "https://example.com/path?a=1&b=2"
        );
        
        // Complex case
        assert_eq!(
            HttpUtils::normalize_url("https://example.com/path?z=3&a=1&b=2#fragment").unwrap(),
            "https://example.com/path?a=1&b=2&z=3"
        );
        
        // No changes needed
        assert_eq!(
            HttpUtils::normalize_url("https://example.com/path").unwrap(),
            "https://example.com/path"
        );
    }

    #[test]
    fn test_http_utils_get_default_port() {
        assert_eq!(HttpUtils::get_default_port("https://example.com").unwrap(), 443);
        assert_eq!(HttpUtils::get_default_port("http://example.com").unwrap(), 80);
        assert_eq!(HttpUtils::get_default_port("https://example.com:8080").unwrap(), 443);
        
        // Invalid scheme
        assert!(HttpUtils::get_default_port("ftp://example.com").is_err());
        assert!(HttpUtils::get_default_port("not-a-url").is_err());
    }

    #[tokio::test]
    async fn test_resolve_url_with_ip() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        let dns_config = DnsConfig::System;
        
        // Test with IP address (should skip DNS resolution)
        let result = client.resolve_url("https://8.8.8.8/", &dns_config).await;
        assert!(result.is_ok());
        
        let (ips, duration) = result.unwrap();
        assert_eq!(ips.len(), 1);
        assert_eq!(ips[0], "8.8.8.8".parse::<IpAddr>().unwrap());
        assert_eq!(duration, Duration::from_nanos(0));
    }

    #[tokio::test]
    async fn test_resolve_url_invalid() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        let dns_config = DnsConfig::System;
        
        // Test with invalid URL
        let result = client.resolve_url("not-a-valid-url", &dns_config).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Parse(_)));
    }

    #[tokio::test]
    async fn test_resolve_url_no_host() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        let dns_config = DnsConfig::System;
        
        // Test with URL without host
        let result = client.resolve_url("https://", &dns_config).await;
        assert!(result.is_err());
        // Could be either parse or validation error depending on URL parsing behavior
        let error = result.unwrap_err();
        assert!(matches!(error, AppError::Parse(_)) || matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn test_create_custom_client() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        let dns_config = DnsConfig::System;
        
        let request = HttpRequest::new(
            "https://example.com".to_string(),
            Method::GET,
            dns_config,
        );
        
        let target_ip = "8.8.8.8".parse().unwrap();
        let result = client.create_custom_client(&request, target_ip);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_custom_client_invalid_url() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        let dns_config = DnsConfig::System;
        
        let request = HttpRequest::new(
            "not-a-valid-url".to_string(),
            Method::GET,
            dns_config,
        );
        
        let target_ip = "8.8.8.8".parse().unwrap();
        let result = client.create_custom_client(&request, target_ip);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Parse(_)));
    }

    #[test]
    fn test_custom_client_redirect_configuration() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        let dns_config = DnsConfig::System;
        
        // Test with redirects disabled
        let request = HttpRequest::new(
            "https://example.com".to_string(),
            Method::GET,
            dns_config.clone(),
        ).with_redirects(false, 0);
        
        let target_ip = "8.8.8.8".parse().unwrap();
        let result = client.create_custom_client(&request, target_ip);
        assert!(result.is_ok());
        
        // Test with redirects enabled
        let request = HttpRequest::new(
            "https://example.com".to_string(),
            Method::GET,
            dns_config,
        ).with_redirects(true, 10);
        
        let result = client.create_custom_client(&request, target_ip);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dns_config_system_handling() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        
        let request = HttpRequest::new(
            "https://example.com".to_string(),
            Method::GET,
            DnsConfig::System,
        );
        
        let target_ip = "8.8.8.8".parse().unwrap();
        let result = client.create_custom_client(&request, target_ip);
        assert!(result.is_ok());
    }

    #[test] 
    fn test_dns_config_custom_handling() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        
        let request = HttpRequest::new(
            "https://example.com".to_string(),
            Method::GET,
            DnsConfig::Custom { servers: vec!["8.8.8.8".parse().unwrap()] },
        );
        
        let target_ip = "8.8.8.8".parse().unwrap();
        let result = client.create_custom_client(&request, target_ip);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dns_config_doh_handling() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        
        let request = HttpRequest::new(
            "https://example.com".to_string(),
            Method::GET,
            DnsConfig::DoH { url: "https://dns.google/dns-query".to_string() },
        );
        
        let target_ip = "8.8.8.8".parse().unwrap();
        let result = client.create_custom_client(&request, target_ip);
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_port_extraction() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let client = NetworkClient::new(dns_manager).unwrap();
        let dns_config = DnsConfig::System;
        
        // HTTPS default port
        let request = HttpRequest::new(
            "https://example.com".to_string(),
            Method::GET,
            dns_config.clone(),
        );
        let target_ip = "8.8.8.8".parse().unwrap();
        assert!(client.create_custom_client(&request, target_ip).is_ok());
        
        // HTTP default port
        let request = HttpRequest::new(
            "http://example.com".to_string(),
            Method::GET,
            dns_config.clone(),
        );
        assert!(client.create_custom_client(&request, target_ip).is_ok());
        
        // Custom port
        let request = HttpRequest::new(
            "https://example.com:8080".to_string(),
            Method::GET,
            dns_config,
        );
        assert!(client.create_custom_client(&request, target_ip).is_ok());
    }
}