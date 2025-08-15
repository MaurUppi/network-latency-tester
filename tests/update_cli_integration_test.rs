//! CLI integration tests for update parameter handling
//!
//! This module provides comprehensive integration tests for update-related CLI parameters,
//! including parameter parsing, validation, error handling, and short/long parameter equivalence.
//! Tests verify CLI integration works correctly with all update parameters and combinations.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

/// Helper function to create a test command
fn create_test_cmd() -> Command {
    Command::cargo_bin("network-latency-tester").unwrap()
}

/// Test basic update parameter parsing and functionality
#[test]
fn test_update_parameter_parsing() {
    // Test short form update parameter
    create_test_cmd()
        .arg("-u")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Test long form update parameter
    create_test_cmd()
        .arg("--update")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));
}

/// Test version parameter with update (short and long forms)
#[test]
fn test_version_parameter_equivalence() {
    // Test short form: -u -v
    create_test_cmd()
        .arg("-u")
        .arg("-v")
        .arg("1.2.3")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test long form: --update --version
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("1.2.3")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test mixed form: -u --version
    create_test_cmd()
        .arg("-u")
        .arg("--version")
        .arg("v1.2.3")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test mixed form: --update -v
    create_test_cmd()
        .arg("--update")
        .arg("-v")
        .arg("0.1.9")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test force parameter with update (short and long forms)
#[test]
fn test_force_parameter_equivalence() {
    // Test short form: -u -v -f
    create_test_cmd()
        .arg("-u")
        .arg("-v")
        .arg("0.1.5")
        .arg("-f")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test long form: --update --version --force
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("0.1.5")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test mixed forms
    create_test_cmd()
        .arg("-u")
        .arg("--version")
        .arg("v0.1.5")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    create_test_cmd()
        .arg("--update")
        .arg("-v")
        .arg("0.1.5")
        .arg("-f")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test various version format handling
#[test]
fn test_version_format_handling() {
    let valid_versions = vec![
        "1.0.0",
        "v1.0.0",
        "0.1.9",
        "v0.1.9",
        "2.5.10",
        "v2.5.10",
        "1.0.0-alpha",
        "v1.0.0-alpha.1",
        "1.2.3-beta.4",
    ];

    for version in valid_versions {
        create_test_cmd()
            .arg("--update")
            .arg("--version")
            .arg(version)
            .assert()
            .success()
            .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
    }
}

/// Test invalid version format error handling
#[test]
fn test_invalid_version_format_errors() {
    let invalid_versions = vec![
        "invalid",
        "1.2",
        "1",
        "v1.2",
        "1.2.3.4",
        "1.2.3.4.5",
        "",
        "v",
        "1..2",
        "1.2.",
        ".1.2",
        "1.2.a",
        "a.b.c",
    ];

    for version in invalid_versions {
        create_test_cmd()
            .arg("--update")
            .arg("--version")
            .arg(version)
            .assert()
            .failure()
            .stderr(predicate::str::contains("版本格式").or(predicate::str::contains("version format")));
    }
}

/// Test parameter dependency validation
#[test]
fn test_parameter_dependency_validation() {
    // Test --version requires --update
    create_test_cmd()
        .arg("--version")
        .arg("1.2.3")
        .assert()
        .failure(); // Should fail because --version requires --update

    create_test_cmd()
        .arg("-v")
        .arg("1.2.3")
        .assert()
        .failure(); // Should fail because -v requires -u or --update

    // Test --force requires --update
    create_test_cmd()
        .arg("--force")
        .assert()
        .failure(); // Should fail because --force requires --update

    create_test_cmd()
        .arg("-f")
        .assert()
        .failure(); // Should fail because -f requires -u or --update

    // Test --force requires --version (contextually)
    create_test_cmd()
        .arg("--update")
        .arg("--force")
        .assert()
        .success(); // Should succeed (interactive mode with force is valid)
}

/// Test parameter order independence
#[test]
fn test_parameter_order_independence() {
    let test_cases = vec![
        // Different orderings of the same parameters
        vec!["--update", "--version", "1.2.3", "--force"],
        vec!["--version", "1.2.3", "--update", "--force"],
        vec!["--force", "--update", "--version", "1.2.3"],
        vec!["--version", "1.2.3", "--force", "--update"],
        vec!["-u", "-v", "1.2.3", "-f"],
        vec!["-v", "1.2.3", "-u", "-f"],
        vec!["-f", "-u", "-v", "1.2.3"],
        vec!["-v", "1.2.3", "-f", "-u"],
    ];

    for args in test_cases {
        create_test_cmd()
            .args(&args)
            .assert()
            .success()
            .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
    }
}

/// Test update mode vs normal mode behavior
#[test]
fn test_update_mode_vs_normal_mode() {
    // Normal mode requires URLs
    create_test_cmd()
        .arg("--count")
        .arg("3")
        .arg("--timeout")
        .arg("10")
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL").or(predicate::str::contains("url")));

    // Update mode doesn't require URLs
    create_test_cmd()
        .arg("--update")
        .arg("--count")
        .arg("3")
        .arg("--timeout")
        .arg("10")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Update mode with version doesn't require URLs
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("1.2.3")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test update parameters with other CLI options
#[test]
fn test_update_with_other_options() {
    // Test update with verbose output
    create_test_cmd()
        .arg("--update")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Test update with debug output
    create_test_cmd()
        .arg("--update")
        .arg("--debug")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Test update with color options
    create_test_cmd()
        .arg("--update")
        .arg("--color")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    create_test_cmd()
        .arg("--update")
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Test version update with verbose and color
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("1.2.3")
        .arg("--verbose")
        .arg("--color")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test complex update parameter combinations
#[test]
fn test_complex_update_combinations() {
    // Test all update flags together with other options
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("v1.2.3")
        .arg("--force")
        .arg("--verbose")
        .arg("--debug")
        .arg("--color")
        .arg("--count")
        .arg("5")
        .arg("--timeout")
        .arg("30")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test with short forms
    create_test_cmd()
        .arg("-u")
        .arg("-v")
        .arg("0.1.9")
        .arg("-f")
        .arg("--verbose")
        .arg("--no-color")
        .arg("-c")
        .arg("3")
        .arg("-t")
        .arg("20")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test edge cases and boundary conditions
#[test]
fn test_edge_cases() {
    // Test minimum valid version
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("0.0.1")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test high version numbers
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("999.999.999")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test long version identifiers
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("v1.0.0-alpha.beta.gamma.1.2.3")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test conflicting parameter combinations
#[test]
fn test_conflicting_combinations() {
    // Test color conflicts with update (should handle gracefully)
    create_test_cmd()
        .arg("--update")
        .arg("--color")
        .arg("--no-color")
        .assert()
        .failure() // Should fail due to conflicting color options
        .stderr(predicate::str::contains("color"));

    // Test debug and verbose together with update (should handle gracefully)
    create_test_cmd()
        .arg("--update")
        .arg("--debug")
        .arg("--verbose")
        .assert()
        .success(); // Should succeed, debug or verbose takes precedence
}

/// Test parameter repetition handling
#[test]
fn test_parameter_repetition() {
    // Test repeated update flag (should handle gracefully)
    create_test_cmd()
        .arg("--update")
        .arg("--update")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Test repeated version (last one should win or error appropriately)
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("1.0.0")
        .arg("--version")
        .arg("2.0.0")
        .assert()
        .success() // Clap handles multiple values appropriately
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test repeated force flag
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("1.0.0")
        .arg("--force")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test help interaction with update parameters
#[test]
fn test_help_with_update_parameters() {
    // Help should take precedence over update parameters
    create_test_cmd()
        .arg("--help")
        .arg("--update")
        .arg("--version")
        .arg("1.2.3")
        .assert()
        .success()
        .stdout(predicate::str::contains("--update"))
        .stdout(predicate::str::contains("--version"))
        .stdout(predicate::str::contains("--force"));

    // Version flag should take precedence
    create_test_cmd()
        .arg("--version")
        .arg("--update")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Test malformed update parameter combinations
#[test]
fn test_malformed_update_combinations() {
    // Missing version value
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        // Missing version value
        .arg("--force")
        .assert()
        .failure();

    // Invalid flag combinations
    create_test_cmd()
        .arg("--invalid-flag")
        .arg("--update")
        .assert()
        .failure();

    // Version without value
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .assert()
        .failure();
}

/// Test interactive update mode
#[test]
fn test_interactive_update_mode() {
    // Interactive update (no version specified)
    create_test_cmd()
        .arg("--update")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Interactive update with short form
    create_test_cmd()
        .arg("-u")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Interactive update with other options
    create_test_cmd()
        .arg("--update")
        .arg("--verbose")
        .arg("--color")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));
}

/// Test update parameter validation with boundary values
#[test]
fn test_boundary_value_validation() {
    // Test version with zeros
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("0.0.0")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Test very long version strings
    let long_version = "v1.2.3-very.long.prerelease.identifier.with.many.segments.alpha.beta.gamma.delta.epsilon";
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg(long_version)
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test update parameter case sensitivity
#[test]
fn test_case_sensitivity() {
    // Parameters should be case sensitive (lowercase required)
    create_test_cmd()
        .arg("--UPDATE")
        .assert()
        .failure(); // Should fail because UPDATE != update

    create_test_cmd()
        .arg("--VERSION")
        .arg("1.2.3")
        .assert()
        .failure(); // Should fail because VERSION != version

    create_test_cmd()
        .arg("--FORCE")
        .assert()
        .failure(); // Should fail because FORCE != force

    // But version values should be handled appropriately
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("V1.2.3") // Version value with capital V
        .assert()
        .success() // Should succeed, version parsing handles this
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test performance of complex update parameter combinations
#[test]
fn test_performance_with_update_combinations() {
    use std::time::{Duration, Instant};

    let start = Instant::now();

    // Test complex parameter combination doesn't slow startup significantly
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("v1.2.3-alpha.beta.gamma.1")
        .arg("--force")
        .arg("--verbose")
        .arg("--debug")
        .arg("--color")
        .arg("--count")
        .arg("10")
        .arg("--timeout")
        .arg("30")
        .assert()
        .success();

    let elapsed = start.elapsed();

    // Should complete CLI parsing and update check within reasonable time
    assert!(elapsed < Duration::from_secs(30), 
        "Complex update parameter combination took too long: {:?}", elapsed);
}

/// Test update parameter validation across different scenarios
#[test]
fn test_comprehensive_validation_scenarios() {
    // Scenario 1: Valid interactive update
    create_test_cmd()
        .arg("--update")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Scenario 2: Valid version update
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("1.2.3")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Scenario 3: Valid forced downgrade
    create_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("0.1.0")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));

    // Scenario 4: Mixed parameter forms
    create_test_cmd()
        .arg("-u")
        .arg("--version")
        .arg("v2.0.0")
        .arg("-f")
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}

/// Test environment interaction with update parameters
#[test]
fn test_environment_interaction() {
    // Update mode should work regardless of environment variables
    // that might affect normal URL validation
    create_test_cmd()
        .env("TARGET_URLS", "https://example.com") // Normally would be used
        .arg("--update")
        .assert()
        .success()
        .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update")));

    // Update with version should override any environment config
    create_test_cmd()
        .env("DEBUG", "true")
        .env("VERBOSE", "true")
        .arg("--update")
        .arg("--version")
        .arg("1.2.3")
        .arg("--no-color") // Should override any color environment settings
        .assert()
        .success()
        .stdout(predicate::str::contains("版本").or(predicate::str::contains("version")));
}