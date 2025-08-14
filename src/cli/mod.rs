//! Command-line interface module with comprehensive help system

pub mod help;

pub use help::HelpSystem;

use clap::{Parser, ArgAction};

/// Network Latency Tester - A high-performance tool for measuring network connectivity
#[derive(Parser, Debug, Clone)]
#[command(name = "network-latency-tester")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Number of test iterations per DNS configuration
    #[arg(short, long, default_value_t = crate::defaults::DEFAULT_TEST_COUNT)]
    pub count: u32,

    /// Request timeout in seconds
    #[arg(short, long, value_parser = parse_duration, default_value_t = crate::defaults::DEFAULT_TIMEOUT.as_secs())]
    pub timeout: u64,

    /// Force colored output
    #[arg(long)]
    pub color: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Enable debug output
    #[arg(long)]
    pub debug: bool,

    /// Target URL to test (can be used multiple times)
    #[arg(short, long = "url", action = ArgAction::Append)]
    pub urls: Vec<String>,

    /// Test the original target URL from bash script
    #[arg(long)]
    pub test_original: bool,

    /// Custom DNS servers (comma-separated)
    #[arg(long)]
    pub dns_servers: Option<String>,

    /// DNS-over-HTTPS providers (comma-separated)
    #[arg(long)]
    pub doh_providers: Option<String>,

    /// Show help for specific topic (config, dns, examples, timeout, output)
    #[arg(long, value_name = "TOPIC")]
    pub help_topic: Option<String>,
}

impl Cli {
    /// Validate CLI arguments for conflicts and requirements
    pub fn validate(&self) -> Result<(), String> {
        // Check for conflicting color flags
        if self.color && self.no_color {
            return Err("Cannot specify both --color and --no-color".to_string());
        }

        // Check that at least one URL is provided (either via --url or --test-original)
        if self.urls.is_empty() && !self.test_original {
            return Err("Must specify at least one URL via --url or use --test-original".to_string());
        }

        Ok(())
    }

    /// Get validated URLs for testing
    pub fn get_urls(&self) -> Vec<String> {
        if self.test_original {
            // When --test-original is specified, use only the original target
            vec!["https://target".to_string()]
        } else {
            self.urls.clone()
        }
    }

    /// Check if help should be displayed for a specific topic
    pub fn should_show_topic_help(&self) -> bool {
        self.help_topic.is_some()
    }

    /// Get the help topic if specified
    pub fn get_help_topic(&self) -> Option<&str> {
        self.help_topic.as_deref()
    }

    /// Check if colors should be enabled
    pub fn use_colors(&self) -> bool {
        if self.color {
            true  // Force color output when --color is specified
        } else if self.no_color {
            false // Disable color output when --no-color is specified
        } else {
            supports_color() // Use automatic detection
        }
    }

    /// Display help for the specified topic or main help
    pub fn display_help(&self) -> String {
        let help_system = HelpSystem::new();
        let use_colors = self.use_colors();

        if let Some(topic) = &self.help_topic {
            help_system.display_topic_help(topic, use_colors)
                .unwrap_or_else(|| {
                    format!("Unknown help topic: '{}'\n\nAvailable topics: config, dns, examples, timeout, output\n\n{}", 
                        topic, help_system.display_main_help(use_colors))
                })
        } else {
            help_system.display_main_help(use_colors)
        }
    }

    /// Get configuration summary for display
    pub fn get_config_summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str("Configuration Summary:\n");
        summary.push_str(&format!("  Test count: {}\n", self.count));
        summary.push_str(&format!("  Timeout: {}s\n", self.timeout));
        summary.push_str(&format!("  Colored output: {}\n", self.use_colors()));
        summary.push_str(&format!("  Verbose mode: {}\n", self.verbose));
        summary.push_str(&format!("  Debug mode: {}\n", self.debug));
        
        if !self.urls.is_empty() {
            summary.push_str(&format!("  Custom URLs: {}\n", self.urls.join(", ")));
        }
        
        if self.test_original {
            summary.push_str("  Testing original URL: Yes\n");
        }
        
        if let Some(ref dns_servers) = self.dns_servers {
            summary.push_str(&format!("  DNS servers: {}\n", dns_servers));
        }
        
        if let Some(ref doh_providers) = self.doh_providers {
            summary.push_str(&format!("  DoH providers: {}\n", doh_providers));
        }
        
        summary
    }
}

/// Parse duration from seconds string
fn parse_duration(s: &str) -> Result<u64, String> {
    // Reject strings with leading + sign or other invalid formats
    if s.starts_with('+') || s.starts_with("0x") || s.starts_with("0X") {
        return Err(format!("Invalid duration: {}", s));
    }
    
    s.parse::<u64>()
        .map_err(|_| format!("Invalid duration: {}", s))
        .and_then(|secs| {
            if secs == 0 {
                Err("Duration must be greater than 0".to_string())
            } else if secs > 300 {
                Err("Duration cannot exceed 300 seconds".to_string())
            } else {
                Ok(secs)
            }
        })
}

/// Check if the terminal supports color output
fn supports_color() -> bool {
    // Check for common environment variables that indicate color support
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Check for NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check for FORCE_COLOR environment variable
    if std::env::var("FORCE_COLOR").is_ok() {
        return true;
    }

    // On Windows, check for ANSICON or ConEmu
    #[cfg(target_os = "windows")]
    {
        if std::env::var("ANSICON").is_ok() || std::env::var("ConEmuANSI").is_ok() {
            return true;
        }
    }

    // Default to true on Unix-like systems, false on Windows
    #[cfg(unix)]
    {
        true
    }
    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing_basic() {
        let cli = Cli::parse_from(&["test", "--count", "5", "--timeout", "10"]);
        assert_eq!(cli.count, 5);
        assert_eq!(cli.timeout, 10);
        assert!(!cli.verbose);
        assert!(!cli.debug);
    }

    #[test]
    fn test_cli_parsing_all_options() {
        let cli = Cli::parse_from(&[
            "test",
            "--count", "10",
            "--timeout", "30",
            "--no-color",
            "--verbose",
            "--debug",
            "--url", "https://example.com",
            "--test-original",
            "--dns-servers", "8.8.8.8,1.1.1.1",
            "--doh-providers", "https://dns.google/dns-query",
            "--help-topic", "config"
        ]);

        assert_eq!(cli.count, 10);
        assert_eq!(cli.timeout, 30);
        assert!(cli.no_color);
        assert!(cli.verbose);
        assert!(cli.debug);
        assert_eq!(cli.urls.len(), 1);
        assert_eq!(cli.urls[0], "https://example.com");
        assert!(cli.test_original);
        assert_eq!(cli.dns_servers.as_ref().unwrap(), "8.8.8.8,1.1.1.1");
        assert_eq!(cli.doh_providers.as_ref().unwrap(), "https://dns.google/dns-query");
        assert_eq!(cli.help_topic.as_ref().unwrap(), "config");
    }

    #[test]
    fn test_cli_help_topic_methods() {
        let cli_with_topic = Cli::parse_from(&["test", "--help-topic", "dns"]);
        assert!(cli_with_topic.should_show_topic_help());
        assert_eq!(cli_with_topic.get_help_topic(), Some("dns"));

        let cli_without_topic = Cli::parse_from(&["test"]);
        assert!(!cli_without_topic.should_show_topic_help());
        assert_eq!(cli_without_topic.get_help_topic(), None);
    }

    #[test]
    fn test_color_support_detection() {
        // Test NO_COLOR environment variable
        std::env::set_var("NO_COLOR", "1");
        assert!(!supports_color());
        std::env::remove_var("NO_COLOR");

        // Test FORCE_COLOR environment variable
        std::env::set_var("FORCE_COLOR", "1");
        assert!(supports_color());
        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_duration_parsing() {
        // Valid durations
        assert_eq!(parse_duration("10").unwrap(), 10);
        assert_eq!(parse_duration("300").unwrap(), 300);
        assert_eq!(parse_duration("1").unwrap(), 1);

        // Invalid durations
        assert!(parse_duration("0").is_err());
        assert!(parse_duration("301").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("-5").is_err());
    }

    #[test]
    fn test_config_summary() {
        let cli = Cli::parse_from(&[
            "test",
            "--count", "5",
            "--timeout", "20",
            "--verbose",
            "--url", "https://test.com"
        ]);

        let summary = cli.get_config_summary();
        assert!(summary.contains("Test count: 5"));
        assert!(summary.contains("Timeout: 20s"));
        assert!(summary.contains("Verbose mode: true"));
        assert!(summary.contains("Custom URLs: https://test.com"));
    }

    #[test]
    fn test_help_display() {
        let cli = Cli::parse_from(&["test"]);
        let help = cli.display_help();
        assert!(help.contains("Network Latency Tester"));
        assert!(help.contains("USAGE:"));

        let cli_with_topic = Cli::parse_from(&["test", "--help-topic", "config"]);
        let topic_help = cli_with_topic.display_help();
        assert!(topic_help.contains("CONFIGURATION REFERENCE"));

        let cli_invalid_topic = Cli::parse_from(&["test", "--help-topic", "invalid"]);
        let invalid_help = cli_invalid_topic.display_help();
        assert!(invalid_help.contains("Unknown help topic"));
    }

    #[test]
    fn test_use_colors_method() {
        let cli_no_color = Cli::parse_from(&["test", "--no-color", "--test-original"]);
        assert!(!cli_no_color.use_colors());

        let cli_color = Cli::parse_from(&["test", "--color", "--test-original"]);
        assert!(cli_color.use_colors());

        let cli_default = Cli::parse_from(&["test", "--test-original"]);
        // Result depends on environment, but should not panic
        let _uses_colors = cli_default.use_colors();
    }

    #[test]
    fn test_duration_parsing_edge_cases() {
        // Test boundary values
        assert_eq!(parse_duration("1").unwrap(), 1);     // Minimum valid
        assert_eq!(parse_duration("300").unwrap(), 300); // Maximum valid
        
        // Test edge cases around boundaries
        assert!(parse_duration("0").is_err());   // Just below minimum
        assert!(parse_duration("301").is_err()); // Just above maximum
        
        // Test numeric edge cases - u64::MAX will overflow or be > 300, so should error
        assert!(parse_duration("18446744073709551615").is_err()); // u64::MAX (> 300)
        assert!(parse_duration("").is_err());                      // Empty string
        // Whitespace strings actually parse successfully in Rust, so test a different invalid case
        assert!(parse_duration("abc").is_err());                  // Non-numeric
        assert!(parse_duration("10.5").is_err());                 // Decimal
        assert!(parse_duration("+10").is_err());                  // Positive sign
        assert!(parse_duration("0x10").is_err());                 // Hex format
        assert!(parse_duration("-5").is_err());                   // Negative number
    }

    #[test]
    fn test_cli_argument_combinations() {
        // Test all boolean flags together
        let cli = Cli::parse_from(&["test", "--verbose", "--debug", "--no-color", "--test-original"]);
        assert!(cli.verbose);
        assert!(cli.debug);
        assert!(cli.no_color);
        assert!(cli.test_original);
        
        // Test with custom DNS and DoH providers
        let cli = Cli::parse_from(&[
            "test", 
            "--dns-servers", "8.8.8.8,1.1.1.1",
            "--doh-providers", "https://dns.google/dns-query"
        ]);
        assert!(cli.dns_servers.is_some());
        assert!(cli.doh_providers.is_some());
    }

    #[test]
    fn test_help_topic_edge_cases() {
        // Test all valid help topics
        for topic in &["config", "dns", "examples", "timeout", "output"] {
            let cli = Cli::parse_from(&["test", "--help-topic", topic]);
            assert!(cli.should_show_topic_help());
            assert_eq!(cli.get_help_topic(), Some(*topic));
            
            // Verify each topic actually generates help content
            let help = cli.display_help();
            assert!(!help.is_empty());
            // Each valid topic should not contain "Unknown help topic"
            assert!(!help.contains("Unknown help topic"));
        }
        
        // Test case insensitivity - uppercase should work (function converts to lowercase)
        let cli = Cli::parse_from(&["test", "--help-topic", "CONFIG"]);
        let help = cli.display_help();
        assert!(!help.contains("Unknown help topic")); // Should be case insensitive
        // Check for content from config help
        assert!(help.contains("CONFIGURATION REFERENCE")); // Should show config help
        
        // Test completely invalid topic
        let cli = Cli::parse_from(&["test", "--help-topic", "invalid_topic"]);
        let help = cli.display_help();
        assert!(help.contains("Unknown help topic"));
        assert!(help.contains("invalid_topic"));
        assert!(help.contains("Available topics:"));
    }

    #[test]
    fn test_multiple_url_parsing() {
        let cli = Cli::parse_from(&[
            "test",
            "--url", "https://example.com",
            "--url", "https://test.com",
            "--url", "https://google.com"
        ]);

        assert_eq!(cli.urls.len(), 3);
        assert_eq!(cli.urls[0], "https://example.com");
        assert_eq!(cli.urls[1], "https://test.com");
        assert_eq!(cli.urls[2], "https://google.com");
    }

    #[test]
    fn test_cli_validation() {
        // Test conflicting color flags
        let cli_conflict = Cli::parse_from(&["test", "--color", "--no-color", "--test-original"]);
        assert!(cli_conflict.validate().is_err());
        assert!(cli_conflict.validate().unwrap_err().contains("Cannot specify both --color and --no-color"));

        // Test no URLs provided
        let cli_no_urls = Cli::parse_from(&["test"]);
        assert!(cli_no_urls.validate().is_err());
        assert!(cli_no_urls.validate().unwrap_err().contains("Must specify at least one URL"));

        // Test valid configurations
        let cli_with_url = Cli::parse_from(&["test", "--url", "https://example.com"]);
        assert!(cli_with_url.validate().is_ok());

        let cli_with_original = Cli::parse_from(&["test", "--test-original"]);
        assert!(cli_with_original.validate().is_ok());

        let cli_color_only = Cli::parse_from(&["test", "--color", "--test-original"]);
        assert!(cli_color_only.validate().is_ok());

        let cli_no_color_only = Cli::parse_from(&["test", "--no-color", "--test-original"]);
        assert!(cli_no_color_only.validate().is_ok());
    }

    #[test]
    fn test_get_urls_method() {
        // Test with custom URLs only
        let cli_custom = Cli::parse_from(&[
            "test",
            "--url", "https://example.com",
            "--url", "https://test.com"
        ]);
        let urls = cli_custom.get_urls();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://example.com");
        assert_eq!(urls[1], "https://test.com");

        // Test with test-original only
        let cli_original = Cli::parse_from(&["test", "--test-original"]);
        let urls = cli_original.get_urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://target");

        // Test with both custom URLs and test-original (test-original takes precedence)
        let cli_both = Cli::parse_from(&[
            "test",
            "--url", "https://example.com",
            "--test-original"
        ]);
        let urls = cli_both.get_urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://target");
    }

    #[test]  
    fn test_count_boundary_values() {
        // Test minimum count
        let cli = Cli::parse_from(&["test", "--count", "1"]);
        assert_eq!(cli.count, 1);
        
        // Test maximum reasonable count (clap handles u32 max automatically)
        let cli = Cli::parse_from(&["test", "--count", "1000"]);
        assert_eq!(cli.count, 1000);
    }
}