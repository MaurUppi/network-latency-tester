//! Complete update flow integration tests
//!
//! This module provides comprehensive end-to-end integration tests for the entire update 
//! workflow, including upgrade scenarios, downgrade handling, version targeting, interactive
//! selection, and error recovery mechanisms. Tests verify that the complete update system
//! functions correctly with all components working together.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::time::Duration;

/// Helper function to create a test command for update flow testing
fn create_update_test_cmd() -> Command {
    Command::cargo_bin("nlt").unwrap()
}

/// Test helper to create predicate for update mode activation
fn update_mode_predicate() -> impl predicates::Predicate<str> {
    predicate::str::contains("Starting update operation")
        .or(predicate::str::contains("检查更新"))
        .or(predicate::str::contains("Update mode"))
        .or(predicate::str::contains("更新模式"))
        .or(predicate::str::contains("Already up to date"))
        .or(predicate::str::contains("version"))
}

/// Test helper to create predicate for version-related output
fn version_output_predicate() -> impl predicates::Predicate<str> {
    predicate::str::contains("版本")
        .or(predicate::str::contains("version"))
        .or(predicate::str::contains("Already up to date"))
}

// ========== BASIC UPDATE FLOW INTEGRATION TESTS ==========

/// Test basic update mode activation with --update flag
#[test]
fn test_update_mode_activation() {
    create_update_test_cmd()
        .arg("--update")
        .assert()
        .success()
        .stdout(update_mode_predicate());
}

/// Test update mode activation with short flag
#[test]
fn test_update_mode_activation_short_flag() {
    create_update_test_cmd()
        .arg("-u")
        .assert()
        .success()
        .stdout(update_mode_predicate());
}

/// Test that update mode bypasses normal URL requirements
#[test]
fn test_update_mode_bypasses_url_requirements() {
    // Normal mode should fail without URLs
    create_update_test_cmd()
        .arg("--count")
        .arg("1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL").or(predicate::str::contains("url")));
    
    // Update mode should succeed without URLs
    create_update_test_cmd()
        .arg("--update")
        .arg("--count")
        .arg("1") // These params should be ignored in update mode
        .assert()
        .success()
        .stdout(update_mode_predicate());
}

// ========== VERSION TARGETING INTEGRATION TESTS ==========

/// Test targeting a specific version with update mode
#[test]
fn test_version_targeting_basic() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("1.0.0")
        .assert()
        .success()
        .stdout(version_output_predicate());
}

/// Test version targeting with v prefix
#[test]
fn test_version_targeting_with_prefix() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("v1.0.0")
        .assert()
        .success()
        .stdout(version_output_predicate());
}

/// Test version targeting with prerelease versions
#[test]
fn test_version_targeting_prerelease() {
    let prerelease_versions = vec![
        "1.0.0-alpha",
        "v1.0.0-beta.1",
        "2.1.0-rc.2",
        "v1.2.3-alpha.beta.gamma",
    ];
    
    for version in prerelease_versions {
        create_update_test_cmd()
            .arg("--update")
            .arg("--version")
            .arg(version)
            .assert()
            .success()
            .stdout(version_output_predicate());
    }
}

/// Test version targeting with complex version formats
#[test]
fn test_version_targeting_complex_formats() {
    let complex_versions = vec![
        "0.1.9",
        "v0.1.9",
        "10.20.30",
        "0.0.1",
        "999.999.999",
        "1.0.0-alpha.1",
        "v2.0.0-beta.2.test",
    ];
    
    for version in complex_versions {
        create_update_test_cmd()
            .arg("--update")
            .arg("--version")
            .arg(version)
            .assert()
            .success()
            .stdout(version_output_predicate());
    }
}

// ========== FORCE DOWNGRADE INTEGRATION TESTS ==========

/// Test force downgrade functionality
#[test]
fn test_force_downgrade_basic() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("0.1.0") // Likely older than current version
        .arg("--force")
        .assert()
        .success()
        .stdout(version_output_predicate());
}

/// Test force downgrade with short flags
#[test]
fn test_force_downgrade_short_flags() {
    create_update_test_cmd()
        .arg("-u")
        .arg("-v")
        .arg("0.1.0")
        .arg("-f")
        .assert()
        .success()
        .stdout(version_output_predicate());
}

/// Test force downgrade with various version formats
#[test]
fn test_force_downgrade_version_formats() {
    let downgrade_versions = vec![
        "0.1.0",
        "v0.1.0",
        "0.0.1",
        "v0.0.1",
        "0.1.0-beta",
        "v0.1.0-alpha.1",
    ];
    
    for version in downgrade_versions {
        create_update_test_cmd()
            .arg("--update")
            .arg("--version")
            .arg(version)
            .arg("--force")
            .assert()
            .success()
            .stdout(version_output_predicate());
    }
}

// ========== INTERACTIVE MODE INTEGRATION TESTS ==========

/// Test interactive update mode (no version specified)
#[test]
fn test_interactive_update_mode() {
    create_update_test_cmd()
        .arg("--update")
        .assert()
        .success()
        .stdout(update_mode_predicate());
}

/// Test interactive mode with verbose output
#[test]
fn test_interactive_mode_verbose() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(update_mode_predicate());
}

/// Test interactive mode with colored output
#[test]
fn test_interactive_mode_colored() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--color")
        .assert()
        .success()
        .stdout(update_mode_predicate());
}

/// Test interactive mode with no color
#[test]
fn test_interactive_mode_no_color() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--no-color")
        .assert()
        .success()
        .stdout(update_mode_predicate());
}

// ========== ERROR HANDLING AND VALIDATION TESTS ==========

/// Test invalid version format error handling
#[test]
fn test_invalid_version_format_errors() {
    let invalid_versions = vec![
        "invalid",
        "1.2",
        "1",
        "v1.2",
        "1.2.3.4",
        "",
        "v",
        "1..2",
        "1.2.",
        ".1.2",
        "1.2.a",
        "a.b.c",
        "1.2.3.4.5",
        "not-a-version",
        "1.2.3-",
        "v1.2.3-",
    ];
    
    for version in invalid_versions {
        create_update_test_cmd()
            .arg("--update")
            .arg("--version")
            .arg(version)
            .assert()
            .failure()
            .stderr(predicate::str::contains("版本格式").or(predicate::str::contains("version format"))
                .or(predicate::str::contains("Invalid version")));
    }
}

/// Test CLI argument validation for update mode
#[test]
fn test_update_argument_validation() {
    // Version without update should fail
    create_update_test_cmd()
        .arg("--version")
        .arg("1.0.0")
        .assert()
        .failure();
    
    // Force without update should fail
    create_update_test_cmd()
        .arg("--force")
        .assert()
        .failure();
    
    // Version and force without update should fail
    create_update_test_cmd()
        .arg("--version")
        .arg("1.0.0")
        .arg("--force")
        .assert()
        .failure();
}

/// Test conflicting options in update mode
#[test]
fn test_conflicting_options_update_mode() {
    // Color conflicts should be handled
    create_update_test_cmd()
        .arg("--update")
        .arg("--color")
        .arg("--no-color")
        .assert()
        .failure()
        .stderr(predicate::str::contains("color"));
    
    // Debug and verbose together (should be handled gracefully)
    create_update_test_cmd()
        .arg("--update")
        .arg("--debug")
        .arg("--verbose")
        .assert()
        .success();
}

/// Test version without value error
#[test]
fn test_version_without_value_error() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        // Missing version value
        .arg("--force")
        .assert()
        .failure();
}

/// Test missing version value with only update flag
#[test]
fn test_missing_version_value() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        // Missing version value, no other args
        .assert()
        .failure();
}

// ========== COMPLEX WORKFLOW INTEGRATION TESTS ==========

/// Test complete workflow with all valid update options
#[test]
fn test_complete_workflow_all_options() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("1.2.3")
        .arg("--force")
        .arg("--verbose")
        .arg("--color")
        .arg("--count")    // Should be ignored in update mode
        .arg("5")
        .arg("--timeout")  // Should be ignored in update mode
        .arg("30")
        .assert()
        .success()
        .stdout(version_output_predicate());
}

/// Test workflow with mixed short and long flags
#[test]
fn test_workflow_mixed_flags() {
    create_update_test_cmd()
        .arg("-u")
        .arg("--version")
        .arg("v2.0.0")
        .arg("-f")
        .arg("--verbose")
        .arg("--no-color")
        .assert()
        .success()
        .stdout(version_output_predicate());
}

/// Test workflow with environment variables
#[test]
fn test_workflow_with_environment() {
    create_update_test_cmd()
        .env("TARGET_URLS", "https://example.com") // Should be ignored in update mode
        .env("DEBUG", "true")   // Should not interfere
        .env("VERBOSE", "true") // Should not interfere
        .arg("--update")
        .arg("--version")
        .arg("1.0.0")
        .assert()
        .success()
        .stdout(version_output_predicate());
}

// ========== PARAMETER ORDER INDEPENDENCE TESTS ==========

/// Test parameter order independence in update mode
#[test]
fn test_update_parameter_order_independence() {
    let parameter_orderings = vec![
        // Different valid orderings
        vec!["--update", "--version", "1.2.3", "--force", "--verbose"],
        vec!["--version", "1.2.3", "--update", "--force", "--verbose"],
        vec!["--force", "--verbose", "--update", "--version", "1.2.3"],
        vec!["--verbose", "--force", "--version", "1.2.3", "--update"],
        
        // With short flags
        vec!["-u", "-v", "1.2.3", "-f", "--verbose"],
        vec!["-v", "1.2.3", "-u", "-f", "--verbose"],
        vec!["-f", "--verbose", "-u", "-v", "1.2.3"],
        vec!["--verbose", "-f", "-v", "1.2.3", "-u"],
    ];
    
    for args in parameter_orderings {
        create_update_test_cmd()
            .args(&args)
            .assert()
            .success()
            .stdout(version_output_predicate());
    }
}

/// Test parameter order with output options
#[test]
fn test_parameter_order_with_output_options() {
    let orderings = vec![
        vec!["--update", "--no-color", "--version", "1.0.0"],
        vec!["--color", "--update", "--version", "1.0.0"],
        vec!["--version", "1.0.0", "--color", "--update"],
        vec!["--update", "--debug", "--version", "1.0.0"],
        vec!["--verbose", "--update", "--version", "1.0.0"],
    ];
    
    for args in orderings {
        let mut cmd = create_update_test_cmd();
        cmd.args(&args);
        
        if args.contains(&"--color") && args.contains(&"--no-color") {
            // Should fail for conflicting color options
            cmd.assert().failure();
        } else {
            // Should succeed
            cmd.assert()
                .success()
                .stdout(version_output_predicate());
        }
    }
}

// ========== SAME VERSION DETECTION TESTS ==========

/// Test same version detection workflow
#[test]
fn test_same_version_detection() {
    // Try to target current version (should result in "already up to date")
    let current_version = env!("CARGO_PKG_VERSION");
    
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg(current_version)
        .assert()
        .success()
        .stdout(version_output_predicate());
    
    // Also test with v prefix
    let v_prefixed = format!("v{}", current_version);
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg(&v_prefixed)
        .assert()
        .success()
        .stdout(version_output_predicate());
}

/// Test same version with force flag
#[test]
fn test_same_version_with_force() {
    let current_version = env!("CARGO_PKG_VERSION");
    
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg(current_version)
        .arg("--force")
        .assert()
        .success()
        .stdout(version_output_predicate());
}

// ========== HELP AND VERSION PRECEDENCE TESTS ==========

/// Test help takes precedence over update mode
#[test]
fn test_help_precedence_over_update() {
    create_update_test_cmd()
        .arg("--help")
        .arg("--update")
        .arg("--version")
        .arg("1.2.3")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("--update"))
        .stdout(predicate::str::contains("--version"))
        .stdout(predicate::str::contains("--force"));
}

/// Test version flag takes precedence over update mode
#[test]
fn test_version_flag_precedence() {
    create_update_test_cmd()
        .arg("--version")
        .arg("--update")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Test help topic with update parameters
#[test]
fn test_help_topic_with_update() {
    create_update_test_cmd()
        .arg("--help-topic")
        .arg("config")
        .arg("--update")
        .arg("--version")
        .arg("1.0.0")
        .assert()
        .success()
        .stdout(predicate::str::contains("CONFIGURATION"));
}

// ========== PLATFORM DETECTION INTEGRATION TESTS ==========

/// Test update mode respects platform detection
#[test]
fn test_update_mode_platform_detection() {
    create_update_test_cmd()
        .arg("--update")
        .arg("--verbose") // To see platform info
        .assert()
        .success()
        .stdout(update_mode_predicate());
}

/// Test update with all output formats
#[test]
fn test_update_with_output_formats() {
    // Test with color
    create_update_test_cmd()
        .arg("--update")
        .arg("--color")
        .assert()
        .success();
    
    // Test with no color
    create_update_test_cmd()
        .arg("--update")
        .arg("--no-color")
        .assert()
        .success();
    
    // Test with verbose
    create_update_test_cmd()
        .arg("--update")
        .arg("--verbose")
        .assert()
        .success();
    
    // Test with debug
    create_update_test_cmd()
        .arg("--update")
        .arg("--debug")
        .assert()
        .success();
}

// ========== PERFORMANCE AND TIMEOUT TESTS ==========

/// Test update operations complete within reasonable time
#[test]
fn test_update_operation_performance() {
    use std::time::Instant;
    
    let start = Instant::now();
    
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("1.0.0")
        .assert()
        .success();
    
    let elapsed = start.elapsed();
    
    // Update operations should complete quickly (placeholder implementation)
    assert!(elapsed < Duration::from_secs(30), 
        "Update operation took too long: {:?}", elapsed);
}

/// Test complex update operations performance
#[test]
fn test_complex_update_performance() {
    use std::time::Instant;
    
    let start = Instant::now();
    
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("v1.2.3-alpha.beta.gamma")
        .arg("--force")
        .arg("--verbose")
        .arg("--debug")
        .arg("--color")
        .assert()
        .success();
    
    let elapsed = start.elapsed();
    
    // Complex operations should still complete within reasonable time
    assert!(elapsed < Duration::from_secs(45), 
        "Complex update operation took too long: {:?}", elapsed);
}

// ========== MALFORMED INPUT HANDLING TESTS ==========

/// Test malformed version inputs
#[test]
fn test_malformed_version_inputs() {
    let malformed_inputs = vec![
        "--version",      // No value
        "--version=",     // Empty value with equals
        "--version=invalid", // Invalid with equals
        "--version invalid space", // Space in value
    ];
    
    for input in malformed_inputs {
        if input.contains('=') {
            // Test equals format
            create_update_test_cmd()
                .arg("--update")
                .arg(input)
                .assert()
                .failure();
        } else if input == "--version" {
            // Test missing value
            create_update_test_cmd()
                .arg("--update")
                .arg(input)
                .assert()
                .failure();
        }
    }
}

/// Test invalid flag combinations
#[test]
fn test_invalid_flag_combinations() {
    // Invalid flags that don't exist
    create_update_test_cmd()
        .arg("--update")
        .arg("--invalid-flag")
        .assert()
        .failure();
    
    // Typos in update flags
    create_update_test_cmd()
        .arg("--updates")  // Typo
        .assert()
        .failure();
    
    create_update_test_cmd()
        .arg("--Update")   // Wrong case
        .assert()
        .failure();
}

// ========== COMPREHENSIVE WORKFLOW VALIDATION TESTS ==========

/// Test complete valid update workflows
#[test]
fn test_comprehensive_valid_workflows() {
    let workflows = vec![
        // Basic update check
        vec!["--update"],
        
        // Version targeting
        vec!["--update", "--version", "1.0.0"],
        vec!["-u", "-v", "1.0.0"],
        
        // Force operations
        vec!["--update", "--version", "0.1.0", "--force"],
        vec!["-u", "-v", "0.1.0", "-f"],
        
        // With output options
        vec!["--update", "--verbose"],
        vec!["--update", "--debug"],
        vec!["--update", "--color"],
        vec!["--update", "--no-color"],
        
        // Complex combinations
        vec!["--update", "--version", "1.2.3", "--force", "--verbose", "--color"],
        vec!["-u", "-v", "v2.0.0", "-f", "--debug", "--no-color"],
    ];
    
    for workflow in workflows {
        create_update_test_cmd()
            .args(&workflow)
            .assert()
            .success()
            .stdout(predicate::str::contains("检查更新").or(predicate::str::contains("update"))
                .or(version_output_predicate()));
    }
}

/// Test update workflow error recovery
#[test]
fn test_update_workflow_error_recovery() {
    let error_scenarios = vec![
        // Invalid version formats (should fail gracefully)
        (vec!["--update", "--version", "invalid"], "version format"),
        (vec!["--update", "--version", "1.2"], "version format"),
        (vec!["--update", "--version", ""], "version format"),
        
        // Conflicting options (should fail with clear message)
        (vec!["--update", "--color", "--no-color"], "color"),
    ];
    
    for (args, expected_error) in error_scenarios {
        create_update_test_cmd()
            .args(&args)
            .assert()
            .failure()
            .stderr(predicate::str::contains(expected_error));
    }
}

// ========== END-TO-END INTEGRATION VALIDATION ==========

/// Test end-to-end update flow integration with all components
#[test]
fn test_end_to_end_update_integration() {
    // This test verifies that all update components work together
    // Testing the three main update scenarios: upgrade, same version, downgrade
    
    let current_version = env!("CARGO_PKG_VERSION");
    
    // Scenario 1: Target same version (should succeed)
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg(current_version)
        .arg("--verbose")
        .assert()
        .success()
        .stdout(version_output_predicate());
    
    // Scenario 2: Target potential upgrade version (should succeed)
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("9.9.9")  // Likely higher than current
        .arg("--verbose")
        .assert()
        .success()
        .stdout(version_output_predicate());
    
    // Scenario 3: Force downgrade (should succeed with warning)
    create_update_test_cmd()
        .arg("--update")
        .arg("--version")
        .arg("0.1.0")  // Likely lower than current
        .arg("--force")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(version_output_predicate());
}

/// Test update mode integration with existing CLI patterns
#[test]
fn test_update_cli_integration_patterns() {
    // Verify update mode follows same patterns as rest of CLI
    
    // Should work with existing output flags
    create_update_test_cmd()
        .arg("--update")
        .arg("--verbose")
        .arg("--no-color")
        .assert()
        .success();
    
    // Should work with existing help system
    create_update_test_cmd()
        .arg("--help-topic")
        .arg("config")
        .arg("--update")  // Should be ignored when help takes precedence
        .assert()
        .success()
        .stdout(predicate::str::contains("CONFIGURATION"));
    
    // Should follow same error patterns
    create_update_test_cmd()
        .arg("--update")
        .arg("--invalid-option")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("invalid")));
}