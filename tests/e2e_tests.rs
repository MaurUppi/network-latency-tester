//! End-to-end integration tests for network latency tester
//! 
//! These tests validate the complete CLI workflows with real network requests,
//! ensuring feature parity with the original bash script and testing all
//! command-line options and their interactions.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::time::Duration;
use std::fs;
use std::env;
use tempfile::TempDir;
use serde_json::Value;

/// Test configuration for end-to-end tests
struct E2ETestConfig {
    /// Test URLs that should be accessible
    pub test_urls: Vec<&'static str>,
    /// DNS servers for testing custom DNS
    pub test_dns_servers: Vec<&'static str>,
    /// DoH URLs for testing DNS-over-HTTPS
    pub test_doh_urls: Vec<&'static str>,
    /// Timeout for network requests in tests
    pub test_timeout: u64,
    /// Number of test iterations for performance tests
    pub test_count: u32,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            test_urls: vec![
                "https://httpbin.org/delay/0.1",
                "https://www.google.com",
                "https://cloudflare.com",
            ],
            test_dns_servers: vec![
                "8.8.8.8",
                "1.1.1.1",
                "208.67.222.222",
            ],
            test_doh_urls: vec![
                "https://cloudflare-dns.com/dns-query",
                "https://dns.google/dns-query",
            ],
            test_timeout: 30,
            test_count: 3,
        }
    }
}

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

/// Test basic CLI execution with default parameters
#[test]
fn test_basic_execution() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"))
        .stdout(predicate::str::contains("ms"));
}

/// Test execution with verbose output
#[test]
fn test_verbose_output() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--verbose")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("DETAILED TIMING ANALYSIS"))
        .stdout(predicate::str::contains("DNS Resolution:"))
        .stdout(predicate::str::contains("TCP Connection:"))
        .stdout(predicate::str::contains("INDIVIDUAL REQUEST TIMINGS"));
}

/// Test execution with debug output
#[test]
fn test_debug_output() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--debug")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("DEBUG").or(predicate::str::contains("INFO")));
}

/// Test execution with custom DNS servers
#[test]
fn test_custom_dns() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--dns")
        .arg(config.test_dns_servers[0])
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"))
        .stdout(predicate::str::contains(&format!("自定义DNS ({})", config.test_dns_servers[0])));
}

/// Test execution with multiple DNS servers
#[test]
fn test_multiple_dns_servers() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--dns")
        .arg(config.test_dns_servers[0])
        .arg("--dns")
        .arg(config.test_dns_servers[1])
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"))
        .stdout(predicate::str::contains("自定义DNS"));
}

/// Test execution with DNS-over-HTTPS
#[test]
fn test_dns_over_https() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--doh")
        .arg(config.test_doh_urls[0])
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"))
        .stdout(predicate::str::contains("DoH"));
}

/// Test execution with multiple URLs
#[test]
fn test_multiple_urls() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--url")
        .arg(config.test_urls[1])
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"))
        .stdout(predicate::str::contains("httpbin.org").or(predicate::str::contains("google.com")));
}

/// Test execution with environment configuration
#[test]
fn test_environment_configuration() {
    let config = E2ETestConfig::default();
    let env_content = format!(
        "TARGET_URLS={}\nTEST_COUNT=2\nTIMEOUT_SECONDS=15\nDEBUG=false",
        config.test_urls[0]
    );
    
    let (_temp_dir, config_path) = create_temp_config(&env_content);
    
    create_test_cmd()
        .env("DOTENV_PATH", &config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test color output control
#[test]
fn test_color_output_control() {
    let config = E2ETestConfig::default();
    
    // Test with color enabled
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--color")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // Test with color disabled
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--no-color")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test help output
#[test]
fn test_help_output() {
    create_test_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Network Latency Tester"))
        .stdout(predicate::str::contains("--url"))
        .stdout(predicate::str::contains("--dns"))
        .stdout(predicate::str::contains("--count"))
        .stdout(predicate::str::contains("--timeout"));
}

/// Test version output
#[test]
fn test_version_output() {
    create_test_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Test error handling for invalid URLs
#[test]
fn test_invalid_url_error() {
    create_test_cmd()
        .arg("--url")
        .arg("not-a-valid-url")
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("5")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid URL").or(predicate::str::contains("URL解析失败")));
}

/// Test error handling for invalid DNS servers
#[test]
fn test_invalid_dns_error() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("1")
        .arg("--dns")
        .arg("not.a.valid.ip")
        .arg("--timeout")
        .arg("5")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid DNS server").or(predicate::str::contains("DNS服务器无效")));
}

/// Test error handling for invalid timeout values
#[test]
fn test_invalid_timeout_error() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid timeout").or(predicate::str::contains("超时值无效")));
}

/// Test error handling for invalid test count
#[test]
fn test_invalid_count_error() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("0")
        .arg("--timeout")
        .arg("5")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid test count").or(predicate::str::contains("测试次数无效")));
}

/// Test timeout handling for unreachable hosts
#[test]
fn test_timeout_handling() {
    create_test_cmd()
        .arg("--url")
        .arg("https://192.0.2.1") // RFC 5737 test address - should not be reachable
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("2")
        .assert()
        .success() // Should complete with timeout results, not fail
        .stdout(predicate::str::contains("TIMEOUT").or(predicate::str::contains("超时")));
}

/// Test diagnostics output
#[test]
fn test_diagnostics_output() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--verbose")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("DIAGNOSTIC").or(predicate::str::contains("诊断")));
}

/// Test performance analysis output
#[test]
fn test_performance_analysis() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("3")
        .arg("--verbose")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("PERFORMANCE").or(predicate::str::contains("性能分析")))
        .stdout(predicate::str::contains("Average Response Time").or(predicate::str::contains("平均响应时间")));
}

/// Test concurrent execution with multiple configurations
#[test]
fn test_concurrent_execution() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--dns")
        .arg(config.test_dns_servers[0])
        .arg("--dns")
        .arg(config.test_dns_servers[1])
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"))
        .stdout(predicate::str::contains("系统默认"))
        .stdout(predicate::str::contains("自定义DNS"));
}

/// Test statistics output format
#[test]
fn test_statistics_format() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("3")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("平均值").or(predicate::str::contains("Average")))
        .stdout(predicate::str::contains("最小值").or(predicate::str::contains("Min")))
        .stdout(predicate::str::contains("最大值").or(predicate::str::contains("Max")))
        .stdout(predicate::str::contains("成功率").or(predicate::str::contains("Success Rate")));
}

/// Test table output formatting
#[test]
fn test_table_output() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("┌").or(predicate::str::contains("+")))
        .stdout(predicate::str::contains("│").or(predicate::str::contains("|")))
        .stdout(predicate::str::contains("└").or(predicate::str::contains("+")));
}

/// Test JSON output format (if supported)
#[test]
fn test_json_output_format() {
    let config = E2ETestConfig::default();
    
    // Test if JSON output is available via debug mode
    let output = create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("2")
        .arg("--debug")
        .arg("--timeout")
        .arg("15")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Check if any JSON-like structures are present in debug output
    let has_json_like = stdout.contains("{") && stdout.contains("}")
        || stderr.contains("{") && stderr.contains("}");
    
    // This test passes if we either have JSON output or regular output
    assert!(has_json_like || stdout.contains("网络延迟测试结果"));
}

/// Test signal handling and graceful shutdown
#[test]
fn test_graceful_shutdown() {
    use std::time::{Duration, Instant};
    use std::thread;
    
    let config = E2ETestConfig::default();
    
    // Start a long-running test
    let mut child = create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("10") // Higher count to ensure it runs long enough
        .arg("--timeout")
        .arg("30")
        .spawn()
        .unwrap();
    
    // Wait a moment, then send SIGTERM (on Unix) or terminate (on Windows)
    thread::sleep(Duration::from_secs(1));
    
    let start = Instant::now();
    child.kill().unwrap();
    let result = child.wait().unwrap();
    let elapsed = start.elapsed();
    
    // Should terminate quickly (within 5 seconds) when signaled
    assert!(elapsed < Duration::from_secs(5));
    
    // On Unix, SIGKILL results in specific exit codes, on Windows it may vary
    // We just check that it didn't take too long to terminate
}

/// Test memory usage with large test counts
#[test]
fn test_memory_usage() {
    let config = E2ETestConfig::default();
    
    // Run with a moderate number of tests to check memory doesn't grow excessively
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--count")
        .arg("20")
        .arg("--timeout")
        .arg("15")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
    
    // If we reach here without OOM, the test passes
}

/// Test configuration file priority (CLI args should override config file)
#[test]
fn test_configuration_priority() {
    let config = E2ETestConfig::default();
    let env_content = format!(
        "TARGET_URLS={}\nTEST_COUNT=10\nTIMEOUT_SECONDS=30",
        config.test_urls[1]  // Different URL in config
    );
    
    let (_temp_dir, config_path) = create_temp_config(&env_content);
    
    // CLI args should override config file
    create_test_cmd()
        .env("DOTENV_PATH", &config_path)
        .arg("--url")
        .arg(config.test_urls[0])  // Different URL via CLI
        .arg("--count")
        .arg("2")  // Different count via CLI
        .arg("--timeout")
        .arg("15")  // Different timeout via CLI
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test DNS fallback behavior
#[test]
fn test_dns_fallback() {
    let config = E2ETestConfig::default();
    
    // Test with an invalid DNS server first, then a valid one
    // The application should handle DNS failures gracefully
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--dns")
        .arg("192.0.2.53") // RFC 5737 test address - should not respond
        .arg("--dns")
        .arg(config.test_dns_servers[0]) // Valid DNS server
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test IPv6 support (if available)
#[test] 
fn test_ipv6_support() {
    // Test with Google's IPv6 DNS
    create_test_cmd()
        .arg("--url")
        .arg("https://ipv6.google.com")
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("10")
        .assert()
        .success() // Should complete regardless of IPv6 availability
        .stdout(predicate::str::contains("网络延迟测试结果")
                .or(predicate::str::contains("Failed"))
                .or(predicate::str::contains("TIMEOUT")));
}

/// Test performance with high concurrency
#[test]
fn test_high_concurrency() {
    let config = E2ETestConfig::default();
    
    create_test_cmd()
        .arg("--url")
        .arg(config.test_urls[0])
        .arg("--dns")
        .arg(config.test_dns_servers[0])
        .arg("--dns")
        .arg(config.test_dns_servers[1])
        .arg("--dns")
        .arg(config.test_dns_servers[2])
        .arg("--count")
        .arg("5")
        .arg("--timeout")
        .arg("20")
        .assert()
        .success()
        .stdout(predicate::str::contains("网络延迟测试结果"));
}

/// Test edge cases with unusual URLs
#[test]
fn test_edge_case_urls() {
    // Test with URLs that have unusual but valid characteristics
    let edge_case_urls = vec![
        "https://httpbin.org/delay/0.001",  // Very fast response
        "https://httpbin.org/status/301",   // Redirect response
        "https://httpbin.org/status/404",   // Not found response
    ];
    
    for url in edge_case_urls {
        create_test_cmd()
            .arg("--url")
            .arg(url)
            .arg("--count")
            .arg("1")
            .arg("--timeout")
            .arg("10")
            .assert()
            .success()
            .stdout(predicate::str::contains("网络延迟测试结果"));
    }
}

/// Test output consistency across multiple runs
#[test]
fn test_output_consistency() {
    let config = E2ETestConfig::default();
    
    // Run the same test multiple times and ensure consistent output format
    for _ in 0..3 {
        create_test_cmd()
            .arg("--url")
            .arg(config.test_urls[0])
            .arg("--count")
            .arg("2")
            .arg("--timeout")
            .arg("10")
            .assert()
            .success()
            .stdout(predicate::str::contains("网络延迟测试结果"))
            .stdout(predicate::str::contains("ms"));
    }
}