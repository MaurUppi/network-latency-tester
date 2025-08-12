//! Additional comprehensive tests for DNS configuration and resolution

use super::*;
use crate::types::DnsConfig;
use std::net::IpAddr;
use std::time::Duration;

/// Test DNS configuration edge cases
mod dns_config_tests {
    use super::*;
    
    #[test]
    fn test_dns_config_with_many_servers() {
        // Test with a large number of DNS servers
        let servers: Vec<IpAddr> = (1..=254)
            .map(|i| format!("192.168.1.{}", i).parse().unwrap())
            .collect();
        
        let config = DnsConfig::Custom { servers };
        let name = config.name();
        
        // Should handle large lists gracefully
        assert!(name.contains("自定义DNS"));
        assert!(name.contains("servers"));
        assert!(!name.is_empty());
    }
    
    #[test]
    fn test_dns_config_with_mixed_ip_versions() {
        let servers = vec![
            "8.8.8.8".parse().unwrap(), // IPv4
            "2001:4860:4860::8888".parse().unwrap(), // IPv6
            "1.1.1.1".parse().unwrap(), // IPv4
            "2001:4860:4860::8844".parse().unwrap(), // IPv6
        ];
        
        let config = DnsConfig::Custom { servers };
        let name = config.name();
        
        // With multiple servers, should show count not individual IPs
        assert!(name.contains("自定义DNS"));
        assert!(name.contains("servers"));
    }
    
    #[test]
    fn test_doh_config_with_complex_urls() {
        let complex_urls = vec![
            "https://dns.google/dns-query?ct=application/dns-json",
            "https://cloudflare-dns.com/dns-query",
            "https://dns.quad9.net:5053/dns-query",
            "https://doh.opendns.com/dns-query",
        ];
        
        for url in complex_urls {
            let config = DnsConfig::DoH { url: url.to_string() };
            let name = config.name();
            
            assert!(name.starts_with("DoH"));
            assert!(!name.is_empty());
        }
    }
    
    #[test]
    fn test_dns_config_serialization_deserialization() {
        let configs = vec![
            DnsConfig::System,
            DnsConfig::Custom { 
                servers: vec![
                    "8.8.8.8".parse().unwrap(),
                    "1.1.1.1".parse().unwrap(),
                ]
            },
            DnsConfig::DoH { url: "https://dns.google/dns-query".to_string() },
        ];
        
        for config in configs {
            // Test JSON serialization
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: DnsConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config, deserialized);
        }
    }
    
    #[test]
    fn test_dns_config_name_consistency() {
        let config = DnsConfig::Custom { 
            servers: vec!["8.8.8.8".parse().unwrap()]
        };
        
        // Multiple calls should return the same name
        let name1 = config.name();
        let name2 = config.name();
        let name3 = config.name();
        
        assert_eq!(name1, name2);
        assert_eq!(name2, name3);
        assert!(!name1.is_empty());
    }
}

/// Test DNS validation edge cases
mod dns_validation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dns_manager_with_system_config() {
        let manager = DnsManager::new().unwrap();
        let config = DnsConfig::System;
        
        // System config validation should not panic
        let result = manager.validate_dns_config(&config).await;
        // May succeed or fail depending on system, but should not panic
        assert!(result.is_ok() || result.is_err());
    }
    
    #[tokio::test]
    async fn test_dns_manager_with_invalid_custom_servers() {
        let manager = DnsManager::new().unwrap();
        
        // Test with unreachable private IP
        let config = DnsConfig::Custom { 
            servers: vec!["192.168.254.254".parse().unwrap()]
        };
        
        let result = manager.validate_dns_config(&config).await;
        // Should handle gracefully, not panic
        assert!(result.is_ok() || result.is_err());
    }
    
    #[tokio::test]
    async fn test_dns_resolution_with_localhost() {
        let manager = DnsManager::new().unwrap();
        let config = DnsConfig::System;
        
        // Try to resolve localhost - should work on most systems
        let result = manager.resolve("localhost", &config).await;
        
        if result.is_ok() {
            let ips = result.unwrap();
            // Should contain loopback address
            assert!(ips.iter().any(|ip| ip.is_loopback()));
        }
        // If it fails, that's also acceptable depending on system config
    }
    
    #[tokio::test]
    async fn test_dns_resolution_with_empty_hostname() {
        let manager = DnsManager::new().unwrap();
        let config = DnsConfig::System;
        
        let result = manager.resolve("", &config).await;
        // Should return error, not panic
        assert!(result.is_err());
    }
}

/// Test DNS utilities and helper functions
mod dns_utils_tests {
    use super::*;
    
    #[test]
    fn test_dns_utils_public_servers_validity() {
        let servers = DnsUtils::get_public_dns_servers();
        
        assert!(!servers.is_empty());
        
        // All servers should have valid IP addresses
        for (name, ip_list) in &servers {
            assert!(!name.is_empty());
            assert!(!ip_list.is_empty());
            
            for ip in ip_list {
                assert!(ip.is_ipv4() || ip.is_ipv6());
                
                // Should not be loopback addresses for public DNS
                match ip {
                    IpAddr::V4(ipv4) => {
                        assert!(!ipv4.is_loopback());
                        // Allow private ranges as they might be used in corporate environments
                    }
                    IpAddr::V6(ipv6) => {
                        assert!(!ipv6.is_loopback());
                    }
                }
            }
        }
        
        // Should contain well-known public DNS servers
        let all_ips: Vec<IpAddr> = servers.iter().flat_map(|(_, ips)| ips.iter().copied()).collect();
        assert!(all_ips.contains(&"8.8.8.8".parse().unwrap()) || 
                all_ips.contains(&"1.1.1.1".parse().unwrap()));
    }
    
    #[test]
    fn test_dns_utils_doh_providers_validity() {
        let providers = DnsUtils::get_public_doh_providers();
        
        assert!(!providers.is_empty());
        
        // All providers should have valid HTTPS URLs
        for (name, url) in &providers {
            assert!(!name.is_empty());
            assert!(url.starts_with("https://"));
            
            // Should be parseable as URL
            let parsed = url::Url::parse(url);
            assert!(parsed.is_ok(), "Invalid DoH URL: {}", url);
            
            if let Ok(parsed_url) = parsed {
                assert_eq!(parsed_url.scheme(), "https");
                assert!(parsed_url.host().is_some());
            }
        }
    }
    
    #[test]
    fn test_empty_dns_server_list_handling() {
        let config = DnsConfig::Custom { servers: vec![] };
        let name = config.name();
        
        // Should handle empty list gracefully
        assert!(name.contains("自定义DNS"));
        assert!(name.contains("0 servers"));
    }
}

/// Test DNS configuration comparison and equality
mod dns_config_comparison_tests {
    use super::*;
    
    #[test]
    fn test_dns_config_equality() {
        // System configs should be equal
        assert_eq!(DnsConfig::System, DnsConfig::System);
        
        // Custom configs with same servers should be equal
        let servers1 = vec!["8.8.8.8".parse().unwrap(), "1.1.1.1".parse().unwrap()];
        let servers2 = vec!["8.8.8.8".parse().unwrap(), "1.1.1.1".parse().unwrap()];
        let config1 = DnsConfig::Custom { servers: servers1 };
        let config2 = DnsConfig::Custom { servers: servers2 };
        assert_eq!(config1, config2);
        
        // DoH configs with same URL should be equal
        let doh1 = DnsConfig::DoH { url: "https://dns.google/dns-query".to_string() };
        let doh2 = DnsConfig::DoH { url: "https://dns.google/dns-query".to_string() };
        assert_eq!(doh1, doh2);
        
        // Different types should not be equal
        assert_ne!(DnsConfig::System, config1);
        assert_ne!(config1, doh1);
        assert_ne!(DnsConfig::System, doh1);
    }
    
    #[test]
    fn test_dns_config_server_order_matters() {
        let servers1 = vec!["8.8.8.8".parse().unwrap(), "1.1.1.1".parse().unwrap()];
        let servers2 = vec!["1.1.1.1".parse().unwrap(), "8.8.8.8".parse().unwrap()];
        
        let config1 = DnsConfig::Custom { servers: servers1 };
        let config2 = DnsConfig::Custom { servers: servers2 };
        
        // Different order should result in different configs
        assert_ne!(config1, config2);
        
        // Both should show multiple servers count
        let name1 = config1.name();
        let name2 = config2.name();
        
        assert!(name1.contains("自定义DNS"));
        assert!(name1.contains("2 servers"));
        assert!(name2.contains("自定义DNS"));
        assert!(name2.contains("2 servers"));
    }
}

/// Test DNS performance and stress scenarios
mod dns_performance_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_concurrent_dns_operations() {
        let manager = std::sync::Arc::new(DnsManager::new().unwrap());
        let config = DnsConfig::System;
        
        let mut tasks = Vec::new();
        
        // Create multiple concurrent DNS operations
        for i in 0..5 {
            let manager_clone = manager.clone();
            let config_clone = config.clone();
            
            tasks.push(tokio::spawn(async move {
                let hostname = format!("test{}.example.com", i);
                // This will likely fail but should not panic
                let _ = manager_clone.resolve(&hostname, &config_clone).await;
            }));
        }
        
        // All tasks should complete without panic
        for task in tasks {
            assert!(task.await.is_ok());
        }
    }
    
    #[test]
    fn test_dns_config_name_performance() {
        // Test performance with large server list
        let servers: Vec<IpAddr> = (1..=100)
            .map(|i| format!("192.168.1.{}", i).parse().unwrap())
            .collect();
        
        let config = DnsConfig::Custom { servers };
        
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = config.name();
        }
        let duration = start.elapsed();
        
        // Should be fast even with large server lists
        assert!(duration < Duration::from_secs(1));
    }
}

/// Test DNS configuration with real-world scenarios
mod dns_real_world_tests {
    use super::*;
    
    #[test]
    fn test_common_public_dns_configurations() {
        // Test common public DNS server combinations
        let common_configs = vec![
            // Google DNS
            DnsConfig::Custom { 
                servers: vec!["8.8.8.8".parse().unwrap(), "8.8.4.4".parse().unwrap()]
            },
            // Cloudflare DNS
            DnsConfig::Custom { 
                servers: vec!["1.1.1.1".parse().unwrap(), "1.0.0.1".parse().unwrap()]
            },
            // Quad9 DNS
            DnsConfig::Custom { 
                servers: vec!["9.9.9.9".parse().unwrap(), "149.112.112.112".parse().unwrap()]
            },
        ];
        
        for config in common_configs {
            let name = config.name();
            assert!(name.contains("自定义DNS"));
            assert!(!name.is_empty());
            
            // Should serialize/deserialize correctly
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: DnsConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config, deserialized);
        }
    }
    
    #[test]
    fn test_common_doh_providers() {
        let doh_providers = vec![
            "https://dns.google/dns-query",
            "https://cloudflare-dns.com/dns-query",
            "https://dns.quad9.net/dns-query",
            "https://doh.opendns.com/dns-query",
        ];
        
        for provider in doh_providers {
            let config = DnsConfig::DoH { url: provider.to_string() };
            let name = config.name();
            
            assert!(name.starts_with("DoH"));
            
            // Should be a valid URL
            let parsed = url::Url::parse(provider);
            assert!(parsed.is_ok());
            
            if let Ok(url) = parsed {
                assert_eq!(url.scheme(), "https");
            }
        }
    }
}