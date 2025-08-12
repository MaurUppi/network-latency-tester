//! User-friendly error messages and troubleshooting system
//!
//! This module provides enhanced error messaging with detailed troubleshooting tips,
//! platform-specific guidance, and contextual help for common issues users encounter.

use super::AppError;
use crate::models::Config;
use std::collections::HashMap;
use colored::Colorize;

/// Enhanced error message with detailed troubleshooting information
#[derive(Debug, Clone)]
pub struct EnhancedErrorMessage {
    /// Primary error message
    pub message: String,
    /// Brief description of the issue
    pub description: String,
    /// Immediate actions user can take
    pub immediate_actions: Vec<String>,
    /// Step-by-step troubleshooting guide
    pub troubleshooting_steps: Vec<TroubleshootingStep>,
    /// Related documentation links or help topics
    pub help_topics: Vec<String>,
    /// Platform-specific considerations
    pub platform_notes: Vec<PlatformNote>,
    /// Example commands or configurations
    pub examples: Vec<String>,
    /// Whether this error commonly occurs
    pub is_common: bool,
    /// Estimated time to resolve
    pub resolution_time: ResolutionTime,
}

/// Individual troubleshooting step
#[derive(Debug, Clone)]
pub struct TroubleshootingStep {
    /// Step number
    pub number: usize,
    /// Step description
    pub description: String,
    /// Commands or actions to perform
    pub actions: Vec<String>,
    /// Expected outcome
    pub expected_outcome: String,
    /// If this step fails, what to do next
    pub failure_next_step: Option<usize>,
}

/// Platform-specific note
#[derive(Debug, Clone)]
pub struct PlatformNote {
    /// Target platforms (e.g., "Windows", "macOS", "Linux")
    pub platforms: Vec<String>,
    /// Platform-specific message
    pub message: String,
    /// Platform-specific actions
    pub actions: Vec<String>,
}

/// Estimated time to resolve issue
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolutionTime {
    /// Quick fix (< 1 minute)
    Quick,
    /// Moderate fix (1-5 minutes)
    Moderate,
    /// Complex fix (5-15 minutes)
    Complex,
    /// Advanced troubleshooting (> 15 minutes)
    Advanced,
}

impl ResolutionTime {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Quick => "< 1 minute",
            Self::Moderate => "1-5 minutes",
            Self::Complex => "5-15 minutes",
            Self::Advanced => "> 15 minutes",
        }
    }
    
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Quick => "🚀",
            Self::Moderate => "⚡",
            Self::Complex => "🔧",
            Self::Advanced => "🛠️",
        }
    }
}

/// Enhanced error message provider
pub struct UserMessageProvider {
    /// Configuration for message customization
    config: UserMessageConfig,
    /// Cache of generated messages
    message_cache: HashMap<String, EnhancedErrorMessage>,
}

/// Configuration for user message generation
#[derive(Debug, Clone)]
pub struct UserMessageConfig {
    /// Use colored output
    pub use_color: bool,
    /// Include platform-specific notes
    pub include_platform_notes: bool,
    /// Show detailed troubleshooting steps
    pub show_detailed_steps: bool,
    /// Current platform
    pub platform: Platform,
    /// User experience level
    pub experience_level: ExperienceLevel,
}

/// Supported platforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOS
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Unknown
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOS => "macOS",
            Self::Linux => "Linux",
            Self::Unknown => "Unknown",
        }
    }
}

/// User experience level for tailored messaging
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExperienceLevel {
    /// New user, needs detailed explanations
    Beginner,
    /// Regular user, needs balanced guidance
    Intermediate,
    /// Expert user, prefers concise information
    Advanced,
}

impl UserMessageProvider {
    /// Create a new user message provider
    pub fn new(config: UserMessageConfig) -> Self {
        Self {
            config,
            message_cache: HashMap::new(),
        }
    }
    
    /// Create provider with default configuration
    pub fn with_defaults() -> Self {
        Self::new(UserMessageConfig {
            use_color: true,
            include_platform_notes: true,
            show_detailed_steps: true,
            platform: Platform::current(),
            experience_level: ExperienceLevel::Intermediate,
        })
    }
    
    /// Create provider from application config
    pub fn from_app_config(app_config: &Config) -> Self {
        let experience_level = if app_config.debug {
            ExperienceLevel::Advanced
        } else if app_config.verbose {
            ExperienceLevel::Intermediate
        } else {
            ExperienceLevel::Beginner
        };
        
        Self::new(UserMessageConfig {
            use_color: app_config.enable_color,
            include_platform_notes: true,
            show_detailed_steps: app_config.verbose || app_config.debug,
            platform: Platform::current(),
            experience_level,
        })
    }
    
    /// Get enhanced error message for an AppError
    pub fn get_enhanced_message(&mut self, error: &AppError) -> EnhancedErrorMessage {
        let error_key = format!("{}:{}", error.category(), error.to_string());
        
        if let Some(cached_message) = self.message_cache.get(&error_key) {
            return cached_message.clone();
        }
        
        let enhanced_message = self.generate_enhanced_message(error);
        self.message_cache.insert(error_key, enhanced_message.clone());
        enhanced_message
    }
    
    /// Generate enhanced message for error
    fn generate_enhanced_message(&self, error: &AppError) -> EnhancedErrorMessage {
        match error {
            AppError::Config(_) => self.generate_config_error_message(error),
            AppError::Network(_) => self.generate_network_error_message(error),
            AppError::DnsResolution(_) => self.generate_dns_error_message(error),
            AppError::HttpRequest(_) => self.generate_http_error_message(error),
            AppError::Timeout(_) => self.generate_timeout_error_message(error),
            AppError::Validation(_) => self.generate_validation_error_message(error),
            AppError::Io(_) => self.generate_io_error_message(error),
            AppError::Parse(_) => self.generate_parse_error_message(error),
            AppError::Auth(_) => self.generate_auth_error_message(error),
            AppError::TestExecution(_) => self.generate_test_execution_error_message(error),
            AppError::Statistics(_) => self.generate_statistics_error_message(error),
            AppError::Internal(_) => self.generate_internal_error_message(error),
        }
    }
    
    /// Generate configuration error message
    fn generate_config_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "Configuration Problem".to_string(),
            description: "There's an issue with your application configuration. This usually means invalid command-line arguments, environment variables, or configuration file values.".to_string(),
            immediate_actions: vec![
                "Check your command-line arguments for typos".to_string(),
                "Verify your .env file format if using one".to_string(),
                "Run with --help to see valid options".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Validate command-line arguments".to_string(),
                    actions: vec![
                        "Run: network-latency-tester --help".to_string(),
                        "Compare your arguments with the help output".to_string(),
                        "Check for required arguments you may have missed".to_string(),
                    ],
                    expected_outcome: "You should see all available options and their formats".to_string(),
                    failure_next_step: Some(2),
                },
                TroubleshootingStep {
                    number: 2,
                    description: "Check environment variables".to_string(),
                    actions: vec![
                        "Review your .env file if using one".to_string(),
                        "Verify environment variable names and values".to_string(),
                        "Try running without environment variables to isolate the issue".to_string(),
                    ],
                    expected_outcome: "Environment variables should be properly formatted".to_string(),
                    failure_next_step: Some(3),
                },
                TroubleshootingStep {
                    number: 3,
                    description: "Test with minimal configuration".to_string(),
                    actions: vec![
                        "Try: network-latency-tester --url https://example.com".to_string(),
                        "If that works, gradually add your other options".to_string(),
                        "This helps identify which specific setting is causing issues".to_string(),
                    ],
                    expected_outcome: "Basic command should work, helping isolate the problem".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "Configuration file format".to_string(),
                "Environment variables".to_string(),
                "Command-line options".to_string(),
            ],
            platform_notes: self.get_config_platform_notes(),
            examples: vec![
                "network-latency-tester --url https://example.com --count 3".to_string(),
                "network-latency-tester --url https://google.com --dns-servers 8.8.8.8,1.1.1.1".to_string(),
                "TARGET_URLS=https://example.com network-latency-tester".to_string(),
            ],
            is_common: true,
            resolution_time: ResolutionTime::Quick,
        }
    }
    
    /// Generate network error message
    fn generate_network_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "Network Connectivity Issue".to_string(),
            description: "Unable to establish network connection. This could be due to internet connectivity, firewall settings, or the target server being unavailable.".to_string(),
            immediate_actions: vec![
                "Check your internet connection".to_string(),
                "Try accessing the URL in a web browser".to_string(),
                "Test with a different URL (e.g., https://google.com)".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Test basic connectivity".to_string(),
                    actions: self.get_network_test_commands(),
                    expected_outcome: "Should confirm internet connectivity".to_string(),
                    failure_next_step: Some(2),
                },
                TroubleshootingStep {
                    number: 2,
                    description: "Check firewall and proxy settings".to_string(),
                    actions: vec![
                        "Temporarily disable firewall (if safe to do so)".to_string(),
                        "Check for corporate proxy settings".to_string(),
                        "Try connecting from a different network".to_string(),
                    ],
                    expected_outcome: "Network access should be restored".to_string(),
                    failure_next_step: Some(3),
                },
                TroubleshootingStep {
                    number: 3,
                    description: "Test with different DNS settings".to_string(),
                    actions: vec![
                        "Try: network-latency-tester --url https://1.1.1.1 --dns-servers 8.8.8.8".to_string(),
                        "Test with: --dns-servers 1.1.1.1,8.8.8.8".to_string(),
                        "Try DoH: --doh-providers https://cloudflare-dns.com/dns-query".to_string(),
                    ],
                    expected_outcome: "Alternative DNS should resolve connectivity".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "DNS configuration".to_string(),
                "Firewall settings".to_string(),
                "Proxy configuration".to_string(),
            ],
            platform_notes: self.get_network_platform_notes(),
            examples: vec![
                "network-latency-tester --url https://8.8.8.8".to_string(),
                "network-latency-tester --url https://example.com --dns-servers 8.8.8.8".to_string(),
            ],
            is_common: true,
            resolution_time: ResolutionTime::Moderate,
        }
    }
    
    /// Generate DNS error message
    fn generate_dns_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "DNS Resolution Failed".to_string(),
            description: "Unable to resolve domain names to IP addresses. This might be due to DNS server issues, incorrect DNS configuration, or domain not existing.".to_string(),
            immediate_actions: vec![
                "Try using public DNS servers (8.8.8.8, 1.1.1.1)".to_string(),
                "Check if the domain exists by visiting it in a browser".to_string(),
                "Test with an IP address instead of domain name".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Test with public DNS servers".to_string(),
                    actions: vec![
                        "Try: --dns-servers 8.8.8.8,1.1.1.1".to_string(),
                        "Try: --dns-servers 208.67.222.222,208.67.220.220".to_string(),
                        "Test with: --doh-providers https://dns.google/dns-query".to_string(),
                    ],
                    expected_outcome: "DNS resolution should work with public servers".to_string(),
                    failure_next_step: Some(2),
                },
                TroubleshootingStep {
                    number: 2,
                    description: "Verify domain exists".to_string(),
                    actions: self.get_dns_verification_commands(),
                    expected_outcome: "Should confirm domain is valid and reachable".to_string(),
                    failure_next_step: Some(3),
                },
                TroubleshootingStep {
                    number: 3,
                    description: "Test with IP addresses".to_string(),
                    actions: vec![
                        "Try: --url https://8.8.8.8".to_string(),
                        "Try: --url https://1.1.1.1".to_string(),
                        "If IPs work but domains don't, it's definitely DNS".to_string(),
                    ],
                    expected_outcome: "IP addresses should work if DNS is the only issue".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "DNS configuration".to_string(),
                "DoH setup".to_string(),
                "DNS troubleshooting".to_string(),
            ],
            platform_notes: self.get_dns_platform_notes(),
            examples: vec![
                "network-latency-tester --url https://example.com --dns-servers 8.8.8.8".to_string(),
                "network-latency-tester --url https://google.com --doh-providers https://dns.google/dns-query".to_string(),
            ],
            is_common: true,
            resolution_time: ResolutionTime::Moderate,
        }
    }
    
    /// Generate HTTP request error message
    fn generate_http_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "HTTP Request Failed".to_string(),
            description: "The HTTP request to the target URL failed. This could be due to server issues, invalid URLs, SSL/TLS problems, or the server blocking requests.".to_string(),
            immediate_actions: vec![
                "Verify the URL is correct and accessible".to_string(),
                "Try the URL in a web browser".to_string(),
                "Check if the server requires authentication".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Verify URL format and accessibility".to_string(),
                    actions: vec![
                        "Check URL starts with https:// or http://".to_string(),
                        "Test the URL in a web browser".to_string(),
                        "Try a known working URL like https://httpbin.org/get".to_string(),
                    ],
                    expected_outcome: "URL should be accessible in browser".to_string(),
                    failure_next_step: Some(2),
                },
                TroubleshootingStep {
                    number: 2,
                    description: "Check for SSL/TLS issues".to_string(),
                    actions: vec![
                        "Try HTTP instead of HTTPS if available".to_string(),
                        "Check certificate validity in browser".to_string(),
                        "Test with: --timeout 30 for slow SSL handshakes".to_string(),
                    ],
                    expected_outcome: "Should identify SSL/certificate issues".to_string(),
                    failure_next_step: Some(3),
                },
                TroubleshootingStep {
                    number: 3,
                    description: "Test server availability".to_string(),
                    actions: vec![
                        "Try from a different network/location".to_string(),
                        "Check if server blocks automated requests".to_string(),
                        "Test with different user agent if possible".to_string(),
                    ],
                    expected_outcome: "Should determine if server is accessible".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "SSL/TLS troubleshooting".to_string(),
                "HTTP status codes".to_string(),
                "URL format".to_string(),
            ],
            platform_notes: vec![],
            examples: vec![
                "network-latency-tester --url https://httpbin.org/get".to_string(),
                "network-latency-tester --url http://example.com --timeout 30".to_string(),
            ],
            is_common: true,
            resolution_time: ResolutionTime::Moderate,
        }
    }
    
    /// Generate timeout error message
    fn generate_timeout_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "Request Timeout".to_string(),
            description: "Requests are taking longer than the configured timeout limit. This usually indicates slow network connectivity, server overload, or configuration issues.".to_string(),
            immediate_actions: vec![
                "Increase timeout with --timeout 30".to_string(),
                "Test with fewer concurrent requests".to_string(),
                "Try a faster/closer server".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Increase timeout duration".to_string(),
                    actions: vec![
                        "Try: --timeout 30 (30 seconds)".to_string(),
                        "For slow connections: --timeout 60".to_string(),
                        "Test different timeout values to find optimal setting".to_string(),
                    ],
                    expected_outcome: "Requests should complete within extended timeout".to_string(),
                    failure_next_step: Some(2),
                },
                TroubleshootingStep {
                    number: 2,
                    description: "Reduce test intensity".to_string(),
                    actions: vec![
                        "Try: --count 1 (single test)".to_string(),
                        "Reduce concurrent operations".to_string(),
                        "Test during off-peak hours".to_string(),
                    ],
                    expected_outcome: "Single requests should complete successfully".to_string(),
                    failure_next_step: Some(3),
                },
                TroubleshootingStep {
                    number: 3,
                    description: "Test network performance".to_string(),
                    actions: vec![
                        "Run speed test on your connection".to_string(),
                        "Try testing a faster/local server".to_string(),
                        "Check for network congestion".to_string(),
                    ],
                    expected_outcome: "Should identify network performance issues".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "Timeout configuration".to_string(),
                "Network performance".to_string(),
                "Concurrent testing".to_string(),
            ],
            platform_notes: vec![],
            examples: vec![
                "network-latency-tester --url https://example.com --timeout 30".to_string(),
                "network-latency-tester --url https://example.com --count 1 --timeout 60".to_string(),
            ],
            is_common: true,
            resolution_time: ResolutionTime::Quick,
        }
    }
    
    /// Generate validation error message
    fn generate_validation_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "Invalid Input".to_string(),
            description: "One or more of your input values is invalid. This includes URLs, IP addresses, timeout values, or other configuration parameters.".to_string(),
            immediate_actions: vec![
                "Check URL format (must start with http:// or https://)".to_string(),
                "Verify IP addresses are valid (e.g., 8.8.8.8)".to_string(),
                "Ensure numeric values are within valid ranges".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Validate URL format".to_string(),
                    actions: vec![
                        "URLs must start with http:// or https://".to_string(),
                        "Examples: https://example.com, http://192.168.1.1".to_string(),
                        "Avoid spaces and special characters in URLs".to_string(),
                    ],
                    expected_outcome: "URLs should be properly formatted".to_string(),
                    failure_next_step: Some(2),
                },
                TroubleshootingStep {
                    number: 2,
                    description: "Check IP address format".to_string(),
                    actions: vec![
                        "IP addresses: 4 numbers separated by dots".to_string(),
                        "Valid range: 0-255 for each number".to_string(),
                        "Examples: 8.8.8.8, 192.168.1.1, 127.0.0.1".to_string(),
                    ],
                    expected_outcome: "IP addresses should be valid IPv4 format".to_string(),
                    failure_next_step: Some(3),
                },
                TroubleshootingStep {
                    number: 3,
                    description: "Verify numeric parameters".to_string(),
                    actions: vec![
                        "Count: must be 1-100".to_string(),
                        "Timeout: must be 1-300 seconds".to_string(),
                        "Check for typos in numbers".to_string(),
                    ],
                    expected_outcome: "All numeric values should be within valid ranges".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "URL format".to_string(),
                "IP address format".to_string(),
                "Parameter ranges".to_string(),
            ],
            platform_notes: vec![],
            examples: vec![
                "Valid URL: https://example.com".to_string(),
                "Valid IP: 8.8.8.8".to_string(),
                "Valid command: --url https://google.com --count 5 --timeout 10".to_string(),
            ],
            is_common: true,
            resolution_time: ResolutionTime::Quick,
        }
    }
    
    /// Generate I/O error message
    fn generate_io_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "File Operation Failed".to_string(),
            description: "Unable to read or write a file. This could be due to permissions, disk space, or the file not existing.".to_string(),
            immediate_actions: vec![
                "Check file permissions".to_string(),
                "Verify file path is correct".to_string(),
                "Ensure sufficient disk space".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Check file permissions and existence".to_string(),
                    actions: self.get_file_check_commands(),
                    expected_outcome: "Should show file status and permissions".to_string(),
                    failure_next_step: Some(2),
                },
                TroubleshootingStep {
                    number: 2,
                    description: "Verify disk space and directory access".to_string(),
                    actions: self.get_disk_space_commands(),
                    expected_outcome: "Should confirm adequate disk space".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "File permissions".to_string(),
                "Disk space".to_string(),
            ],
            platform_notes: self.get_file_platform_notes(),
            examples: vec![],
            is_common: false,
            resolution_time: ResolutionTime::Quick,
        }
    }
    
    /// Generate parsing error message
    fn generate_parse_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "Data Parsing Failed".to_string(),
            description: "Unable to parse the provided data format. This usually means invalid JSON, malformed URLs, or incorrect number formats.".to_string(),
            immediate_actions: vec![
                "Check data format for typos".to_string(),
                "Validate JSON syntax if applicable".to_string(),
                "Ensure numbers are properly formatted".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Validate data format".to_string(),
                    actions: vec![
                        "Use online JSON validator for JSON data".to_string(),
                        "Check for missing quotes, commas, or brackets".to_string(),
                        "Verify number formats (no letters in numbers)".to_string(),
                    ],
                    expected_outcome: "Data should pass format validation".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "JSON format".to_string(),
                "Data validation".to_string(),
            ],
            platform_notes: vec![],
            examples: vec![
                "Valid JSON: {\"key\": \"value\"}".to_string(),
                "Valid number: 123".to_string(),
            ],
            is_common: false,
            resolution_time: ResolutionTime::Quick,
        }
    }
    
    /// Generate authentication error message
    fn generate_auth_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "Authentication Failed".to_string(),
            description: "The server requires authentication that was not provided or is invalid.".to_string(),
            immediate_actions: vec![
                "Check if the URL requires login".to_string(),
                "Verify credentials if using authentication".to_string(),
                "Try accessing the URL in a browser first".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Verify authentication requirements".to_string(),
                    actions: vec![
                        "Check if URL requires login in browser".to_string(),
                        "Look for authentication documentation".to_string(),
                        "Try public endpoints first".to_string(),
                    ],
                    expected_outcome: "Should understand authentication requirements".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "Authentication".to_string(),
                "API keys".to_string(),
            ],
            platform_notes: vec![],
            examples: vec![
                "Try public endpoint: https://httpbin.org/get".to_string(),
            ],
            is_common: false,
            resolution_time: ResolutionTime::Moderate,
        }
    }
    
    /// Generate test execution error message
    fn generate_test_execution_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "Test Execution Failed".to_string(),
            description: "The test execution process encountered an error. This might be temporary and retrying could resolve it.".to_string(),
            immediate_actions: vec![
                "Try running the test again".to_string(),
                "Check if the server is temporarily unavailable".to_string(),
                "Reduce test intensity (lower count, longer timeout)".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Retry with reduced intensity".to_string(),
                    actions: vec![
                        "Try: --count 1 --timeout 30".to_string(),
                        "Test one URL at a time".to_string(),
                        "Wait a few minutes and retry".to_string(),
                    ],
                    expected_outcome: "Reduced intensity tests should succeed".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "Test configuration".to_string(),
                "Retry strategies".to_string(),
            ],
            platform_notes: vec![],
            examples: vec![
                "network-latency-tester --url https://example.com --count 1".to_string(),
            ],
            is_common: false,
            resolution_time: ResolutionTime::Quick,
        }
    }
    
    /// Generate statistics error message
    fn generate_statistics_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "Statistics Calculation Failed".to_string(),
            description: "Unable to calculate statistics from test results. This usually means insufficient or invalid test data.".to_string(),
            immediate_actions: vec![
                "Ensure tests are completing successfully".to_string(),
                "Try running more test iterations".to_string(),
                "Check for data consistency".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Verify test data quality".to_string(),
                    actions: vec![
                        "Run with --verbose to see individual results".to_string(),
                        "Ensure at least one test succeeds".to_string(),
                        "Try increasing --count to get more data points".to_string(),
                    ],
                    expected_outcome: "Should have valid test results for statistics".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "Statistics calculation".to_string(),
                "Test data quality".to_string(),
            ],
            platform_notes: vec![],
            examples: vec![
                "network-latency-tester --url https://example.com --count 5 --verbose".to_string(),
            ],
            is_common: false,
            resolution_time: ResolutionTime::Quick,
        }
    }
    
    /// Generate internal error message
    fn generate_internal_error_message(&self, _error: &AppError) -> EnhancedErrorMessage {
        EnhancedErrorMessage {
            message: "Internal Application Error".to_string(),
            description: "An unexpected internal error occurred. This is likely a bug in the application.".to_string(),
            immediate_actions: vec![
                "Try running the command again".to_string(),
                "Report this issue with the error details".to_string(),
                "Try with different parameters to work around the issue".to_string(),
            ],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Gather error information".to_string(),
                    actions: vec![
                        "Run with --debug for detailed error information".to_string(),
                        "Note the exact command that caused the error".to_string(),
                        "Try reproducing the error with minimal options".to_string(),
                    ],
                    expected_outcome: "Should gather information for bug report".to_string(),
                    failure_next_step: None,
                },
            ],
            help_topics: vec![
                "Bug reporting".to_string(),
                "Debug mode".to_string(),
            ],
            platform_notes: vec![],
            examples: vec![
                "network-latency-tester --url https://example.com --debug".to_string(),
            ],
            is_common: false,
            resolution_time: ResolutionTime::Advanced,
        }
    }
    
    /// Format enhanced message for console display
    pub fn format_enhanced_message(&self, message: &EnhancedErrorMessage) -> String {
        let mut output = String::new();
        
        // Title
        if self.config.use_color {
            output.push_str(&format!("{} {}\n", 
                "🚨".to_string(),
                message.message.red().bold()
            ));
        } else {
            output.push_str(&format!("ERROR: {}\n", message.message));
        }
        
        // Description
        output.push_str(&format!("\n{}\n", message.description));
        
        // Resolution time indicator
        if self.config.show_detailed_steps {
            let time_indicator = if self.config.use_color {
                format!("{} Estimated resolution time: {}", 
                    message.resolution_time.emoji(),
                    message.resolution_time.description().cyan()
                )
            } else {
                format!("Estimated resolution time: {}", message.resolution_time.description())
            };
            output.push_str(&format!("\n{}\n", time_indicator));
        }
        
        // Immediate actions
        if !message.immediate_actions.is_empty() {
            let header = if self.config.use_color {
                "🔧 Quick Fixes:".yellow().bold()
            } else {
                "Quick Fixes:".to_string().into()
            };
            output.push_str(&format!("\n{}\n", header));
            
            for (i, action) in message.immediate_actions.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, action));
            }
        }
        
        // Detailed troubleshooting steps
        if self.config.show_detailed_steps && !message.troubleshooting_steps.is_empty() {
            let header = if self.config.use_color {
                "🔍 Detailed Troubleshooting:".blue().bold()
            } else {
                "Detailed Troubleshooting:".to_string().into()
            };
            output.push_str(&format!("\n{}\n", header));
            
            for step in &message.troubleshooting_steps {
                output.push_str(&format!("\n  Step {}: {}\n", step.number, step.description));
                
                for action in &step.actions {
                    output.push_str(&format!("    • {}\n", action));
                }
                
                output.push_str(&format!("    Expected: {}\n", step.expected_outcome));
                
                if let Some(next_step) = step.failure_next_step {
                    output.push_str(&format!("    If this fails, continue to Step {}\n", next_step));
                }
            }
        }
        
        // Platform-specific notes
        if self.config.include_platform_notes && !message.platform_notes.is_empty() {
            let platform_specific: Vec<_> = message.platform_notes.iter()
                .filter(|note| note.platforms.contains(&self.config.platform.name().to_string()))
                .collect();
            
            if !platform_specific.is_empty() {
                let header = if self.config.use_color {
                    format!("💻 {} Specific Notes:", self.config.platform.name()).magenta().bold()
                } else {
                    format!("{} Specific Notes:", self.config.platform.name()).into()
                };
                output.push_str(&format!("\n{}\n", header));
                
                for note in platform_specific {
                    output.push_str(&format!("  {}\n", note.message));
                    for action in &note.actions {
                        output.push_str(&format!("    • {}\n", action));
                    }
                }
            }
        }
        
        // Examples
        if !message.examples.is_empty() {
            let header = if self.config.use_color {
                "📋 Examples:".green().bold()
            } else {
                "Examples:".to_string().into()
            };
            output.push_str(&format!("\n{}\n", header));
            
            for example in &message.examples {
                output.push_str(&format!("  {}\n", example));
            }
        }
        
        // Help topics
        if !message.help_topics.is_empty() && self.config.show_detailed_steps {
            let header = if self.config.use_color {
                "📚 Related Help Topics:".cyan().bold()
            } else {
                "Related Help Topics:".to_string().into()
            };
            output.push_str(&format!("\n{}\n", header));
            
            for topic in &message.help_topics {
                output.push_str(&format!("  • {}\n", topic));
                output.push_str(&format!("    Run: network-latency-tester --help {}\n", topic.to_lowercase().replace(' ', "-")));
            }
        }
        
        output
    }
    
    // Platform-specific helper methods
    fn get_config_platform_notes(&self) -> Vec<PlatformNote> {
        vec![
            PlatformNote {
                platforms: vec!["Windows".to_string()],
                message: "On Windows, use PowerShell or Command Prompt for best results".to_string(),
                actions: vec![
                    "Set environment variables with: set TARGET_URLS=https://example.com".to_string(),
                    "Use double quotes around URLs with special characters".to_string(),
                ],
            },
            PlatformNote {
                platforms: vec!["macOS".to_string(), "Linux".to_string()],
                message: "On Unix systems, use your preferred shell".to_string(),
                actions: vec![
                    "Set environment variables with: export TARGET_URLS=https://example.com".to_string(),
                    "Use single quotes to prevent shell interpretation".to_string(),
                ],
            },
        ]
    }
    
    fn get_network_platform_notes(&self) -> Vec<PlatformNote> {
        vec![
            PlatformNote {
                platforms: vec!["Windows".to_string()],
                message: "Windows Defender may block network access".to_string(),
                actions: vec![
                    "Check Windows Defender firewall settings".to_string(),
                    "Allow the application through the firewall".to_string(),
                    "Temporarily disable firewall for testing".to_string(),
                ],
            },
            PlatformNote {
                platforms: vec!["macOS".to_string()],
                message: "macOS may require network permissions".to_string(),
                actions: vec![
                    "Check System Preferences > Security & Privacy > Privacy > Network".to_string(),
                    "Allow network access for the terminal app".to_string(),
                ],
            },
            PlatformNote {
                platforms: vec!["Linux".to_string()],
                message: "Linux systems may have iptables or firewall rules".to_string(),
                actions: vec![
                    "Check: sudo iptables -L".to_string(),
                    "Check: sudo ufw status".to_string(),
                    "Temporarily allow outbound connections".to_string(),
                ],
            },
        ]
    }
    
    fn get_dns_platform_notes(&self) -> Vec<PlatformNote> {
        vec![
            PlatformNote {
                platforms: vec!["Windows".to_string()],
                message: "Windows uses different DNS resolution than Unix systems".to_string(),
                actions: vec![
                    "Check DNS with: nslookup example.com".to_string(),
                    "Flush DNS cache: ipconfig /flushdns".to_string(),
                ],
            },
            PlatformNote {
                platforms: vec!["macOS".to_string()],
                message: "macOS has its own DNS caching system".to_string(),
                actions: vec![
                    "Check DNS with: dig example.com".to_string(),
                    "Flush DNS cache: sudo dscacheutil -flushcache".to_string(),
                ],
            },
            PlatformNote {
                platforms: vec!["Linux".to_string()],
                message: "Linux DNS resolution depends on /etc/resolv.conf".to_string(),
                actions: vec![
                    "Check: cat /etc/resolv.conf".to_string(),
                    "Test with: dig @8.8.8.8 example.com".to_string(),
                ],
            },
        ]
    }
    
    fn get_file_platform_notes(&self) -> Vec<PlatformNote> {
        vec![
            PlatformNote {
                platforms: vec!["Windows".to_string()],
                message: "Windows file paths use backslashes".to_string(),
                actions: vec![
                    "Use: dir filename to check file existence".to_string(),
                    "Check permissions in file properties".to_string(),
                ],
            },
            PlatformNote {
                platforms: vec!["macOS".to_string(), "Linux".to_string()],
                message: "Unix systems use forward slashes in paths".to_string(),
                actions: vec![
                    "Use: ls -la filename to check permissions".to_string(),
                    "Fix permissions with: chmod 644 filename".to_string(),
                ],
            },
        ]
    }
    
    fn get_network_test_commands(&self) -> Vec<String> {
        match self.config.platform {
            Platform::Windows => vec![
                "ping google.com".to_string(),
                "nslookup google.com".to_string(),
                "curl https://google.com".to_string(),
            ],
            Platform::MacOS | Platform::Linux => vec![
                "ping -c 4 google.com".to_string(),
                "dig google.com".to_string(),
                "curl -I https://google.com".to_string(),
            ],
            Platform::Unknown => vec![
                "ping google.com".to_string(),
                "Test network in web browser".to_string(),
            ],
        }
    }
    
    fn get_dns_verification_commands(&self) -> Vec<String> {
        match self.config.platform {
            Platform::Windows => vec![
                "nslookup example.com".to_string(),
                "nslookup example.com 8.8.8.8".to_string(),
            ],
            Platform::MacOS | Platform::Linux => vec![
                "dig example.com".to_string(),
                "dig @8.8.8.8 example.com".to_string(),
                "host example.com".to_string(),
            ],
            Platform::Unknown => vec![
                "Test domain in web browser".to_string(),
            ],
        }
    }
    
    fn get_file_check_commands(&self) -> Vec<String> {
        match self.config.platform {
            Platform::Windows => vec![
                "dir filename".to_string(),
                "Check file properties for permissions".to_string(),
            ],
            Platform::MacOS | Platform::Linux => vec![
                "ls -la filename".to_string(),
                "stat filename".to_string(),
            ],
            Platform::Unknown => vec![
                "Check if file exists".to_string(),
            ],
        }
    }
    
    fn get_disk_space_commands(&self) -> Vec<String> {
        match self.config.platform {
            Platform::Windows => vec![
                "dir".to_string(),
                "Check disk space in File Explorer".to_string(),
            ],
            Platform::MacOS | Platform::Linux => vec![
                "df -h".to_string(),
                "du -sh .".to_string(),
            ],
            Platform::Unknown => vec![
                "Check available disk space".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        assert!(matches!(platform, Platform::Windows | Platform::MacOS | Platform::Linux | Platform::Unknown));
        
        assert_eq!(Platform::Windows.name(), "Windows");
        assert_eq!(Platform::MacOS.name(), "macOS");
        assert_eq!(Platform::Linux.name(), "Linux");
        assert_eq!(Platform::Unknown.name(), "Unknown");
    }
    
    #[test]
    fn test_resolution_time() {
        assert_eq!(ResolutionTime::Quick.description(), "< 1 minute");
        assert_eq!(ResolutionTime::Moderate.description(), "1-5 minutes");
        assert_eq!(ResolutionTime::Complex.description(), "5-15 minutes");
        assert_eq!(ResolutionTime::Advanced.description(), "> 15 minutes");
        
        assert_eq!(ResolutionTime::Quick.emoji(), "🚀");
        assert_eq!(ResolutionTime::Moderate.emoji(), "⚡");
        assert_eq!(ResolutionTime::Complex.emoji(), "🔧");
        assert_eq!(ResolutionTime::Advanced.emoji(), "🛠️");
    }
    
    #[test]
    fn test_user_message_provider_creation() {
        let provider = UserMessageProvider::with_defaults();
        assert!(provider.config.use_color);
        assert!(provider.config.include_platform_notes);
        assert!(provider.config.show_detailed_steps);
    }
    
    #[test]
    fn test_user_message_from_app_config() {
        let app_config = Config {
            debug: true,
            verbose: false,
            enable_color: false,
            ..Default::default()
        };
        
        let provider = UserMessageProvider::from_app_config(&app_config);
        assert!(!provider.config.use_color);
        assert!(provider.config.show_detailed_steps);
        assert_eq!(provider.config.experience_level, ExperienceLevel::Advanced);
    }
    
    #[test]
    fn test_enhanced_message_generation() {
        let mut provider = UserMessageProvider::with_defaults();
        
        let config_error = AppError::config("Invalid URL format");
        let message = provider.get_enhanced_message(&config_error);
        
        assert_eq!(message.message, "Configuration Problem");
        assert!(message.is_common);
        assert_eq!(message.resolution_time, ResolutionTime::Quick);
        assert!(!message.immediate_actions.is_empty());
        assert!(!message.troubleshooting_steps.is_empty());
    }
    
    #[test]
    fn test_message_caching() {
        let mut provider = UserMessageProvider::with_defaults();
        
        let error = AppError::network("Connection failed");
        let message1 = provider.get_enhanced_message(&error);
        let message2 = provider.get_enhanced_message(&error);
        
        // Messages should be identical (cached)
        assert_eq!(message1.message, message2.message);
        assert_eq!(message1.description, message2.description);
    }
    
    #[test]
    fn test_different_error_types() {
        let mut provider = UserMessageProvider::with_defaults();
        
        let errors = vec![
            AppError::config("test"),
            AppError::network("test"),
            AppError::dns_resolution("test"),
            AppError::http_request("test"),
            AppError::timeout("test"),
            AppError::validation("test"),
            AppError::io("test"),
            AppError::parse("test"),
            AppError::auth("test"),
            AppError::test_execution("test"),
            AppError::statistics("test"),
            AppError::internal("test"),
        ];
        
        for error in errors {
            let message = provider.get_enhanced_message(&error);
            assert!(!message.message.is_empty());
            assert!(!message.description.is_empty());
            // Each error type should have a unique message
        }
    }
    
    #[test]
    fn test_message_formatting() {
        let provider = UserMessageProvider::with_defaults();
        
        let message = EnhancedErrorMessage {
            message: "Test Error".to_string(),
            description: "Test description".to_string(),
            immediate_actions: vec!["Action 1".to_string(), "Action 2".to_string()],
            troubleshooting_steps: vec![
                TroubleshootingStep {
                    number: 1,
                    description: "Test step".to_string(),
                    actions: vec!["Step action".to_string()],
                    expected_outcome: "Should work".to_string(),
                    failure_next_step: None,
                }
            ],
            help_topics: vec!["Test topic".to_string()],
            platform_notes: vec![],
            examples: vec!["test command".to_string()],
            is_common: true,
            resolution_time: ResolutionTime::Quick,
        };
        
        let formatted = provider.format_enhanced_message(&message);
        assert!(formatted.contains("Test Error"));
        assert!(formatted.contains("Test description"));
        assert!(formatted.contains("Quick Fixes"));
        assert!(formatted.contains("Action 1"));
        assert!(formatted.contains("Examples"));
    }
    
    #[test]
    fn test_platform_specific_messages() {
        let config = UserMessageConfig {
            use_color: false,
            include_platform_notes: true,
            show_detailed_steps: true,
            platform: Platform::Windows,
            experience_level: ExperienceLevel::Intermediate,
        };
        
        let provider = UserMessageProvider::new(config);
        let notes = provider.get_network_platform_notes();
        
        let windows_notes: Vec<_> = notes.iter()
            .filter(|note| note.platforms.contains(&"Windows".to_string()))
            .collect();
        
        assert!(!windows_notes.is_empty());
        assert!(windows_notes[0].message.contains("Windows"));
    }
    
    #[test]
    fn test_troubleshooting_step_structure() {
        let step = TroubleshootingStep {
            number: 1,
            description: "Test description".to_string(),
            actions: vec!["Action 1".to_string(), "Action 2".to_string()],
            expected_outcome: "Should work".to_string(),
            failure_next_step: Some(2),
        };
        
        assert_eq!(step.number, 1);
        assert_eq!(step.actions.len(), 2);
        assert_eq!(step.failure_next_step, Some(2));
    }
    
    #[test]
    fn test_experience_level_based_messaging() {
        // Test that different experience levels get appropriate messaging
        let beginner_config = UserMessageConfig {
            use_color: true,
            include_platform_notes: true,
            show_detailed_steps: true,
            platform: Platform::current(),
            experience_level: ExperienceLevel::Beginner,
        };
        
        let advanced_config = UserMessageConfig {
            use_color: true,
            include_platform_notes: false,
            show_detailed_steps: false,
            platform: Platform::current(),
            experience_level: ExperienceLevel::Advanced,
        };
        
        let beginner_provider = UserMessageProvider::new(beginner_config);
        let advanced_provider = UserMessageProvider::new(advanced_config);
        
        // Configurations should be different
        assert!(beginner_provider.config.show_detailed_steps);
        assert!(!advanced_provider.config.show_detailed_steps);
        assert!(beginner_provider.config.include_platform_notes);
        assert!(!advanced_provider.config.include_platform_notes);
    }
}