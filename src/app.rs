//! Main application orchestration and execution

use crate::{
    cli::Cli,
    config::{load_config, validate_config, display_config_summary},
    error::Result,
};

/// Main application struct that coordinates all components
pub struct App {
    cli: Cli,
}

impl App {
    /// Create a new application instance with CLI configuration
    pub fn new(cli: Cli) -> Result<Self> {
        Ok(Self { cli })
    }

    /// Run the application
    pub async fn run(self) -> Result<()> {
        println!("Network Latency Tester v{}", crate::VERSION);
        
        // Load and validate configuration
        println!("Loading configuration...");
        let config = load_config(self.cli.clone())?;
        
        // Validate configuration with warnings
        let warnings = validate_config(&config)?;
        
        if config.debug {
            println!("Debug mode enabled");
            println!("\nConfiguration Summary:");
            println!("{}", display_config_summary(&config));
        }
        
        // Display validation warnings
        if !warnings.is_empty() {
            println!("\nConfiguration Warnings:");
            for warning in &warnings {
                println!("  {}", warning.format(config.enable_color));
            }
        }
        
        println!("\nStarting network latency tests...");
        
        // Display what will be tested
        println!("Target URLs: {}", config.target_urls.join(", "));
        println!("DNS configurations: {} (System + {} custom + {} DoH)", 
                1 + config.dns_servers.len() + config.doh_providers.len(),
                config.dns_servers.len(), 
                config.doh_providers.len());
        println!("Test iterations per configuration: {}", config.test_count);
        println!("Request timeout: {}s", config.timeout_seconds);
        
        // TODO: This is a placeholder - actual testing will be implemented in future tasks
        println!("\n✅ Configuration loaded and validated successfully!");
        println!("Ready for HTTP client and test execution implementation.");
        
        if config.verbose {
            println!("\nVerbose mode - detailed statistics will be displayed");
        }

        Ok(())
    }
}