//! Output validation tests for network latency tester
//! 
//! These tests validate that the output format matches original bash script
//! expectations and ensures consistent formatting across different scenarios.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use regex::Regex;
use serde_json::Value;

/// Helper function to create a test command
fn create_test_cmd() -> Command {
    Command::cargo_bin("network-latency-tester").unwrap()
}

/// Validation patterns for different output formats
struct OutputPatterns {
    /// Pattern for timing values (e.g., "123.456ms")
    pub timing_pattern: Regex,
    /// Pattern for percentage values (e.g., "95.5%")
    pub percentage_pattern: Regex,
    /// Pattern for IP addresses
    pub ip_pattern: Regex,
    /// Pattern for DNS configuration names
    pub dns_config_pattern: Regex,
    /// Pattern for status indicators
    pub status_pattern: Regex,
    /// Pattern for table borders
    pub table_border_pattern: Regex,
    /// Pattern for Chinese characters (for bilingual support)
    pub chinese_pattern: Regex,
}

impl Default for OutputPatterns {
    fn default() -> Self {
        Self {
            timing_pattern: Regex::new(r"\d+\.\d+\s*ms").unwrap(),
            percentage_pattern: Regex::new(r"\d+\.\d+\s*%").unwrap(),
            ip_pattern: Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
            dns_config_pattern: Regex::new(r"(系统默认|自定义DNS|DoH)").unwrap(),
            status_pattern: Regex::new(r"(✓|✗|OK|FAIL|SUCCESS|FAILED|成功|失败)").unwrap(),
            table_border_pattern: Regex::new(r"[┌┐└┘│─├┤┬┴┼\+\-\|]").unwrap(),
            chinese_pattern: Regex::new(r"[\u4e00-\u9fff]").unwrap(),
        }
    }
}

/// Test that basic output contains required elements
#[test]
fn test_basic_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("3")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let patterns = OutputPatterns::default();
    
    // Should contain timing measurements
    assert!(patterns.timing_pattern.is_match(&stdout), 
        "Output should contain timing values in ms format");
    
    // Should contain percentage values (success rate)
    assert!(patterns.percentage_pattern.is_match(&stdout), 
        "Output should contain percentage values");
    
    // Should contain status indicators
    assert!(patterns.status_pattern.is_match(&stdout), 
        "Output should contain status indicators");
    
    // Should contain Chinese text (bilingual support)
    assert!(patterns.chinese_pattern.is_match(&stdout), 
        "Output should contain Chinese text for bilingual support");
}

/// Test table formatting consistency
#[test]
fn test_table_formatting() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let patterns = OutputPatterns::default();
    
    // Should contain table borders
    assert!(patterns.table_border_pattern.is_match(&stdout), 
        "Output should contain table formatting");
    
    // Check for consistent column alignment
    let lines: Vec<&str> = stdout.lines().collect();
    let table_lines: Vec<&str> = lines.iter()
        .filter(|line| line.contains("│") || line.contains("|"))
        .cloned()
        .collect();
    
    // If we have table lines, they should have consistent structure
    if !table_lines.is_empty() {
        let first_line_separators = table_lines[0].matches("│").count() 
            + table_lines[0].matches("|").count();
        
        for line in &table_lines {
            let separators = line.matches("│").count() + line.matches("|").count();
            if separators > 0 {  // Only check lines that are actually table rows
                assert_eq!(separators, first_line_separators, 
                    "All table rows should have consistent column count");
            }
        }
    }
}

/// Test verbose output format
#[test] 
fn test_verbose_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("3")
        .arg("--verbose")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Verbose output should contain detailed sections
    assert!(stdout.contains("DETAILED TIMING ANALYSIS") || stdout.contains("详细时间分析"), 
        "Verbose output should contain detailed timing analysis section");
    
    assert!(stdout.contains("DNS Resolution:") || stdout.contains("DNS解析:"), 
        "Verbose output should contain DNS resolution timing");
    
    assert!(stdout.contains("TCP Connection:") || stdout.contains("TCP连接:"), 
        "Verbose output should contain TCP connection timing");
    
    assert!(stdout.contains("INDIVIDUAL REQUEST TIMINGS") || stdout.contains("单个请求时间"), 
        "Verbose output should contain individual request timings");
    
    assert!(stdout.contains("PERFORMANCE") || stdout.contains("性能分析"), 
        "Verbose output should contain performance analysis");
    
    // Should contain timing recommendations
    assert!(stdout.contains("RECOMMENDATIONS") || stdout.contains("建议") || stdout.contains("优化建议"), 
        "Verbose output should contain optimization recommendations");
}

/// Test debug output format
#[test]
fn test_debug_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("2")
        .arg("--debug")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{}{}", stdout, stderr);
    
    // Debug output should contain log levels
    assert!(combined_output.contains("DEBUG") || combined_output.contains("INFO"), 
        "Debug output should contain debug log levels");
    
    // Should contain timestamps
    let timestamp_pattern = Regex::new(r"\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}").unwrap();
    assert!(timestamp_pattern.is_match(&combined_output), 
        "Debug output should contain timestamps");
    
    // May contain JSON structured logs
    if combined_output.contains("{") && combined_output.contains("}") {
        // Try to find and validate JSON structures
        let json_pattern = Regex::new(r"\{[^{}]*\}").unwrap();
        if let Some(json_match) = json_pattern.find(&combined_output) {
            let potential_json = json_match.as_str();
            // Try to parse as JSON to verify structure
            serde_json::from_str::<Value>(potential_json)
                .expect("JSON-like structures in debug output should be valid JSON");
        }
    }
}

/// Test error output format
#[test]
fn test_error_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("invalid-url-format")
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("5")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Error output should contain descriptive error message
    assert!(!stderr.is_empty(), "Error output should not be empty");
    
    // Should contain error category or type
    assert!(stderr.contains("Error") || stderr.contains("错误") || stderr.contains("失败"), 
        "Error output should contain error indicators");
    
    // Should provide useful information for troubleshooting
    assert!(stderr.len() > 20, "Error messages should be descriptive");
}

/// Test colored output format (when colors are enabled)
#[test]
fn test_colored_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("2")
        .arg("--color")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // When colors are enabled, output may contain ANSI escape sequences
    // But we should still have the basic content
    let patterns = OutputPatterns::default();
    assert!(patterns.timing_pattern.is_match(&stdout));
    assert!(patterns.chinese_pattern.is_match(&stdout));
}

/// Test plain (no-color) output format
#[test]
fn test_plain_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("2")
        .arg("--no-color")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Plain output should not contain ANSI escape sequences
    assert!(!stdout.contains("\x1b["), "Plain output should not contain ANSI escape codes");
    
    // But should still contain all required information
    let patterns = OutputPatterns::default();
    assert!(patterns.timing_pattern.is_match(&stdout));
    assert!(patterns.chinese_pattern.is_match(&stdout));
}

/// Test output format with multiple DNS configurations
#[test]
fn test_multiple_dns_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--dns")
        .arg("1.1.1.1")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let patterns = OutputPatterns::default();
    
    // Should contain multiple DNS configuration sections
    let dns_matches: Vec<_> = patterns.dns_config_pattern.find_iter(&stdout).collect();
    assert!(dns_matches.len() >= 2, "Should show results for multiple DNS configurations");
    
    // Should contain both DNS server references
    assert!(stdout.contains("8.8.8.8") || patterns.dns_config_pattern.is_match(&stdout));
    assert!(stdout.contains("1.1.1.1") || patterns.dns_config_pattern.is_match(&stdout));
}

/// Test output format with multiple URLs
#[test]
fn test_multiple_urls_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--url")
        .arg("https://httpbin.org/delay/0.2")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain results for both URLs
    assert!(stdout.contains("httpbin.org"), "Should show results for test URLs");
    
    // Should have multiple result sections
    let result_sections = stdout.matches("网络延迟测试结果").count() 
        + stdout.matches("Network Latency Test Results").count();
    assert!(result_sections >= 1, "Should contain result sections");
}

/// Test statistics output format and accuracy
#[test]
fn test_statistics_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("5")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let patterns = OutputPatterns::default();
    
    // Should contain statistical measures
    assert!(stdout.contains("平均值") || stdout.contains("Average") || stdout.contains("Avg"), 
        "Should contain average timing");
    
    assert!(stdout.contains("最小值") || stdout.contains("Min"), 
        "Should contain minimum timing");
    
    assert!(stdout.contains("最大值") || stdout.contains("Max"), 
        "Should contain maximum timing");
    
    assert!(stdout.contains("成功率") || stdout.contains("Success Rate"), 
        "Should contain success rate");
    
    // Extract timing values and verify they are reasonable
    let timing_matches: Vec<_> = patterns.timing_pattern.find_iter(&stdout).collect();
    assert!(!timing_matches.is_empty(), "Should contain timing measurements");
    
    // Verify percentage values are in valid range (0-100%)
    let percentage_matches: Vec<_> = patterns.percentage_pattern.find_iter(&stdout).collect();
    for percentage_match in percentage_matches {
        let percentage_str = percentage_match.as_str();
        let number_str = percentage_str.replace("%", "").trim().to_string();
        if let Ok(percentage) = number_str.parse::<f64>() {
            assert!(percentage >= 0.0 && percentage <= 100.0, 
                "Percentage values should be between 0 and 100");
        }
    }
}

/// Test output format consistency across different scenarios
#[test]
fn test_output_consistency() {
    let test_scenarios = vec![
        ("https://httpbin.org/delay/0.1", "8.8.8.8", 2),
        ("https://httpbin.org/delay/0.2", "1.1.1.1", 3),
        ("https://www.google.com", "208.67.222.222", 2),
    ];
    
    let patterns = OutputPatterns::default();
    
    for (url, dns, count) in test_scenarios {
        let output = create_test_cmd()
            .arg("--url")
            .arg(url)
            .arg("--dns")
            .arg(dns)
            .arg("--count")
            .arg(&count.to_string())
            .arg("--timeout")
            .arg("10")
            .output()
            .unwrap();
        
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout).unwrap();
            
            // Each successful run should have consistent format elements
            assert!(patterns.timing_pattern.is_match(&stdout), 
                "All successful runs should contain timing values");
            
            assert!(patterns.chinese_pattern.is_match(&stdout), 
                "All runs should contain Chinese text");
            
            assert!(patterns.status_pattern.is_match(&stdout) || 
                   stdout.contains("ms"), 
                "All runs should contain status or timing indicators");
        }
    }
}

/// Test timeout output format
#[test]
fn test_timeout_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://192.0.2.1") // RFC 5737 test address - should timeout
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("2")
        .output()
        .unwrap();
    
    // Should complete successfully even with timeouts
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should indicate timeout condition
    assert!(stdout.contains("TIMEOUT") || stdout.contains("超时") || stdout.contains("Timeout"), 
        "Timeout scenarios should be clearly indicated");
    
    // Should still maintain proper table format
    let patterns = OutputPatterns::default();
    assert!(patterns.table_border_pattern.is_match(&stdout) || stdout.contains("ms"), 
        "Even timeout results should maintain proper formatting");
}

/// Test DNS over HTTPS output format
#[test]
fn test_doh_output_format() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--doh")
        .arg("https://cloudflare-dns.com/dns-query")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should indicate DoH is being used
    assert!(stdout.contains("DoH") || stdout.contains("DNS-over-HTTPS"), 
        "DoH configuration should be indicated in output");
    
    // Should contain cloudflare or indicate the DoH provider
    assert!(stdout.contains("cloudflare") || stdout.contains("DoH (cloudflare-dns.com)"), 
        "Should show DoH provider information");
}

/// Test output format with very fast and very slow responses
#[test] 
fn test_extreme_timing_output_format() {
    // Test very fast response
    let fast_output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.001")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    if fast_output.status.success() {
        let stdout = String::from_utf8(fast_output.stdout).unwrap();
        let patterns = OutputPatterns::default();
        
        // Should handle very small timing values properly
        assert!(patterns.timing_pattern.is_match(&stdout));
        
        // Fast responses should be marked as good performance
        assert!(stdout.contains("Good") || stdout.contains("优秀") || stdout.contains("✓"));
    }
    
    // Test slower response
    let slow_output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/1")
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    if slow_output.status.success() {
        let stdout = String::from_utf8(slow_output.stdout).unwrap();
        let patterns = OutputPatterns::default();
        
        // Should handle larger timing values properly
        assert!(patterns.timing_pattern.is_match(&stdout));
        
        // Should still provide meaningful results
        assert!(stdout.contains("ms"));
    }
}

/// Test bilingual output consistency
#[test]
fn test_bilingual_output() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let patterns = OutputPatterns::default();
    
    // Should contain both Chinese and English elements
    assert!(patterns.chinese_pattern.is_match(&stdout), 
        "Should contain Chinese text");
    
    // Should contain English technical terms or values
    assert!(stdout.contains("ms") || stdout.contains("DNS") || stdout.contains("TCP"), 
        "Should contain English technical terms");
    
    // Key result sections should be present
    assert!(stdout.contains("网络延迟测试结果") || stdout.contains("Network Latency Test Results"), 
        "Should contain main results header");
}

/// Test output format with connection failures
#[test]
fn test_failure_output_format() {
    // Test with a domain that should not resolve
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://this-domain-should-not-exist-12345.com")
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("5")
        .output()
        .unwrap();
    
    // Should complete successfully even with failures
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should indicate failure condition clearly
    assert!(stdout.contains("FAILED") || stdout.contains("失败") || stdout.contains("✗") || stdout.contains("FAIL"), 
        "Connection failures should be clearly indicated");
    
    // Should still maintain proper format structure
    assert!(!stdout.is_empty(), "Should produce output even for failures");
}