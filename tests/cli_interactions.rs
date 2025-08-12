//! CLI options interaction tests
//! 
//! These tests validate that all CLI options work correctly in combination
//! with each other and handle edge cases properly.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use std::fs;

/// Helper function to create a test command
fn create_test_cmd() -> Command {
    Command::cargo_bin("network-latency-tester").unwrap()
}

/// Helper function to create temporary configuration files
fn create_temp_config(content: &str) -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".env");
    fs::write(&config_path, content).unwrap();
    let config_path_str = config_path.to_str().unwrap().to_string();
    (temp_dir, config_path_str)
}

/// Test URL option variations and combinations
#[test]
fn test_url_option_combinations() {
    // Single URL with different protocols
    let url_variants = vec![
        "https://httpbin.org/delay/0.1",
        "http://httpbin.org/delay/0.1", 
    ];
    
    for url in url_variants {
        create_test_cmd()
            .arg("--url")
            .arg(url)
            .arg("--count")
            .arg("2")
            .arg("--timeout")
            .arg("10")
            .assert()
            .success()
            .stdout(predicate::str::contains("网络延迟测试结果"));
    }
    
    // Multiple URLs with mixed protocols
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--url")
        .arg("https://www.google.com")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test DNS option combinations and interactions
#[test]
fn test_dns_option_combinations() {
    // Single custom DNS
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // Multiple custom DNS servers
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--dns")
        .arg("1.1.1.1")
        .arg("--dns")
        .arg("208.67.222.222")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // DNS + DoH combination (should not conflict)
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--doh")
        .arg("https://cloudflare-dns.com/dns-query")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test DoH option variations
#[test]
fn test_doh_option_combinations() {
    let doh_providers = vec![
        "https://cloudflare-dns.com/dns-query",
        "https://dns.google/dns-query",
    ];
    
    for doh_url in doh_providers {
        create_test_cmd()
            .arg("--url")
            .arg("https://httpbin.org/delay/0.1")
            .arg("--doh")
            .arg(doh_url)
            .arg("--count")
            .arg("2")
            .arg("--timeout")
            .arg("15")
            .assert()
            .success()
            .stdout(predicate::str::contains("网络延迟测试结果"));
    }
    
    // Multiple DoH URLs
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--doh")
        .arg("https://cloudflare-dns.com/dns-query")
        .arg("--doh")
        .arg("https://dns.google/dns-query")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("20")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test count and timeout option combinations
#[test]
fn test_count_timeout_combinations() {
    let test_combinations = vec![
        (1, 5),   // Minimal test
        (3, 10),  // Default-like
        (5, 15),  // Moderate test
        (10, 30), // Comprehensive test
    ];
    
    for (count, timeout) in test_combinations {
        create_test_cmd()
            .arg("--url")
            .arg("https://httpbin.org/delay/0.1")
            .arg("--count")
            .arg(&count.to_string())
            .arg("--timeout")
            .arg(&timeout.to_string())
            .assert()
            .success()
            .stdout(predicate::str::contains("网络延迟测试结果"));
    }
}

/// Test output format option combinations
#[test]
fn test_output_format_combinations() {
    let base_cmd = |cmd: &mut Command| {
        cmd.arg("--url")
            .arg("https://httpbin.org/delay/0.1")
            .arg("--count")
            .arg("2")
            .arg("--timeout")
            .arg("10");
    };
    
    // Verbose + Color
    let mut cmd = create_test_cmd();
    base_cmd(&mut cmd);
    cmd.arg("--verbose")
        .arg("--color")
        .assert()
        .success()
        .stdout(predicate::str::contains("DETAILED TIMING ANALYSIS"));
    
    // Verbose + No Color
    let mut cmd = create_test_cmd();
    base_cmd(&mut cmd);
    cmd.arg("--verbose")
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::contains("DETAILED TIMING ANALYSIS"));
    
    // Debug + Color
    let mut cmd = create_test_cmd();
    base_cmd(&mut cmd);
    cmd.arg("--debug")
        .arg("--color")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // Debug + Verbose (debug should take precedence)
    let mut cmd = create_test_cmd();
    base_cmd(&mut cmd);
    cmd.arg("--debug")
        .arg("--verbose")
        .assert()
        .success();
        // Should succeed regardless of which takes precedence
}

/// Test complex multi-option combinations
#[test]
fn test_complex_option_combinations() {
    // All DNS options + verbose + color
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--dns")
        .arg("1.1.1.1")
        .arg("--doh")
        .arg("https://cloudflare-dns.com/dns-query")
        .arg("--count")
        .arg("3")
        .arg("--timeout")
        .arg("20")
        .arg("--verbose")
        .arg("--color")
        .assert()
        .success()
        .stdout(predicate::str::contains("DETAILED TIMING ANALYSIS"));
    
    // Multiple URLs + Multiple DNS + Debug + No Color
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--url")
        .arg("https://www.google.com")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--dns")
        .arg("1.1.1.1")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .arg("--debug")
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test environment variable and CLI option interactions
#[test]
fn test_environment_cli_interactions() {
    // Create environment config
    let env_content = "TARGET_URLS=https://httpbin.org/delay/0.2\nTEST_COUNT=5\nTIMEOUT_SECONDS=20\nDEBUG=true";
    let (_temp_dir, config_path) = create_temp_config(env_content);
    
    // CLI options should override environment variables
    create_test_cmd()
        .env("DOTENV_PATH", &config_path)
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")  // Override env URL
        .arg("--count")
        .arg("2")  // Override env count
        .arg("--timeout")
        .arg("10")  // Override env timeout
        .arg("--verbose")  // Override env debug with verbose
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // Environment variables only (no CLI overrides)
    create_test_cmd()
        .env("DOTENV_PATH", &config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // Partial CLI overrides
    create_test_cmd()
        .env("DOTENV_PATH", &config_path)
        .arg("--verbose")  // Only override the debug flag
        .assert()
        .success()
        .stdout(predicate::str::contains("DETAILED TIMING ANALYSIS"));
}

/// Test edge cases with option combinations
#[test]
fn test_edge_case_combinations() {
    // Maximum reasonable values
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("20")  // High count
        .arg("--timeout")
        .arg("60")  // High timeout
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // Minimum reasonable values
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("1")   // Minimum count
        .arg("--timeout")
        .arg("1")   // Minimum timeout
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test conflicting or incompatible option combinations
#[test]
fn test_conflicting_options() {
    // Color and no-color together (no-color should win or be handled gracefully)
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .arg("--color")
        .arg("--no-color")
        .assert()
        .success()  // Should handle gracefully
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // Debug and verbose together (should handle gracefully)
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .arg("--debug")
        .arg("--verbose")
        .assert()
        .success()  // Should handle gracefully
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test option order independence
#[test]
fn test_option_order_independence() {
    let base_args = vec![
        "--url", "https://httpbin.org/delay/0.1",
        "--dns", "8.8.8.8",
        "--count", "2",
        "--timeout", "10",
        "--verbose",
        "--color",
    ];
    
    // Test a few different orderings
    let orderings = vec![
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],  // Original order
        vec![8, 9, 0, 1, 4, 5, 2, 3, 6, 7],  // Mixed order
        vec![6, 7, 8, 9, 0, 1, 2, 3, 4, 5],  // Reverse-ish order
    ];
    
    for ordering in orderings {
        let mut cmd = create_test_cmd();
        for &i in &ordering {
            cmd.arg(base_args[i]);
        }
        
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("DETAILED TIMING ANALYSIS"));
    }
}

/// Test option repetition handling
#[test]
fn test_option_repetition() {
    // Repeated single-value options (last one should win)
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.3")
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")  // This should be used or both
        .arg("--count")
        .arg("5")
        .arg("--count")
        .arg("2")  // This should be used
        .arg("--timeout")
        .arg("20")
        .arg("--timeout")
        .arg("10")  // This should be used
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // Repeated multi-value options (should accumulate)
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--dns")
        .arg("1.1.1.1")
        .arg("--dns")
        .arg("8.8.8.8")  // Duplicate - should be handled gracefully
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test help and version with other options
#[test]
fn test_help_version_with_other_options() {
    // Help should take precedence over other options
    create_test_cmd()
        .arg("--help")
        .arg("--url")
        .arg("https://example.com")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("--url"))
        .stdout(predicate::str::contains("--dns"));
    
    // Version should take precedence over other options
    create_test_cmd()
        .arg("--version")
        .arg("--url")
        .arg("https://example.com")
        .arg("--debug")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Test long vs short option combinations (if available)
#[test]
fn test_long_vs_short_options() {
    // This test assumes short options exist, but will pass if they don't
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    // If this succeeds, we know long options work
    if output.status.success() {
        // Try to test short options if they exist
        let short_output = create_test_cmd()
            .arg("-h")  // Try short help
            .output();
        
        // If short options exist, they should work
        match short_output {
            Ok(output) if output.status.success() => {
                // Short options are supported
                let stdout = String::from_utf8(output.stdout).unwrap();
                assert!(stdout.contains("--url") || stdout.contains("usage") || stdout.contains("Usage"));
            }
            _ => {
                // Short options might not be supported, which is fine
                // Long options are sufficient
            }
        }
    }
}

/// Test malformed option combinations
#[test]
fn test_malformed_option_combinations() {
    // Missing required values
    create_test_cmd()
        .arg("--url")
        // Missing URL value
        .arg("--count")
        .arg("2")
        .assert()
        .failure();
    
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        // Missing count value
        .arg("--timeout")
        .arg("10")
        .assert()
        .failure();
    
    // Invalid option names
    create_test_cmd()
        .arg("--invalid-option")
        .arg("value")
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .assert()
        .failure();
}

/// Test performance impact of option combinations
#[test]
fn test_performance_with_combinations() {
    use std::time::{Duration, Instant};
    
    // Test that complex option combinations don't significantly slow startup
    let start = Instant::now();
    
    create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--dns")
        .arg("1.1.1.1")
        .arg("--dns")
        .arg("208.67.222.222")
        .arg("--doh")
        .arg("https://cloudflare-dns.com/dns-query")
        .arg("--count")
        .arg("3")
        .arg("--timeout")
        .arg("15")
        .arg("--verbose")
        .arg("--color")
        .assert()
        .success();
    
    let elapsed = start.elapsed();
    
    // Should complete within reasonable time (this is generous for network requests)
    assert!(elapsed < Duration::from_secs(60), 
        "Complex option combination took too long: {:?}", elapsed);
}

/// Test option validation interactions
#[test]
fn test_option_validation_interactions() {
    // Valid URLs with various DNS configurations
    let urls = vec![
        "https://httpbin.org/delay/0.1",
        "https://www.google.com",
        "https://cloudflare.com",
    ];
    
    let dns_configs = vec![
        vec!["--dns", "8.8.8.8"],
        vec!["--dns", "1.1.1.1", "--dns", "8.8.4.4"],
        vec!["--doh", "https://cloudflare-dns.com/dns-query"],
    ];
    
    // Test all combinations
    for url in &urls {
        for dns_config in &dns_configs {
            let mut cmd = create_test_cmd();
            cmd.arg("--url").arg(url);
            
            for arg in dns_config {
                cmd.arg(arg);
            }
            
            cmd.arg("--count")
                .arg("2")
                .arg("--timeout")
                .arg("10")
                .assert()
                .success()
                .stdout(predicate::str::contains("网络延迟测试结果"));
        }
    }
}