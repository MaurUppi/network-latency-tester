//! Environment variable handling and .env file management

use crate::error::{AppError, Result};
use std::path::Path;

/// Environment variable configuration manager
pub struct EnvManager;

impl EnvManager {
    /// Load .env file if it exists
    pub fn load_env_file(debug: bool) -> Result<()> {
        // Try to load .env from current directory
        if Path::new(".env").exists() {
            dotenv::from_filename(".env")
                .map_err(|e| AppError::config(format!("Failed to load .env file: {}", e)))?;
            
            if debug {
                println!("Loaded configuration from .env file");
            }
        } else if debug {
            println!("No .env file found, using defaults and CLI arguments");
        }

        Ok(())
    }

    /// Create example .env file content
    pub fn create_example_env_content() -> String {
        r#"# Network Latency Tester Configuration
# 
# This file contains environment variables that can be used to configure
# the network latency tester. Values specified here will be used as defaults,
# but can be overridden by command-line arguments.

# Target URLs to test (comma-separated)
# TARGET_URLS=https://as.target,https://example.com

# Custom DNS servers to test (comma-separated IP addresses)
# DNS_SERVERS=8.8.8.8,1.1.1.1,208.67.222.222

# DNS-over-HTTPS providers (comma-separated URLs)
# DOH_PROVIDERS=https://cloudflare-dns.com/dns-query,https://dns.google/dns-query

# Number of test iterations per DNS configuration
# TEST_COUNT=5

# Request timeout in seconds
# TIMEOUT_SECONDS=10

# Enable colored output (true/false)
# ENABLE_COLOR=true

# Example configurations for different scenarios:
#
# Testing multiple targets:
# TARGET_URLS=https://as.target,https://api.openai.com,https://www.google.com
#
# Testing with popular public DNS servers:
# DNS_SERVERS=8.8.8.8,1.1.1.1,208.67.222.222,9.9.9.9
#
# Using alternative DoH providers:
# DOH_PROVIDERS=https://cloudflare-dns.com/dns-query,https://dns.google/dns-query
#
# High-frequency testing:
# TEST_COUNT=10
# TIMEOUT_SECONDS=5
"#.to_string()
    }

    /// Save example .env file to disk
    pub fn save_example_env_file(path: &Path) -> Result<()> {
        use std::fs;
        
        let content = Self::create_example_env_content();
        fs::write(path, content)
            .map_err(|e| AppError::config(format!("Failed to write example .env file: {}", e)))?;
        
        Ok(())
    }

    /// Validate environment variable format before parsing
    pub fn validate_env_var(key: &str, value: &str) -> Result<()> {
        match key {
            "TARGET_URLS" => {
                for url in value.split(',') {
                    let url = url.trim();
                    if !url.is_empty() {
                        url::Url::parse(url)
                            .map_err(|e| AppError::config(format!("Invalid TARGET_URLS entry '{}': {}", url, e)))?;
                    }
                }
            }
            "DNS_SERVERS" => {
                for server in value.split(',') {
                    let server = server.trim();
                    if !server.is_empty() {
                        server.parse::<std::net::IpAddr>()
                            .map_err(|e| AppError::config(format!("Invalid DNS_SERVERS entry '{}': {}", server, e)))?;
                    }
                }
            }
            "DOH_PROVIDERS" => {
                for provider in value.split(',') {
                    let provider = provider.trim();
                    if !provider.is_empty() {
                        let parsed = url::Url::parse(provider)
                            .map_err(|e| AppError::config(format!("Invalid DOH_PROVIDERS entry '{}': {}", provider, e)))?;
                        if parsed.scheme() != "https" {
                            return Err(AppError::config(format!("DoH provider must use HTTPS: {}", provider)));
                        }
                    }
                }
            }
            "TEST_COUNT" => {
                let count: u32 = value.parse()
                    .map_err(|e| AppError::config(format!("Invalid TEST_COUNT value '{}': {}", value, e)))?;
                if count == 0 || count > 100 {
                    return Err(AppError::config(format!("TEST_COUNT must be between 1 and 100, got: {}", count)));
                }
            }
            "TIMEOUT_SECONDS" => {
                let timeout: u64 = value.parse()
                    .map_err(|e| AppError::config(format!("Invalid TIMEOUT_SECONDS value '{}': {}", value, e)))?;
                if timeout == 0 || timeout > 300 {
                    return Err(AppError::config(format!("TIMEOUT_SECONDS must be between 1 and 300, got: {}", timeout)));
                }
            }
            "ENABLE_COLOR" => {
                value.parse::<bool>()
                    .map_err(|e| AppError::config(format!("Invalid ENABLE_COLOR value '{}': {}", value, e)))?;
            }
            _ => {
                // Unknown environment variable, ignore
            }
        }
        
        Ok(())
    }

    /// Get list of all supported environment variables with descriptions
    pub fn get_supported_env_vars() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("TARGET_URLS", "Comma-separated list of URLs to test", "https://example.com,https://google.com"),
            ("DNS_SERVERS", "Comma-separated list of DNS server IPs", "8.8.8.8,1.1.1.1,208.67.222.222"),
            ("DOH_PROVIDERS", "Comma-separated list of DoH URLs", "https://cloudflare-dns.com/dns-query"),
            ("TEST_COUNT", "Number of test iterations (1-100)", "5"),
            ("TIMEOUT_SECONDS", "Request timeout in seconds (1-300)", "10"),
            ("ENABLE_COLOR", "Enable colored output", "true"),
        ]
    }

    /// Display environment variable help
    pub fn display_env_help() -> String {
        let mut help = String::new();
        help.push_str("Supported Environment Variables:\n\n");
        
        for (var, description, example) in Self::get_supported_env_vars() {
            help.push_str(&format!("  {:<18} {}\n", var, description));
            help.push_str(&format!("  {:<18} Example: {}\n\n", "", example));
        }
        
        help.push_str("Configuration Priority (highest to lowest):\n");
        help.push_str("  1. Command-line arguments\n");
        help.push_str("  2. Environment variables\n");
        help.push_str("  3. .env file values\n");
        help.push_str("  4. Default values\n");
        
        help
    }

    /// Validate all currently set environment variables
    pub fn validate_current_env() -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        
        for (var_name, _, _) in Self::get_supported_env_vars() {
            if let Ok(value) = std::env::var(var_name) {
                if let Err(e) = Self::validate_env_var(var_name, &value) {
                    warnings.push(format!("Warning: {}", e));
                }
            }
        }
        
        Ok(warnings)
    }

    /// Check if .env file exists and validate its contents
    pub fn check_env_file() -> Result<Option<Vec<String>>> {
        if !Path::new(".env").exists() {
            return Ok(None);
        }

        // Load the .env file temporarily to validate
        let content = std::fs::read_to_string(".env")
            .map_err(|e| AppError::config(format!("Failed to read .env file: {}", e)))?;

        let mut warnings = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                if let Err(e) = Self::validate_env_var(key, value) {
                    warnings.push(format!("Line '{}': {}", line, e));
                }
            }
        }
        
        Ok(Some(warnings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_env_manager_create_example_content() {
        let content = EnvManager::create_example_env_content();
        
        assert!(content.contains("TARGET_URLS="));
        assert!(content.contains("DNS_SERVERS="));
        assert!(content.contains("DOH_PROVIDERS="));
        assert!(content.contains("TEST_COUNT="));
        assert!(content.contains("TIMEOUT_SECONDS="));
        assert!(content.contains("ENABLE_COLOR="));
    }

    #[test]
    fn test_env_manager_save_example_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = EnvManager::save_example_env_file(temp_file.path());
        
        assert!(result.is_ok());
        
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Network Latency Tester Configuration"));
    }

    #[test]
    fn test_env_manager_validate_env_var() {
        // Valid cases
        assert!(EnvManager::validate_env_var("TARGET_URLS", "https://example.com,https://google.com").is_ok());
        assert!(EnvManager::validate_env_var("DNS_SERVERS", "8.8.8.8,1.1.1.1").is_ok());
        assert!(EnvManager::validate_env_var("DOH_PROVIDERS", "https://cloudflare-dns.com/dns-query").is_ok());
        assert!(EnvManager::validate_env_var("TEST_COUNT", "5").is_ok());
        assert!(EnvManager::validate_env_var("TIMEOUT_SECONDS", "10").is_ok());
        assert!(EnvManager::validate_env_var("ENABLE_COLOR", "true").is_ok());

        // Invalid cases
        assert!(EnvManager::validate_env_var("TARGET_URLS", "not-a-url").is_err());
        assert!(EnvManager::validate_env_var("DNS_SERVERS", "not-an-ip").is_err());
        assert!(EnvManager::validate_env_var("DOH_PROVIDERS", "http://insecure.com/dns-query").is_err());
        assert!(EnvManager::validate_env_var("TEST_COUNT", "0").is_err());
        assert!(EnvManager::validate_env_var("TEST_COUNT", "101").is_err());
        assert!(EnvManager::validate_env_var("TIMEOUT_SECONDS", "0").is_err());
        assert!(EnvManager::validate_env_var("TIMEOUT_SECONDS", "301").is_err());
        assert!(EnvManager::validate_env_var("ENABLE_COLOR", "maybe").is_err());
    }

    #[test]
    fn test_get_supported_env_vars() {
        let vars = EnvManager::get_supported_env_vars();
        
        assert_eq!(vars.len(), 6);
        assert!(vars.iter().any(|(name, _, _)| *name == "TARGET_URLS"));
        assert!(vars.iter().any(|(name, _, _)| *name == "DNS_SERVERS"));
        assert!(vars.iter().any(|(name, _, _)| *name == "DOH_PROVIDERS"));
        assert!(vars.iter().any(|(name, _, _)| *name == "TEST_COUNT"));
        assert!(vars.iter().any(|(name, _, _)| *name == "TIMEOUT_SECONDS"));
        assert!(vars.iter().any(|(name, _, _)| *name == "ENABLE_COLOR"));
    }

    #[test]
    fn test_display_env_help() {
        let help = EnvManager::display_env_help();
        
        assert!(help.contains("Supported Environment Variables:"));
        assert!(help.contains("TARGET_URLS"));
        assert!(help.contains("DNS_SERVERS"));
        assert!(help.contains("Configuration Priority"));
        assert!(help.contains("Command-line arguments"));
    }

    #[test]
    fn test_validate_current_env_empty() {
        // Clear any potentially set environment variables for this test
        for (var_name, _, _) in EnvManager::get_supported_env_vars() {
            std::env::remove_var(var_name);
        }
        
        let result = EnvManager::validate_current_env();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}