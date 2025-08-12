//! Configuration data model and validation

use crate::types::{DnsConfig, Result, AppError};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;
use std::str::FromStr;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Target URLs to test
    #[serde(default = "default_target_urls")]
    pub target_urls: Vec<String>,
    
    /// DNS server IP addresses for custom DNS testing
    #[serde(default = "default_dns_servers")]
    pub dns_servers: Vec<String>,
    
    /// DNS-over-HTTPS provider URLs
    #[serde(default = "default_doh_providers")]
    pub doh_providers: Vec<String>,
    
    /// Number of test iterations per configuration
    #[serde(default = "default_test_count")]
    pub test_count: u32,
    
    /// Request timeout duration
    #[serde(default = "default_timeout_secs")]
    pub timeout_seconds: u64,
    
    /// Enable colored terminal output
    #[serde(default = "default_enable_color")]
    pub enable_color: bool,
    
    /// Enable verbose output
    #[serde(default)]
    pub verbose: bool,
    
    /// Enable debug output
    #[serde(default)]
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_urls: default_target_urls(),
            dns_servers: default_dns_servers(),
            doh_providers: default_doh_providers(),
            test_count: default_test_count(),
            timeout_seconds: default_timeout_secs(),
            enable_color: default_enable_color(),
            verbose: false,
            debug: false,
        }
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }
    
    /// Validate the configuration and return any errors
    pub fn validate(&self) -> Result<()> {
        // Validate target URLs
        for url in &self.target_urls {
            if url.is_empty() {
                return Err(AppError::config("Target URL cannot be empty"));
            }
            
            // Basic URL format validation
            if let Err(e) = url::Url::parse(url) {
                return Err(AppError::config(format!("Invalid target URL '{}': {}", url, e)));
            }
        }
        
        // Validate DNS servers
        for dns_server in &self.dns_servers {
            if dns_server.is_empty() {
                return Err(AppError::config("DNS server cannot be empty"));
            }
            
            if IpAddr::from_str(dns_server).is_err() {
                return Err(AppError::config(format!("Invalid DNS server IP address: {}", dns_server)));
            }
        }
        
        // Validate DoH providers
        for doh_url in &self.doh_providers {
            if doh_url.is_empty() {
                return Err(AppError::config("DoH provider URL cannot be empty"));
            }
            
            match url::Url::parse(doh_url) {
                Ok(parsed) => {
                    if parsed.scheme() != "https" {
                        return Err(AppError::config(format!("DoH URL must use HTTPS: {}", doh_url)));
                    }
                }
                Err(e) => {
                    return Err(AppError::config(format!("Invalid DoH provider URL '{}': {}", doh_url, e)));
                }
            }
        }
        
        // Validate numeric parameters
        if self.test_count == 0 {
            return Err(AppError::config("Test count must be greater than 0"));
        }
        
        if self.test_count > 100 {
            return Err(AppError::config("Test count cannot exceed 100"));
        }
        
        if self.timeout_seconds == 0 {
            return Err(AppError::config("Timeout must be greater than 0"));
        }
        
        if self.timeout_seconds > 300 {
            return Err(AppError::config("Timeout cannot exceed 300 seconds"));
        }
        
        Ok(())
    }
    
    /// Create DNS configurations from the config settings
    pub fn create_dns_configs(&self) -> Result<Vec<DnsConfig>> {
        let mut configs = Vec::new();
        
        // Always include system default
        configs.push(DnsConfig::System);
        
        // Add custom DNS servers
        for dns_server in &self.dns_servers {
            match IpAddr::from_str(dns_server) {
                Ok(ip) => configs.push(DnsConfig::Custom { servers: vec![ip] }),
                Err(e) => return Err(AppError::dns_resolution(format!("Failed to parse DNS server {}: {}", dns_server, e))),
            }
        }
        
        // Add DoH providers
        for doh_url in &self.doh_providers {
            configs.push(DnsConfig::DoH { url: doh_url.clone() });
        }
        
        Ok(configs)
    }
    
    /// Merge environment variables into this configuration
    pub fn merge_from_env(&mut self) -> Result<()> {
        if let Ok(target_urls) = std::env::var("TARGET_URLS") {
            self.target_urls = target_urls
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        if let Ok(dns_servers) = std::env::var("DNS_SERVERS") {
            self.dns_servers = dns_servers
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        if let Ok(doh_providers) = std::env::var("DOH_PROVIDERS") {
            self.doh_providers = doh_providers
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        if let Ok(test_count) = std::env::var("TEST_COUNT") {
            self.test_count = test_count.parse()
                .map_err(|e| AppError::config(format!("Invalid TEST_COUNT value '{}': {}", test_count, e)))?;
        }
        
        if let Ok(timeout) = std::env::var("TIMEOUT_SECONDS") {
            self.timeout_seconds = timeout.parse()
                .map_err(|e| AppError::config(format!("Invalid TIMEOUT_SECONDS value '{}': {}", timeout, e)))?;
        }
        
        if let Ok(enable_color) = std::env::var("ENABLE_COLOR") {
            self.enable_color = enable_color.parse()
                .map_err(|e| AppError::config(format!("Invalid ENABLE_COLOR value '{}': {}", enable_color, e)))?;
        }
        
        Ok(())
    }
}

// Default value functions for serde
fn default_target_urls() -> Vec<String> {
    crate::defaults::DEFAULT_TARGET_URLS
        .iter()
        .map(|&s| s.to_string())
        .collect()
}

fn default_dns_servers() -> Vec<String> {
    crate::defaults::DEFAULT_DNS_SERVERS
        .iter()
        .map(|&s| s.to_string())
        .collect()
}

fn default_doh_providers() -> Vec<String> {
    crate::defaults::DEFAULT_DOH_PROVIDERS
        .iter()
        .map(|&s| s.to_string())
        .collect()
}

fn default_test_count() -> u32 {
    crate::defaults::DEFAULT_TEST_COUNT
}

fn default_timeout_secs() -> u64 {
    crate::defaults::DEFAULT_TIMEOUT.as_secs()
}

fn default_enable_color() -> bool {
    crate::defaults::DEFAULT_ENABLE_COLOR
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_is_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_empty_target_url_invalid() {
        let mut config = Config::default();
        config.target_urls = vec!["".to_string()];
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_invalid_target_url_format() {
        let mut config = Config::default();
        config.target_urls = vec!["not-a-url".to_string()];
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_invalid_dns_server_ip() {
        let mut config = Config::default();
        config.dns_servers = vec!["not-an-ip".to_string()];
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_non_https_doh_url_invalid() {
        let mut config = Config::default();
        config.doh_providers = vec!["http://example.com/dns-query".to_string()];
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_zero_test_count_invalid() {
        let mut config = Config::default();
        config.test_count = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_create_dns_configs() {
        let mut config = Config::default();
        config.dns_servers = vec!["8.8.8.8".to_string()];
        config.doh_providers = vec!["https://cloudflare-dns.com/dns-query".to_string()];
        
        let dns_configs = config.create_dns_configs().unwrap();
        assert_eq!(dns_configs.len(), 3); // System + 1 custom + 1 DoH
        
        assert_eq!(dns_configs[0], DnsConfig::System);
        assert!(matches!(dns_configs[1], DnsConfig::Custom { .. }));
        assert!(matches!(dns_configs[2], DnsConfig::DoH { .. }));
    }
}