//! Network Latency Tester - Main CLI Application
//! 
//! A high-performance network latency testing tool that measures connectivity
//! to configurable target URLs using various DNS configurations.

use clap::Parser;
use network_latency_tester::{
    cli::Cli,
    config::parser::load_config,
    client::ClientFactory,
    dns::DnsManager,
    executor::{ExecutionMode, create_executor_for_mode},
    output::{OutputFormatterFactory, OutputCoordinator},
    error::{AppError, Result},
    models::TestResult,
    types::DnsConfig,
    VERSION, PKG_NAME,
};
use std::{process, error::Error};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() {
    // Set up better panic handling
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Application panic: {}", panic_info);
        eprintln!("Please report this issue at: https://github.com/MaurUppi/network-latency-tester/issues");
        process::exit(1);
    }));

    // Parse command line arguments
    let cli = Cli::parse();

    // Handle the actual application logic
    if let Err(e) = run_application(cli).await {
        eprintln!("Error: {}", e);
        
        if let Some(source) = e.source() {
            eprintln!("Caused by: {}", source);
        }
        
        // Print suggestions for common errors
        print_error_suggestions(&e);
        
        process::exit(e.exit_code());
    }
}

/// Main application logic
async fn run_application(cli: Cli) -> Result<()> {
    // Show debug info if requested
    if cli.debug {
        println!("{} v{}", PKG_NAME, VERSION);
        println!("Debug mode enabled");
        println!();
    }

    // Load and validate configuration
    let config = load_config(cli.clone())?;
    
    if config.debug {
        println!("Configuration loaded successfully:");
        println!("  Target URLs: {}", config.target_urls.join(", "));
        println!("  DNS Servers: {}", config.dns_servers.join(", "));
        println!("  DoH Providers: {} configured", config.doh_providers.len());
        println!("  Test Count: {}", config.test_count);
        println!("  Timeout: {}s", config.timeout_seconds);
        println!("  Color Output: {}", config.enable_color);
        println!();
    }

    // Create DNS configurations
    let dns_configs = config.create_dns_configs()
        .map_err(|e| AppError::config(format!("Failed to create DNS configurations: {}", e)))?;

    if config.debug {
        println!("DNS Configurations ({}):", dns_configs.len());
        for (i, dns_config) in dns_configs.iter().enumerate() {
            println!("  {}: {}", i + 1, dns_config.name());
        }
        println!();
    }

    // Initialize core components
    let dns_manager = Arc::new(DnsManager::new()?);
    let _client_factory = ClientFactory::new(dns_manager.clone());

    // Create and configure test executor
    let executor = create_executor_for_mode(&config, ExecutionMode::Optimized).await?;

    if config.verbose || config.debug {
        println!("Starting network latency tests...");
        println!("Testing {} URLs with {} DNS configurations", 
            config.target_urls.len(), 
            dns_configs.len());
        println!();
    }

    // Execute tests
    let test_results = executor.execute_tests(&config.target_urls, &dns_configs).await?;
    
    // Convert to ExecutionResults structure
    let results = create_execution_results(test_results, &config.target_urls, &dns_configs);

    if config.debug {
        println!("Test execution completed:");
        println!("  Total tests: {}", results.execution_summary.total_tests);
        println!("  Successful tests: {}", results.execution_summary.successful_tests);
        println!("  Failed tests: {}", results.execution_summary.failed_tests);
        println!("  Success rate: {:.1}%", 
            if results.execution_summary.total_tests > 0 {
                (results.execution_summary.successful_tests as f64 / results.execution_summary.total_tests as f64) * 100.0
            } else {
                0.0
            });
        println!();
    }

    // Create output formatter and coordinator
    let formatter = OutputFormatterFactory::create_formatter(config.enable_color, config.verbose);
    let coordinator = OutputCoordinator::new(formatter);

    // Generate and display results  
    let output = coordinator.display_results(&results).await?;
    println!("{}", output);

    // Show additional information in verbose mode
    if config.verbose {
        println!();
        println!("{}", "=".repeat(80));
        println!("Test Summary:");
        println!("  Total configurations tested: {}", dns_configs.len());
        println!("  Total URLs tested: {}", config.target_urls.len());
        println!("  Total individual tests: {}", results.execution_summary.total_tests);
        let success_rate = if results.execution_summary.total_tests > 0 {
            (results.execution_summary.successful_tests as f64 / results.execution_summary.total_tests as f64) * 100.0
        } else {
            0.0
        };
        println!("  Overall success rate: {:.1}%", success_rate);
        
        if let Some(best_config) = results.best_config() {
            println!("  Best performing DNS: {}", best_config);
        }
    }

    // Return appropriate exit code
    let success_rate = if results.execution_summary.total_tests > 0 {
        results.execution_summary.successful_tests as f64 / results.execution_summary.total_tests as f64
    } else {
        0.0
    };
    
    if success_rate < 0.5 {
        Err(AppError::test_execution("More than 50% of tests failed - check network connectivity"))
    } else {
        Ok(())
    }
}

/// Print helpful suggestions for common errors
fn print_error_suggestions(error: &AppError) {
    match error {
        AppError::Config { .. } => {
            eprintln!();
            eprintln!("Configuration help:");
            eprintln!("  - Check your .env file format");
            eprintln!("  - Verify URL formats (must start with http:// or https://)");
            eprintln!("  - Ensure DNS server IPs are valid");
            eprintln!("  - DoH URLs must use HTTPS");
        },
        AppError::Network { .. } => {
            eprintln!();
            eprintln!("Network troubleshooting:");
            eprintln!("  - Check your internet connection");
            eprintln!("  - Try different DNS servers");
            eprintln!("  - Verify firewall settings");
            eprintln!("  - Test with a different target URL");
        },
        AppError::DnsResolution { .. } => {
            eprintln!();
            eprintln!("DNS resolution help:");
            eprintln!("  - Try using public DNS servers (8.8.8.8, 1.1.1.1)");
            eprintln!("  - Check if the domain exists");
            eprintln!("  - Test DNS resolution manually with 'nslookup' or 'dig'");
        },
        AppError::TestExecution { .. } => {
            eprintln!();
            eprintln!("Execution troubleshooting:");
            eprintln!("  - Increase timeout with --timeout option");
            eprintln!("  - Reduce test count with --count option");
            eprintln!("  - Check system resources");
        },
        _ => {}
    }
}

/// Convert test results into ExecutionResults structure
fn create_execution_results(test_results: Vec<TestResult>, _urls: &[String], _dns_configs: &[DnsConfig]) -> network_latency_tester::executor::ExecutionResults {
    use network_latency_tester::executor::{ExecutionResults, ExecutionSummary};
    use std::collections::HashMap;
    
    let total_tests = test_results.len() as u32;
    let successful_tests = test_results.iter().filter(|r| r.success_count > 0).count() as u32;
    let failed_tests = total_tests - successful_tests;
    
    let success_rate = if total_tests > 0 {
        (successful_tests as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };
    
    let execution_summary = ExecutionSummary {
        total_duration: std::time::Duration::from_secs(60), // Placeholder
        total_tests,
        successful_tests,
        failed_tests,
        timeout_tests: 0,
        skipped_tests: 0,
        success_rate,
        performance_summary: HashMap::new(),
    };
    
    // Convert test results to HashMap using unique identifiers
    let mut results_map = HashMap::new();
    for (i, result) in test_results.into_iter().enumerate() {
        results_map.insert(format!("test_{}", i), result);
    }
    
    ExecutionResults::new(execution_summary, results_map)
}
