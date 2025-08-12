//! Comprehensive command-line help system with examples and detailed guidance
//!
//! This module provides detailed help text, usage examples, and contextual guidance
//! to help users effectively use the network latency tester.

use crate::{
    config::env::EnvManager,
    dns::platform::get_platform_name,
};
use colored::*;

/// Comprehensive help system for the CLI application
pub struct HelpSystem {
    platform: String,
}

impl HelpSystem {
    /// Create a new help system
    pub fn new() -> Self {
        Self {
            platform: get_platform_name(),
        }
    }

    /// Display the main help message with all available options
    pub fn display_main_help(&self, use_colors: bool) -> String {
        let mut help = String::new();
        
        // Header
        help.push_str(&self.format_header(use_colors));
        help.push_str("\n");
        
        // Usage section
        help.push_str(&self.format_usage_section(use_colors));
        help.push_str("\n");
        
        // Options section
        help.push_str(&self.format_options_section(use_colors));
        help.push_str("\n");
        
        // Examples section
        help.push_str(&self.format_examples_section(use_colors));
        help.push_str("\n");
        
        // Environment variables section
        help.push_str(&self.format_environment_section(use_colors));
        help.push_str("\n");
        
        // Configuration section
        help.push_str(&self.format_configuration_section(use_colors));
        help.push_str("\n");
        
        // Footer with additional resources
        help.push_str(&self.format_footer(use_colors));
        
        help
    }

    /// Display quick help for specific topics
    pub fn display_topic_help(&self, topic: &str, use_colors: bool) -> Option<String> {
        match topic.to_lowercase().as_str() {
            "config" | "configuration" => Some(self.format_configuration_help(use_colors)),
            "env" | "environment" => Some(self.format_environment_help(use_colors)),
            "dns" => Some(self.format_dns_help(use_colors)),
            "examples" => Some(self.format_examples_section(use_colors)),
            "timeout" | "timeouts" => Some(self.format_timeout_help(use_colors)),
            "output" | "formatting" => Some(self.format_output_help(use_colors)),
            _ => None,
        }
    }

    /// Format the main header
    fn format_header(&self, use_colors: bool) -> String {
        let title = "Network Latency Tester";
        let subtitle = "High-performance network latency testing tool with DNS configuration support";
        let version = env!("CARGO_PKG_VERSION");
        
        if use_colors {
            format!(
                "{}\n{}\nVersion: {} | Platform: {}\n",
                title.bright_cyan().bold(),
                subtitle.bright_blue(),
                version.green(),
                self.platform.yellow()
            )
        } else {
            format!(
                "{}\n{}\nVersion: {} | Platform: {}\n",
                title, subtitle, version, self.platform
            )
        }
    }

    /// Format the usage section
    fn format_usage_section(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "USAGE:".bright_green().bold().to_string()
        } else {
            "USAGE:".to_string()
        };

        let usage_patterns = vec![
            "network-latency-tester [OPTIONS]",
            "network-latency-tester --url <URL> [OPTIONS]", 
            "network-latency-tester --test-original [OPTIONS]",
            "network-latency-tester --help [TOPIC]",
        ];

        let mut usage = format!("{}\n", header);
        for pattern in usage_patterns {
            if use_colors {
                usage.push_str(&format!("  {}\n", pattern.bright_white()));
            } else {
                usage.push_str(&format!("  {}\n", pattern));
            }
        }
        
        usage
    }

    /// Format the options section
    fn format_options_section(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "OPTIONS:".bright_green().bold().to_string()
        } else {
            "OPTIONS:".to_string()
        };

        let options = vec![
            OptionHelp {
                short: Some("u"),
                long: "url",
                value: "<URL>",
                description: "Target URL to test (can be used multiple times)",
                example: Some("--url https://google.com --url https://github.com"),
            },
            OptionHelp {
                short: Some("c"),
                long: "count",
                value: "<NUMBER>",
                description: "Number of test iterations per configuration (1-100)",
                example: Some("--count 10"),
            },
            OptionHelp {
                short: Some("t"),
                long: "timeout",
                value: "<SECONDS>",
                description: "Request timeout in seconds (1-300)",
                example: Some("--timeout 30"),
            },
            OptionHelp {
                short: None,
                long: "dns-servers",
                value: "<IPS>",
                description: "Custom DNS servers (comma-separated IP addresses)",
                example: Some("--dns-servers 8.8.8.8,1.1.1.1"),
            },
            OptionHelp {
                short: None,
                long: "doh-providers",
                value: "<URLS>",
                description: "DNS-over-HTTPS providers (comma-separated HTTPS URLs)",
                example: Some("--doh-providers https://dns.google/dns-query"),
            },
            OptionHelp {
                short: None,
                long: "test-original",
                value: "",
                description: "Test the original ctok.ai URL from the bash script",
                example: Some("--test-original"),
            },
            OptionHelp {
                short: Some("v"),
                long: "verbose",
                value: "",
                description: "Enable verbose output with detailed timing information",
                example: Some("--verbose"),
            },
            OptionHelp {
                short: None,
                long: "debug",
                value: "",
                description: "Enable debug output with diagnostic information",
                example: Some("--debug"),
            },
            OptionHelp {
                short: None,
                long: "no-color",
                value: "",
                description: "Disable colored output",
                example: Some("--no-color"),
            },
            OptionHelp {
                short: Some("h"),
                long: "help",
                value: "[TOPIC]",
                description: "Show help information (optionally for specific topic)",
                example: Some("--help config"),
            },
        ];

        let mut output = format!("{}\n", header);
        for option in options {
            output.push_str(&option.format(use_colors));
            output.push_str("\n");
        }
        
        output
    }

    /// Format the examples section
    fn format_examples_section(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "EXAMPLES:".bright_green().bold().to_string()
        } else {
            "EXAMPLES:".to_string()
        };

        let examples = vec![
            ExampleHelp {
                title: "Basic latency test",
                command: "network-latency-tester --url https://google.com",
                description: "Test latency to Google with default settings",
            },
            ExampleHelp {
                title: "Multiple URLs with custom DNS",
                command: "network-latency-tester --url https://google.com --url https://github.com --dns-servers 8.8.8.8,1.1.1.1",
                description: "Test multiple URLs using custom DNS servers",
            },
            ExampleHelp {
                title: "Original script compatibility test",
                command: "network-latency-tester --test-original --count 5 --verbose",
                description: "Run the original ctok.ai test with 5 iterations and verbose output",
            },
            ExampleHelp {
                title: "DNS-over-HTTPS testing",
                command: "network-latency-tester --url https://cloudflare.com --doh-providers https://dns.google/dns-query,https://cloudflare-dns.com/dns-query",
                description: "Test using DNS-over-HTTPS providers",
            },
            ExampleHelp {
                title: "High-frequency testing",
                command: "network-latency-tester --url https://api.example.com --count 50 --timeout 10",
                description: "Run 50 test iterations with 10-second timeout",
            },
            ExampleHelp {
                title: "Debug mode with no colors",
                command: "network-latency-tester --url https://example.com --debug --no-color",
                description: "Run with debug output and no color formatting",
            },
        ];

        let mut output = format!("{}\n", header);
        for example in examples {
            output.push_str(&example.format(use_colors));
            output.push_str("\n");
        }
        
        output
    }

    /// Format the environment variables section
    fn format_environment_section(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "ENVIRONMENT VARIABLES:".bright_green().bold().to_string()
        } else {
            "ENVIRONMENT VARIABLES:".to_string()
        };

        let env_vars = EnvManager::get_supported_env_vars();
        
        let mut output = format!("{}\n", header);
        output.push_str("Configuration priority: CLI arguments > Environment variables > Defaults\n\n");
        
        for (var_name, description, _example) in env_vars {
            if use_colors {
                output.push_str(&format!("  {}: {}\n", 
                    var_name.bright_yellow().bold(),
                    description.white()
                ));
            } else {
                output.push_str(&format!("  {}: {}\n", var_name, description));
            }
        }

        output.push_str("\nExample .env file:\n");
        if use_colors {
            output.push_str(&format!("  {}\n", "TARGET_URLS=https://google.com,https://github.com".bright_blue()));
            output.push_str(&format!("  {}\n", "DNS_SERVERS=8.8.8.8,1.1.1.1".bright_blue()));
            output.push_str(&format!("  {}\n", "TEST_COUNT=10".bright_blue()));
            output.push_str(&format!("  {}\n", "TIMEOUT_SECONDS=30".bright_blue()));
        } else {
            output.push_str("  TARGET_URLS=https://google.com,https://github.com\n");
            output.push_str("  DNS_SERVERS=8.8.8.8,1.1.1.1\n");
            output.push_str("  TEST_COUNT=10\n");
            output.push_str("  TIMEOUT_SECONDS=30\n");
        }

        output
    }

    /// Format the configuration section
    fn format_configuration_section(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "CONFIGURATION:".bright_green().bold().to_string()
        } else {
            "CONFIGURATION:".to_string()
        };

        let mut output = format!("{}\n", header);
        output.push_str("The application supports multiple configuration methods:\n\n");
        
        let config_methods = vec![
            ("Command-line arguments", "Highest priority, override all other settings"),
            ("Environment variables", "Medium priority, can be set in shell or .env file"),
            ("Default values", "Lowest priority, sensible defaults for all platforms"),
        ];

        for (method, description) in config_methods {
            if use_colors {
                output.push_str(&format!("  {}: {}\n",
                    method.bright_cyan().bold(),
                    description.white()
                ));
            } else {
                output.push_str(&format!("  {}: {}\n", method, description));
            }
        }

        output.push_str(&format!("\nPlatform-specific defaults for {}:\n", self.platform));
        output.push_str("  - Timeouts are optimized for platform networking characteristics\n");
        output.push_str("  - DNS servers include platform-appropriate public resolvers\n");
        output.push_str("  - Connection limits are set based on platform capabilities\n");

        output
    }

    /// Format the footer with additional resources
    fn format_footer(&self, use_colors: bool) -> String {
        let mut footer = String::new();
        
        if use_colors {
            footer.push_str(&format!("{}\n", "ADDITIONAL HELP:".bright_green().bold()));
        } else {
            footer.push_str("ADDITIONAL HELP:\n");
        }
        
        let help_topics = vec![
            ("--help config", "Configuration file and environment variable details"),
            ("--help dns", "DNS configuration and troubleshooting"),
            ("--help examples", "More detailed usage examples"),
            ("--help timeout", "Timeout configuration and optimization"),
            ("--help output", "Output formatting and interpretation"),
        ];

        for (command, description) in help_topics {
            if use_colors {
                footer.push_str(&format!("  {}: {}\n",
                    command.bright_yellow(),
                    description.white()
                ));
            } else {
                footer.push_str(&format!("  {}: {}\n", command, description));
            }
        }

        footer.push_str("\nFor more information, visit the project documentation or GitHub repository.\n");
        
        footer
    }

    /// Format detailed configuration help
    fn format_configuration_help(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "CONFIGURATION REFERENCE:".bright_green().bold().to_string()
        } else {
            "CONFIGURATION REFERENCE:".to_string()
        };

        let mut help = format!("{}\n\n", header);
        
        help.push_str("CONFIGURATION PRIORITY (highest to lowest):\n");
        help.push_str("1. Command-line arguments\n");
        help.push_str("2. Environment variables\n");
        help.push_str("3. Default values\n\n");

        help.push_str("PARAMETER LIMITS:\n");
        help.push_str("- Test count: 1-100 iterations\n");
        help.push_str("- Timeout: 1-300 seconds\n");
        help.push_str("- URLs: No limit, but memory usage increases with count\n");
        help.push_str("- DNS servers: IPv4 and IPv6 addresses supported\n");
        help.push_str("- DoH providers: Must be HTTPS URLs\n\n");

        help.push_str(&format!("PLATFORM-SPECIFIC OPTIMIZATIONS ({}):\n", self.platform));
        match self.platform.as_str() {
            "Windows" => {
                help.push_str("- Extended timeouts for Windows networking characteristics\n");
                help.push_str("- Windows certificate store integration\n");
                help.push_str("- Firewall consideration in diagnostics\n");
            }
            "macOS" => {
                help.push_str("- Optimized for excellent IPv6 support\n");
                help.push_str("- Aggressive timeouts for reliable networking\n");
                help.push_str("- Native TLS 1.3 support\n");
            }
            "Linux" => {
                help.push_str("- High concurrency support\n");
                help.push_str("- systemd-resolved integration consideration\n");
                help.push_str("- Optimized for server environments\n");
            }
            _ => {
                help.push_str("- Conservative defaults for unknown platform\n");
            }
        }

        help
    }

    /// Format detailed environment help
    fn format_environment_help(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "ENVIRONMENT VARIABLES REFERENCE:".bright_green().bold().to_string()
        } else {
            "ENVIRONMENT VARIABLES REFERENCE:".to_string()
        };

        let mut help = format!("{}\n\n", header);
        
        help.push_str("LOADING ORDER:\n");
        help.push_str("1. System environment variables\n");
        help.push_str("2. .env file in current directory (if present)\n");
        help.push_str("3. Command-line arguments (override both)\n\n");

        help.push_str("SUPPORTED VARIABLES:\n");
        let env_vars = EnvManager::get_supported_env_vars();
        for (var_name, description, example) in env_vars {
            if use_colors {
                help.push_str(&format!("{}:\n  {}\n  Example: {}\n\n", 
                    var_name.bright_yellow().bold(),
                    description.white(),
                    example.bright_blue().italic()
                ));
            } else {
                help.push_str(&format!("{}:\n  {}\n  Example: {}\n\n", var_name, description, example));
            }
        }

        help.push_str("EXAMPLE .env FILE:\n");
        help.push_str(&EnvManager::create_example_env_content());

        help
    }

    /// Format DNS-specific help
    fn format_dns_help(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "DNS CONFIGURATION REFERENCE:".bright_green().bold().to_string()
        } else {
            "DNS CONFIGURATION REFERENCE:".to_string()
        };

        let mut help = format!("{}\n\n", header);
        
        help.push_str("DNS RESOLVER TYPES:\n");
        help.push_str("1. System DNS - Uses operating system default resolver\n");
        help.push_str("2. Custom DNS - Specify custom DNS server IP addresses\n");
        help.push_str("3. DNS-over-HTTPS (DoH) - Encrypted DNS queries over HTTPS\n\n");

        help.push_str("POPULAR PUBLIC DNS SERVERS:\n");
        help.push_str("- Google DNS: 8.8.8.8, 8.8.4.4\n");
        help.push_str("- Cloudflare DNS: 1.1.1.1, 1.0.0.1\n");
        help.push_str("- OpenDNS: 208.67.222.222, 208.67.220.220\n");
        help.push_str("- Quad9 DNS: 9.9.9.9, 149.112.112.112\n\n");

        help.push_str("DNS-over-HTTPS PROVIDERS:\n");
        help.push_str("- Google: https://dns.google/dns-query\n");
        help.push_str("- Cloudflare: https://cloudflare-dns.com/dns-query\n");
        help.push_str("- Quad9: https://dns.quad9.net/dns-query\n\n");

        help.push_str(&format!("PLATFORM-SPECIFIC DNS NOTES ({}):\n", self.platform));
        match self.platform.as_str() {
            "Windows" => {
                help.push_str("- Windows may have slower DNS resolution\n");
                help.push_str("- IPv6 support varies by Windows version\n");
                help.push_str("- Consider Windows Firewall impacts\n");
            }
            "macOS" => {
                help.push_str("- Excellent DNS resolution performance\n");
                help.push_str("- Strong IPv6 support\n");
                help.push_str("- mDNS (.local) domains supported\n");
            }
            "Linux" => {
                help.push_str("- Fast DNS resolution with systemd-resolved\n");
                help.push_str("- Check /etc/systemd/resolved.conf for system DNS\n");
                help.push_str("- IPv6 support depends on network configuration\n");
            }
            _ => {}
        }

        help
    }

    /// Format timeout-specific help
    fn format_timeout_help(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "TIMEOUT CONFIGURATION REFERENCE:".bright_green().bold().to_string()
        } else {
            "TIMEOUT CONFIGURATION REFERENCE:".to_string()
        };

        let mut help = format!("{}\n\n", header);
        
        help.push_str("TIMEOUT TYPES:\n");
        help.push_str("- DNS Resolution: Time to resolve domain name to IP\n");
        help.push_str("- TCP Connection: Time to establish TCP connection\n");
        help.push_str("- TLS Handshake: Time to complete TLS/SSL handshake\n");
        help.push_str("- HTTP Request: Total time for HTTP request/response\n\n");

        help.push_str("TIMEOUT RECOMMENDATIONS:\n");
        help.push_str("- Local network: 1-5 seconds\n");
        help.push_str("- Internet services: 10-30 seconds\n");
        help.push_str("- Slow/distant servers: 30-60 seconds\n");
        help.push_str("- High-latency connections: 60+ seconds\n\n");

        help.push_str(&format!("PLATFORM-SPECIFIC DEFAULTS ({}):\n", self.platform));
        match self.platform.as_str() {
            "Windows" => {
                help.push_str("- Connection timeout: 10 seconds\n");
                help.push_str("- Request timeout: 30 seconds\n");
                help.push_str("- Longer timeouts due to Windows networking characteristics\n");
            }
            "macOS" | "Linux" => {
                help.push_str("- Connection timeout: 5 seconds\n");
                help.push_str("- Request timeout: 20 seconds\n");
                help.push_str("- Optimized for reliable Unix networking\n");
            }
            _ => {
                help.push_str("- Conservative defaults for unknown platform\n");
            }
        }

        help.push_str("\nADAPTIVE TIMEOUTS:\n");
        help.push_str("The application learns from previous requests and adapts timeouts:\n");
        help.push_str("- Successful fast responses → shorter timeouts\n");
        help.push_str("- Frequent timeouts → longer timeouts\n");
        help.push_str("- Platform differences are automatically considered\n");

        help
    }

    /// Format output formatting help
    fn format_output_help(&self, use_colors: bool) -> String {
        let header = if use_colors {
            "OUTPUT FORMATTING REFERENCE:".bright_green().bold().to_string()
        } else {
            "OUTPUT FORMATTING REFERENCE:".to_string()
        };

        let mut help = format!("{}\n\n", header);
        
        help.push_str("OUTPUT MODES:\n");
        help.push_str("- Default: Colored output with performance indicators\n");
        help.push_str("- --no-color: Plain text output for scripts/logs\n");
        help.push_str("- --verbose: Additional timing details\n");
        help.push_str("- --debug: Diagnostic information and errors\n\n");

        help.push_str("PERFORMANCE INDICATORS:\n");
        if use_colors {
            help.push_str(&format!("- {}: < 100ms (Excellent)\n", "🟢 Green".green()));
            help.push_str(&format!("- {}: 100-300ms (Good)\n", "🟡 Yellow".yellow()));
            help.push_str(&format!("- {}: 300-1000ms (Fair)\n", "🟠 Orange".bright_yellow()));
            help.push_str(&format!("- {}: > 1000ms (Poor)\n", "🔴 Red".red()));
        } else {
            help.push_str("- Green: < 100ms (Excellent)\n");
            help.push_str("- Yellow: 100-300ms (Good)\n");
            help.push_str("- Orange: 300-1000ms (Fair)\n");
            help.push_str("- Red: > 1000ms (Poor)\n");
        }

        help.push_str("\nSTATISTICS REPORTED:\n");
        help.push_str("- Minimum, Maximum, Average response times\n");
        help.push_str("- Standard deviation and percentiles\n");
        help.push_str("- Success rate and error analysis\n");
        help.push_str("- DNS resolution timing breakdown\n");
        help.push_str("- Best performing configuration recommendations\n\n");

        help.push_str("TROUBLESHOOTING OUTPUT:\n");
        help.push_str("- Use --debug to see detailed error messages\n");
        help.push_str("- Use --verbose to see individual request timings\n");
        help.push_str("- Check connectivity status for network issues\n");
        help.push_str("- Review DNS configuration for resolution problems\n");

        help
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper struct for formatting individual options
struct OptionHelp {
    short: Option<&'static str>,
    long: &'static str,
    value: &'static str,
    description: &'static str,
    example: Option<&'static str>,
}

impl OptionHelp {
    fn format(&self, use_colors: bool) -> String {
        let mut option_str = String::new();
        
        // Format the option flags
        if let Some(short) = self.short {
            if use_colors {
                option_str.push_str(&format!("  {}, ", format!("-{}", short).bright_cyan()));
            } else {
                option_str.push_str(&format!("  -{}, ", short));
            }
        } else {
            option_str.push_str("      ");
        }
        
        let long_with_value = if self.value.is_empty() {
            format!("--{}", self.long)
        } else {
            format!("--{} {}", self.long, self.value)
        };
        
        if use_colors {
            option_str.push_str(&format!("{:<30} {}", 
                long_with_value.bright_cyan(),
                self.description.white()
            ));
        } else {
            option_str.push_str(&format!("{:<30} {}", long_with_value, self.description));
        }
        
        // Add example if provided
        if let Some(example) = self.example {
            if use_colors {
                option_str.push_str(&format!("\n{}{}", " ".repeat(36), 
                    format!("Example: {}", example).bright_blue().italic()
                ));
            } else {
                option_str.push_str(&format!("\n{}Example: {}", " ".repeat(36), example));
            }
        }
        
        option_str
    }
}

/// Helper struct for formatting examples
struct ExampleHelp {
    title: &'static str,
    command: &'static str,
    description: &'static str,
}

impl ExampleHelp {
    fn format(&self, use_colors: bool) -> String {
        if use_colors {
            format!("  {}:\n    {}\n    {}\n",
                self.title.bright_yellow().bold(),
                self.command.bright_white(),
                self.description.bright_blue().italic()
            )
        } else {
            format!("  {}:\n    {}\n    {}\n",
                self.title, self.command, self.description
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_system_creation() {
        let help_system = HelpSystem::new();
        assert!(!help_system.platform.is_empty());
    }

    #[test]
    fn test_main_help_display() {
        let help_system = HelpSystem::new();
        
        let colored_help = help_system.display_main_help(true);
        let plain_help = help_system.display_main_help(false);
        
        // Both should contain essential sections
        assert!(colored_help.contains("Network Latency Tester"));
        assert!(colored_help.contains("USAGE:"));
        assert!(colored_help.contains("OPTIONS:"));
        assert!(colored_help.contains("EXAMPLES:"));
        
        assert!(plain_help.contains("Network Latency Tester"));
        assert!(plain_help.contains("USAGE:"));
        assert!(plain_help.contains("OPTIONS:"));
        assert!(plain_help.contains("EXAMPLES:"));
        
        // Colored version should be longer due to ANSI codes
        assert!(colored_help.len() >= plain_help.len());
    }

    #[test]
    fn test_topic_help() {
        let help_system = HelpSystem::new();
        
        // Valid topics
        assert!(help_system.display_topic_help("config", true).is_some());
        assert!(help_system.display_topic_help("dns", false).is_some());
        assert!(help_system.display_topic_help("examples", true).is_some());
        assert!(help_system.display_topic_help("timeout", false).is_some());
        
        // Invalid topic
        assert!(help_system.display_topic_help("invalid", true).is_none());
    }

    #[test]
    fn test_configuration_help() {
        let help_system = HelpSystem::new();
        
        let config_help = help_system.format_configuration_help(false);
        
        assert!(config_help.contains("CONFIGURATION REFERENCE"));
        assert!(config_help.contains("CONFIGURATION PRIORITY"));
        assert!(config_help.contains("PARAMETER LIMITS"));
        assert!(config_help.contains("PLATFORM-SPECIFIC OPTIMIZATIONS"));
    }

    #[test]
    fn test_environment_help() {
        let help_system = HelpSystem::new();
        
        let env_help = help_system.format_environment_help(false);
        
        assert!(env_help.contains("ENVIRONMENT VARIABLES REFERENCE"));
        assert!(env_help.contains("LOADING ORDER"));
        assert!(env_help.contains("SUPPORTED VARIABLES"));
        assert!(env_help.contains("EXAMPLE .env FILE"));
    }

    #[test]
    fn test_dns_help() {
        let help_system = HelpSystem::new();
        
        let dns_help = help_system.format_dns_help(false);
        
        assert!(dns_help.contains("DNS CONFIGURATION REFERENCE"));
        assert!(dns_help.contains("DNS RESOLVER TYPES"));
        assert!(dns_help.contains("POPULAR PUBLIC DNS SERVERS"));
        assert!(dns_help.contains("DNS-over-HTTPS PROVIDERS"));
    }

    #[test]
    fn test_timeout_help() {
        let help_system = HelpSystem::new();
        
        let timeout_help = help_system.format_timeout_help(false);
        
        assert!(timeout_help.contains("TIMEOUT CONFIGURATION REFERENCE"));
        assert!(timeout_help.contains("TIMEOUT TYPES"));
        assert!(timeout_help.contains("TIMEOUT RECOMMENDATIONS"));
        assert!(timeout_help.contains("ADAPTIVE TIMEOUTS"));
    }

    #[test]
    fn test_output_help() {
        let help_system = HelpSystem::new();
        
        let output_help = help_system.format_output_help(false);
        
        assert!(output_help.contains("OUTPUT FORMATTING REFERENCE"));
        assert!(output_help.contains("OUTPUT MODES"));
        assert!(output_help.contains("PERFORMANCE INDICATORS"));
        assert!(output_help.contains("STATISTICS REPORTED"));
    }

    #[test]
    fn test_option_help_formatting() {
        let option = OptionHelp {
            short: Some("u"),
            long: "url",
            value: "<URL>",
            description: "Target URL to test",
            example: Some("--url https://google.com"),
        };
        
        let formatted = option.format(false);
        assert!(formatted.contains("-u"));
        assert!(formatted.contains("--url"));
        assert!(formatted.contains("Target URL to test"));
        assert!(formatted.contains("Example: --url https://google.com"));
    }

    #[test]
    fn test_example_help_formatting() {
        let example = ExampleHelp {
            title: "Basic test",
            command: "network-latency-tester --url https://example.com",
            description: "Test a single URL",
        };
        
        let formatted = example.format(false);
        assert!(formatted.contains("Basic test"));
        assert!(formatted.contains("network-latency-tester --url https://example.com"));
        assert!(formatted.contains("Test a single URL"));
    }

    #[test]
    fn test_platform_specific_content() {
        let help_system = HelpSystem::new();
        
        let config_help = help_system.format_configuration_help(false);
        let timeout_help = help_system.format_timeout_help(false);
        let dns_help = help_system.format_dns_help(false);
        
        // Should contain platform-specific information
        assert!(config_help.contains(&help_system.platform));
        assert!(timeout_help.contains(&help_system.platform));
        
        // DNS help should contain platform-specific notes
        if help_system.platform == "Windows" {
            assert!(dns_help.contains("Windows"));
        } else if help_system.platform == "macOS" {
            assert!(dns_help.contains("macOS"));
        } else if help_system.platform == "Linux" {
            assert!(dns_help.contains("Linux"));
        }
    }

    #[test]
    fn test_color_formatting_differences() {
        let help_system = HelpSystem::new();
        
        let colored = help_system.display_main_help(true);
        let plain = help_system.display_main_help(false);
        
        // Both should contain essential content
        assert!(colored.contains("Network Latency Tester"));
        assert!(plain.contains("Network Latency Tester"));
        
        // Plain version should not contain ANSI escape codes
        let plain_has_ansi = plain.contains("\u{1b}[");
        assert!(!plain_has_ansi);
        
        // Colored version might or might not contain ANSI codes depending on colored crate behavior
        // Just verify that the colored version is either same or longer than plain
        assert!(colored.len() >= plain.len());
    }

    #[test]
    fn test_platform_edge_cases() {
        let help_system = HelpSystem::new();
        
        // Test that platform-specific help sections are complete
        let config_help = help_system.format_configuration_help(false);
        let timeout_help = help_system.format_timeout_help(false);
        let dns_help = help_system.format_dns_help(false);
        let output_help = help_system.format_output_help(false);
        
        // Platform-specific help topics should contain platform information
        assert!(config_help.contains(&help_system.platform));
        assert!(timeout_help.contains(&help_system.platform));
        assert!(dns_help.contains(&help_system.platform));
        
        // Output help doesn't contain platform info, so just check it's not empty
        assert!(!output_help.is_empty());
        assert!(output_help.contains("OUTPUT FORMATTING REFERENCE"));
        
        // Test that all help topics are not empty
        assert!(!config_help.is_empty());
        assert!(!timeout_help.is_empty());
        assert!(!dns_help.is_empty());
        
        // Test that examples work for all platforms
        let examples_help = help_system.format_examples_section(false);
        assert!(examples_help.contains("EXAMPLES"));
        assert!(examples_help.len() > 500); // Should be substantial content
        
        // Test unknown platform handling (defensive test)
        assert!(!help_system.platform.is_empty());
    }
}