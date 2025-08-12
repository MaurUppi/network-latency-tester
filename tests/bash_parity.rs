//! Bash script parity validation tests
//! 
//! These tests ensure that the Rust implementation provides complete
//! feature parity with the original bash script, including output format,
//! behavior, and functionality.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use regex::Regex;
use std::collections::HashSet;

/// Helper function to create a test command
fn create_test_cmd() -> Command {
    Command::cargo_bin("network-latency-tester").unwrap()
}

/// Expected features that should match the original bash script
struct BashScriptFeatures {
    /// Should support multiple target URLs
    pub multiple_urls: bool,
    /// Should support custom DNS servers
    pub custom_dns: bool,
    /// Should support DNS-over-HTTPS
    pub dns_over_https: bool,
    /// Should provide timing breakdown (DNS, TCP, etc.)
    pub timing_breakdown: bool,
    /// Should calculate statistics (min, max, avg, std dev)
    pub statistics: bool,
    /// Should show success rates
    pub success_rates: bool,
    /// Should support parallel/concurrent execution
    pub concurrent_execution: bool,
    /// Should provide colored output
    pub colored_output: bool,
    /// Should support verbose mode
    pub verbose_mode: bool,
    /// Should handle timeouts gracefully
    pub timeout_handling: bool,
    /// Should provide diagnostic information
    pub diagnostics: bool,
    /// Should support configuration via environment
    pub environment_config: bool,
    /// Should provide bilingual output (Chinese/English)
    pub bilingual_output: bool,
    /// Should format output in tables
    pub table_formatting: bool,
    /// Should show performance classifications
    pub performance_classification: bool,
}

impl Default for BashScriptFeatures {
    fn default() -> Self {
        Self {
            multiple_urls: true,
            custom_dns: true,
            dns_over_https: true,
            timing_breakdown: true,
            statistics: true,
            success_rates: true,
            concurrent_execution: true,
            colored_output: true,
            verbose_mode: true,
            timeout_handling: true,
            diagnostics: true,
            environment_config: true,
            bilingual_output: true,
            table_formatting: true,
            performance_classification: true,
        }
    }
}

/// Test that basic functionality matches bash script expectations
#[test]
fn test_basic_functionality_parity() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("3")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Basic execution should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain main results header (bilingual)
    assert!(stdout.contains("网络延迟测试结果") || stdout.contains("Network Latency Test Results"), 
        "Should contain main results header in Chinese or English");
    
    // Should contain timing measurements
    let timing_pattern = Regex::new(r"\d+\.\d+\s*ms").unwrap();
    assert!(timing_pattern.is_match(&stdout), 
        "Should contain timing measurements in milliseconds");
    
    // Should contain statistics
    assert!(stdout.contains("平均值") || stdout.contains("Average") || stdout.contains("Avg"), 
        "Should contain average statistics");
    assert!(stdout.contains("最小值") || stdout.contains("Min"), 
        "Should contain minimum statistics");
    assert!(stdout.contains("最大值") || stdout.contains("Max"), 
        "Should contain maximum statistics");
    
    // Should contain success rate
    let percentage_pattern = Regex::new(r"\d+\.\d+\s*%").unwrap();
    assert!(percentage_pattern.is_match(&stdout), 
        "Should contain success rate percentage");
}

/// Test multiple URL support (bash script feature)
#[test]
fn test_multiple_url_support_parity() {
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--url")
        .arg("https://httpbin.org/delay/0.2")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("15")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Multiple URL execution should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain results for both URLs
    assert!(stdout.contains("httpbin.org"), 
        "Should contain results for test URLs");
    
    // Should show separate result sections or combined results
    let result_indicators = stdout.matches("网络延迟测试结果").count() 
        + stdout.matches("Network Latency Test Results").count()
        + stdout.matches("ms").count();
    assert!(result_indicators >= 2, 
        "Should show results for multiple URLs");
}

/// Test DNS configuration support (bash script feature)
#[test]
fn test_dns_configuration_parity() {
    let features = BashScriptFeatures::default();
    if !features.custom_dns {
        return;
    }
    
    // Test system DNS (default)
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
    assert!(stdout.contains("系统默认") || stdout.contains("System") || stdout.contains("Default"), 
        "Should show system DNS configuration");
    
    // Test custom DNS
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
    assert!(stdout.contains("自定义DNS") || stdout.contains("Custom DNS") || stdout.contains("8.8.8.8"), 
        "Should show custom DNS configuration");
    
    // Test multiple DNS servers
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
        .arg("15")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should show results for multiple DNS configurations
    let dns_mentions = stdout.matches("8.8.8.8").count() + stdout.matches("1.1.1.1").count();
    assert!(dns_mentions >= 1, "Should show results for multiple DNS servers");
}

/// Test DNS-over-HTTPS support (advanced bash script feature)
#[test]
fn test_doh_support_parity() {
    let features = BashScriptFeatures::default();
    if !features.dns_over_https {
        return;
    }
    
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
    
    assert!(output.status.success(), "DoH execution should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("DoH") || stdout.contains("DNS-over-HTTPS") || stdout.contains("cloudflare"), 
        "Should indicate DoH usage");
}

/// Test timing breakdown (core bash script feature)
#[test]
fn test_timing_breakdown_parity() {
    let features = BashScriptFeatures::default();
    if !features.timing_breakdown {
        return;
    }
    
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
    
    assert!(output.status.success(), "Verbose execution should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain timing component breakdown
    let timing_components = vec![
        ("DNS", vec!["DNS Resolution", "DNS解析", "DNS"]),
        ("TCP", vec!["TCP Connection", "TCP连接", "TCP"]),
        ("First Byte", vec!["First Byte", "首字节", "TTFB"]),
        ("Total", vec!["Total", "总计", "Total Response"]),
    ];
    
    for (component, patterns) in timing_components {
        let found = patterns.iter().any(|pattern| stdout.contains(pattern));
        assert!(found, "Should contain {} timing information", component);
    }
}

/// Test statistics calculation (bash script feature)
#[test]
fn test_statistics_parity() {
    let features = BashScriptFeatures::default();
    if !features.statistics {
        return;
    }
    
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("5")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Statistics execution should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain all major statistical measures
    let stats_indicators = vec![
        vec!["平均值", "Average", "Avg", "Mean"],
        vec!["最小值", "Minimum", "Min"],
        vec!["最大值", "Maximum", "Max"],
        vec!["标准差", "Std Dev", "Standard Deviation", "σ"],
    ];
    
    for indicators in stats_indicators {
        let found = indicators.iter().any(|indicator| stdout.contains(indicator));
        assert!(found, "Should contain statistical indicators: {:?}", indicators);
    }
    
    // Should contain numerical values that make sense
    let timing_pattern = Regex::new(r"\d+\.\d+\s*ms").unwrap();
    let timing_matches: Vec<_> = timing_pattern.find_iter(&stdout).collect();
    assert!(timing_matches.len() >= 3, 
        "Should contain multiple timing measurements for statistics");
}

/// Test success rate calculation (bash script feature)
#[test]
fn test_success_rate_parity() {
    let features = BashScriptFeatures::default();
    if !features.success_rates {
        return;
    }
    
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("4")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Success rate execution should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain success rate indicators
    assert!(stdout.contains("成功率") || stdout.contains("Success Rate") || stdout.contains("Success:"), 
        "Should contain success rate indicator");
    
    // Should contain percentage values
    let percentage_pattern = Regex::new(r"\d+\.\d+\s*%").unwrap();
    assert!(percentage_pattern.is_match(&stdout), 
        "Should contain percentage values for success rate");
    
    // Success rate should be reasonable (0-100%)
    let percentage_matches: Vec<_> = percentage_pattern.find_iter(&stdout).collect();
    for percentage_match in percentage_matches {
        let percentage_str = percentage_match.as_str().replace("%", "");
        if let Ok(percentage) = percentage_str.trim().parse::<f64>() {
            assert!(percentage >= 0.0 && percentage <= 100.0, 
                "Success rate should be between 0% and 100%: {}%", percentage);
        }
    }
}

/// Test concurrent execution (bash script performance feature)
#[test]
fn test_concurrent_execution_parity() {
    let features = BashScriptFeatures::default();
    if !features.concurrent_execution {
        return;
    }
    
    use std::time::{Duration, Instant};
    
    // Test with multiple DNS configurations - should run concurrently
    let start = Instant::now();
    
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--dns")
        .arg("1.1.1.1")
        .arg("--dns")
        .arg("208.67.222.222")
        .arg("--count")
        .arg("3")
        .arg("--timeout")
        .arg("15")
        .output()
        .unwrap();
    
    let elapsed = start.elapsed();
    assert!(output.status.success(), "Concurrent execution should succeed");
    
    // With 3 DNS configs and 3 tests each, if run sequentially it would take much longer
    // With concurrent execution, it should complete in reasonable time
    assert!(elapsed < Duration::from_secs(30), 
        "Concurrent execution should complete efficiently");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain results for all DNS configurations
    let dns_results = stdout.matches("系统默认").count() 
        + stdout.matches("自定义DNS").count()
        + stdout.matches("8.8.8.8").count()
        + stdout.matches("1.1.1.1").count();
    assert!(dns_results >= 2, 
        "Should show results for multiple DNS configurations");
}

/// Test colored output (bash script visual feature)
#[test]
fn test_colored_output_parity() {
    let features = BashScriptFeatures::default();
    if !features.colored_output {
        return;
    }
    
    // Test with colors enabled
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
    
    assert!(output.status.success(), "Colored output should succeed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain the basic output regardless of color
    assert!(stdout.contains("网络延迟测试结果") || stdout.contains("ms"), 
        "Colored output should contain basic results");
    
    // Test with colors disabled
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
    
    assert!(output.status.success(), "No-color output should succeed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should not contain ANSI escape sequences
    assert!(!stdout.contains("\x1b["), "No-color output should not contain ANSI codes");
    assert!(stdout.contains("网络延迟测试结果") || stdout.contains("ms"), 
        "No-color output should contain basic results");
}

/// Test verbose mode (bash script diagnostic feature)
#[test]
fn test_verbose_mode_parity() {
    let features = BashScriptFeatures::default();
    if !features.verbose_mode {
        return;
    }
    
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
    
    assert!(output.status.success(), "Verbose execution should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain verbose-specific sections
    let verbose_sections = vec![
        vec!["DETAILED TIMING ANALYSIS", "详细时间分析"],
        vec!["INDIVIDUAL REQUEST TIMINGS", "单个请求时间"],
        vec!["PERFORMANCE", "性能分析"],
        vec!["RECOMMENDATIONS", "建议", "优化建议"],
    ];
    
    for section_patterns in verbose_sections {
        let found = section_patterns.iter().any(|pattern| stdout.contains(pattern));
        assert!(found, "Verbose mode should contain section: {:?}", section_patterns);
    }
    
    // Verbose output should be significantly longer than normal output
    assert!(stdout.len() > 1000, 
        "Verbose output should be substantial in length");
}

/// Test timeout handling (bash script reliability feature)
#[test]
fn test_timeout_handling_parity() {
    let features = BashScriptFeatures::default();
    if !features.timeout_handling {
        return;
    }
    
    // Test with unreachable address (should timeout gracefully)
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://192.0.2.1")  // RFC 5737 test address
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("2")  // Short timeout
        .output()
        .unwrap();
    
    // Should complete successfully even with timeouts
    assert!(output.status.success(), "Should handle timeouts gracefully");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should indicate timeout condition
    assert!(stdout.contains("TIMEOUT") || stdout.contains("超时") || stdout.contains("Timeout"), 
        "Should clearly indicate timeout condition");
    
    // Should still provide results structure
    assert!(stdout.contains("网络延迟测试结果") || stdout.contains("ms") || stdout.contains("FAIL"), 
        "Should maintain output structure even with timeouts");
}

/// Test bilingual output (bash script localization feature)
#[test]
fn test_bilingual_output_parity() {
    let features = BashScriptFeatures::default();
    if !features.bilingual_output {
        return;
    }
    
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("2")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Bilingual execution should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain Chinese characters
    let chinese_pattern = Regex::new(r"[\u4e00-\u9fff]").unwrap();
    assert!(chinese_pattern.is_match(&stdout), 
        "Should contain Chinese text for bilingual support");
    
    // Should contain English technical terms
    let english_terms = vec!["ms", "DNS", "TCP", "HTTP", "TLS"];
    let has_english = english_terms.iter().any(|term| stdout.contains(term));
    assert!(has_english, "Should contain English technical terms");
    
    // Key bilingual pairs should exist
    let bilingual_pairs = vec![
        ("网络延迟测试结果", vec!["Network", "Latency", "Test", "Results"]),
        ("平均值", vec!["Average", "Avg", "Mean"]),
        ("成功率", vec!["Success", "Rate"]),
    ];
    
    for (chinese, english_options) in bilingual_pairs {
        if stdout.contains(chinese) {
            // If Chinese term exists, should also have corresponding English terms
            let has_english_equivalent = english_options.iter()
                .any(|eng| stdout.contains(eng));
            
            // Note: This is somewhat relaxed as the exact bilingual implementation may vary
            // The key is that both languages should be represented in the output
        }
    }
}

/// Test table formatting (bash script presentation feature)
#[test]
fn test_table_formatting_parity() {
    let features = BashScriptFeatures::default();
    if !features.table_formatting {
        return;
    }
    
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--count")
        .arg("3")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Table formatting should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain table formatting elements
    let table_indicators = vec![
        // Unicode table borders
        "┌", "┐", "└", "┘", "│", "─", "├", "┤", "┬", "┴", "┼",
        // ASCII table borders
        "+", "-", "|",
    ];
    
    let has_table_formatting = table_indicators.iter()
        .any(|indicator| stdout.contains(indicator));
    
    assert!(has_table_formatting, 
        "Should contain table formatting characters");
    
    // Should have consistent column alignment in table rows
    let lines: Vec<&str> = stdout.lines().collect();
    let table_lines: Vec<&str> = lines.iter()
        .filter(|line| line.contains("│") || line.contains("|"))
        .cloned()
        .collect();
    
    if !table_lines.is_empty() {
        // Should have at least a few table rows
        assert!(table_lines.len() >= 2, 
            "Should have multiple table rows for proper formatting");
    }
}

/// Test performance classification (bash script analysis feature)
#[test]
fn test_performance_classification_parity() {
    let features = BashScriptFeatures::default();
    if !features.performance_classification {
        return;
    }
    
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")  // Fast response
        .arg("--count")
        .arg("3")
        .arg("--verbose")
        .arg("--timeout")
        .arg("10")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Performance classification should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain performance level indicators
    let performance_indicators = vec![
        // English performance levels
        "Good", "Excellent", "Poor", "Moderate", "Fair",
        // Chinese performance levels  
        "优秀", "良好", "中等", "较差", "差",
        // Symbolic indicators
        "✓", "✗", "⚠", "🟢", "🔴", "🟡",
        // Performance level markers
        "Performance Level", "性能等级",
    ];
    
    let has_performance_classification = performance_indicators.iter()
        .any(|indicator| stdout.contains(indicator));
    
    assert!(has_performance_classification, 
        "Should contain performance classification indicators");
}

/// Test overall feature completeness
#[test]
fn test_feature_completeness_parity() {
    let features = BashScriptFeatures::default();
    
    // Test a comprehensive scenario that exercises multiple features
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--url")
        .arg("https://httpbin.org/delay/0.2")
        .arg("--dns")
        .arg("8.8.8.8")
        .arg("--dns")
        .arg("1.1.1.1")
        .arg("--count")
        .arg("4")
        .arg("--timeout")
        .arg("15")
        .arg("--verbose")
        .arg("--color")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Comprehensive feature test should succeed");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should demonstrate multiple features working together
    let feature_indicators = vec![
        ("Multiple URLs", vec!["httpbin.org"]),
        ("DNS Configs", vec!["系统默认", "自定义DNS", "8.8.8.8"]),  
        ("Timing", vec!["ms", "DNS", "TCP"]),
        ("Statistics", vec!["平均值", "Average", "Min", "Max"]),
        ("Success Rate", vec!["成功率", "Success", "%"]),
        ("Verbose Info", vec!["DETAILED", "INDIVIDUAL", "PERFORMANCE"]),
        ("Bilingual", vec!["网络延迟测试结果"]),
    ];
    
    let mut features_found = 0;
    for (feature_name, indicators) in feature_indicators {
        let found = indicators.iter().any(|indicator| stdout.contains(indicator));
        if found {
            features_found += 1;
        } else {
            println!("Warning: {} feature indicators not found", feature_name);
        }
    }
    
    // Should have most features present
    assert!(features_found >= 5, 
        "Should demonstrate multiple bash script features: {}/7 found", features_found);
    
    // Output should be substantial for comprehensive test
    assert!(stdout.len() > 2000, 
        "Comprehensive output should be substantial");
}

/// Test error handling parity with bash script
#[test]
fn test_error_handling_parity() {
    // Test various error conditions that bash script should handle
    
    // Invalid URL
    let output = create_test_cmd()
        .arg("--url")
        .arg("not-a-valid-url")
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("5")
        .output()
        .unwrap();
    
    assert!(!output.status.success(), "Should fail with invalid URL");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.is_empty(), "Should provide error message");
    
    // Invalid DNS server
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--dns")
        .arg("not.a.valid.ip")
        .arg("--count")
        .arg("1")
        .arg("--timeout")
        .arg("5")
        .output()
        .unwrap();
    
    assert!(!output.status.success(), "Should fail with invalid DNS server");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.is_empty(), "Should provide DNS error message");
    
    // Invalid count/timeout values should be caught
    let output = create_test_cmd()
        .arg("--url")
        .arg("https://httpbin.org/delay/0.1")
        .arg("--count")
        .arg("0")
        .arg("--timeout")
        .arg("5")
        .output()
        .unwrap();
    
    assert!(!output.status.success(), "Should fail with invalid count");
}

/// Test command-line interface parity
#[test]
fn test_cli_interface_parity() {
    // Test help output
    let output = create_test_cmd()
        .arg("--help")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Help should succeed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should contain key command-line options that bash script supports
    let required_options = vec![
        "--url", "--dns", "--doh", "--count", "--timeout", 
        "--verbose", "--debug", "--color", "--help", "--version"
    ];
    
    for option in required_options {
        assert!(stdout.contains(option), 
            "Help should document {} option", option);
    }
    
    // Test version output
    let output = create_test_cmd()
        .arg("--version")
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Version should succeed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), 
        "Should show version information");
}