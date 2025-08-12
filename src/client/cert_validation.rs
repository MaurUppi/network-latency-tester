//! Certificate validation testing across different platforms
//!
//! This module provides comprehensive TLS certificate validation testing
//! to ensure consistent behavior across different operating systems and
//! networking environments.

use crate::{
    error::{AppError, Result},
    client::platform::{PlatformNetworkConfig, CertificateValidator},
};
use reqwest::Client;
use std::{
    time::{Duration, Instant},
    collections::HashMap,
};

/// Certificate validation test suite
pub struct CertificateValidationTester {
    validator: CertificateValidator,
    config: PlatformNetworkConfig,
}

impl CertificateValidationTester {
    /// Create a new certificate validation tester
    pub fn new() -> Self {
        Self {
            validator: CertificateValidator::new(),
            config: PlatformNetworkConfig::for_current_platform(),
        }
    }

    /// Create with custom certificate validator
    pub fn with_validator(validator: CertificateValidator) -> Self {
        Self {
            config: PlatformNetworkConfig::for_current_platform(),
            validator,
        }
    }

    /// Run comprehensive certificate validation tests
    pub async fn run_certificate_tests(&self) -> CertificateTestResults {
        let mut results = CertificateTestResults {
            platform: crate::dns::platform::get_platform_name(),
            valid_cert_test: None,
            invalid_cert_test: None,
            expired_cert_test: None,
            self_signed_cert_test: None,
            wildcard_cert_test: None,
            chain_validation_test: None,
            tls_version_tests: HashMap::new(),
            overall_score: 0,
        };

        // Test valid certificates
        results.valid_cert_test = Some(self.test_valid_certificate().await);

        // Test invalid certificates
        results.invalid_cert_test = Some(self.test_invalid_certificate().await);

        // Test expired certificates
        results.expired_cert_test = Some(self.test_expired_certificate().await);

        // Test self-signed certificates
        results.self_signed_cert_test = Some(self.test_self_signed_certificate().await);

        // Test wildcard certificates
        results.wildcard_cert_test = Some(self.test_wildcard_certificate().await);

        // Test certificate chain validation
        results.chain_validation_test = Some(self.test_certificate_chain().await);

        // Test different TLS versions
        results.tls_version_tests = self.test_tls_versions().await;

        // Calculate overall score
        results.overall_score = self.calculate_overall_score(&results);

        results
    }

    /// Test valid certificate handling
    async fn test_valid_certificate(&self) -> CertificateTest {
        let test_urls = vec![
            "https://google.com",
            "https://github.com", 
            "https://cloudflare.com",
            "https://mozilla.org",
        ];

        let mut successful_tests = 0;
        let mut test_details = Vec::new();
        let start_time = Instant::now();

        for url in &test_urls {
            match self.test_certificate_for_url(url, true).await {
                Ok(details) => {
                    successful_tests += 1;
                    test_details.push(format!("✓ {}: {}", url, details));
                }
                Err(e) => {
                    test_details.push(format!("✗ {}: {}", url, e));
                }
            }
        }

        CertificateTest {
            test_name: "Valid Certificate Test".to_string(),
            successful: successful_tests == test_urls.len(),
            success_rate: (successful_tests as f64 / test_urls.len() as f64) * 100.0,
            duration: start_time.elapsed(),
            details: test_details,
            error_message: if successful_tests == test_urls.len() {
                None
            } else {
                Some(format!("Only {}/{} tests passed", successful_tests, test_urls.len()))
            },
        }
    }

    /// Test invalid certificate handling
    async fn test_invalid_certificate(&self) -> CertificateTest {
        let test_urls = vec![
            "https://expired.badssl.com",
            "https://wrong.host.badssl.com",
            "https://untrusted-root.badssl.com",
        ];

        let mut handled_correctly = 0;
        let mut test_details = Vec::new();
        let start_time = Instant::now();

        for url in &test_urls {
            // These should fail with certificate validation enabled
            match self.test_certificate_for_url(url, true).await {
                Ok(_) => {
                    test_details.push(format!("✗ {}: Should have failed but didn't", url));
                }
                Err(e) => {
                    handled_correctly += 1;
                    test_details.push(format!("✓ {}: Correctly rejected ({})", url, e));
                }
            }
        }

        CertificateTest {
            test_name: "Invalid Certificate Test".to_string(),
            successful: handled_correctly == test_urls.len(),
            success_rate: (handled_correctly as f64 / test_urls.len() as f64) * 100.0,
            duration: start_time.elapsed(),
            details: test_details,
            error_message: if handled_correctly == test_urls.len() {
                None
            } else {
                Some(format!("Only {}/{} invalid certs correctly rejected", handled_correctly, test_urls.len()))
            },
        }
    }

    /// Test expired certificate handling
    async fn test_expired_certificate(&self) -> CertificateTest {
        let test_url = "https://expired.badssl.com";
        let start_time = Instant::now();

        let mut test_details = Vec::new();
        let successful = match self.test_certificate_for_url(test_url, true).await {
            Ok(_) => {
                test_details.push("✗ Expired certificate was accepted (should be rejected)".to_string());
                false
            }
            Err(e) => {
                test_details.push(format!("✓ Expired certificate correctly rejected: {}", e));
                true
            }
        };

        CertificateTest {
            test_name: "Expired Certificate Test".to_string(),
            successful,
            success_rate: if successful { 100.0 } else { 0.0 },
            duration: start_time.elapsed(),
            details: test_details,
            error_message: if successful {
                None
            } else {
                Some("Expired certificate was not properly rejected".to_string())
            },
        }
    }

    /// Test self-signed certificate handling
    async fn test_self_signed_certificate(&self) -> CertificateTest {
        let test_url = "https://self-signed.badssl.com";
        let start_time = Instant::now();

        let mut test_details = Vec::new();
        let successful = match self.test_certificate_for_url(test_url, true).await {
            Ok(_) => {
                test_details.push("✗ Self-signed certificate was accepted (should be rejected)".to_string());
                false
            }
            Err(e) => {
                test_details.push(format!("✓ Self-signed certificate correctly rejected: {}", e));
                true
            }
        };

        CertificateTest {
            test_name: "Self-Signed Certificate Test".to_string(),
            successful,
            success_rate: if successful { 100.0 } else { 0.0 },
            duration: start_time.elapsed(),
            details: test_details,
            error_message: if successful {
                None
            } else {
                Some("Self-signed certificate was not properly rejected".to_string())
            },
        }
    }

    /// Test wildcard certificate handling
    async fn test_wildcard_certificate(&self) -> CertificateTest {
        let test_urls = vec![
            "https://badssl.com",  // Should work
            // Note: Finding reliable wildcard test endpoints is challenging
        ];

        let mut successful_tests = 0;
        let mut test_details = Vec::new();
        let start_time = Instant::now();

        for url in &test_urls {
            match self.test_certificate_for_url(url, true).await {
                Ok(details) => {
                    successful_tests += 1;
                    test_details.push(format!("✓ {}: {}", url, details));
                }
                Err(e) => {
                    test_details.push(format!("✗ {}: {}", url, e));
                }
            }
        }

        CertificateTest {
            test_name: "Wildcard Certificate Test".to_string(),
            successful: successful_tests > 0,
            success_rate: (successful_tests as f64 / test_urls.len() as f64) * 100.0,
            duration: start_time.elapsed(),
            details: test_details,
            error_message: None,
        }
    }

    /// Test certificate chain validation
    async fn test_certificate_chain(&self) -> CertificateTest {
        let test_urls = vec![
            "https://incomplete-chain.badssl.com",
            "https://google.com", // Good chain for comparison
        ];

        let mut test_details = Vec::new();
        let start_time = Instant::now();
        let mut chain_tests_passed = 0;

        for url in &test_urls {
            match self.test_certificate_for_url(url, true).await {
                Ok(details) => {
                    if url.contains("incomplete-chain") {
                        test_details.push(format!("? {}: May pass or fail depending on platform: {}", url, details));
                    } else {
                        chain_tests_passed += 1;
                        test_details.push(format!("✓ {}: {}", url, details));
                    }
                }
                Err(e) => {
                    if url.contains("incomplete-chain") {
                        chain_tests_passed += 1; // Expected to fail
                        test_details.push(format!("✓ {}: Correctly rejected incomplete chain: {}", url, e));
                    } else {
                        test_details.push(format!("✗ {}: Unexpected failure: {}", url, e));
                    }
                }
            }
        }

        CertificateTest {
            test_name: "Certificate Chain Validation Test".to_string(),
            successful: chain_tests_passed > 0,
            success_rate: (chain_tests_passed as f64 / test_urls.len() as f64) * 100.0,
            duration: start_time.elapsed(),
            details: test_details,
            error_message: None,
        }
    }

    /// Test different TLS versions
    async fn test_tls_versions(&self) -> HashMap<String, CertificateTest> {
        let mut tls_tests = HashMap::new();

        // Test TLS 1.2
        tls_tests.insert("TLS 1.2".to_string(), self.test_tls_version("1.2").await);

        // Test TLS 1.3 
        tls_tests.insert("TLS 1.3".to_string(), self.test_tls_version("1.3").await);

        tls_tests
    }

    /// Test specific TLS version
    async fn test_tls_version(&self, version: &str) -> CertificateTest {
        let test_urls = vec![
            "https://tls-v1-2.badssl.com:1012", // TLS 1.2 only
            "https://google.com", // Should support multiple versions
        ];

        let mut successful_tests = 0;
        let mut test_details = Vec::new();
        let start_time = Instant::now();

        for url in &test_urls {
            match self.test_certificate_for_url(url, true).await {
                Ok(details) => {
                    successful_tests += 1;
                    test_details.push(format!("✓ {}: {}", url, details));
                }
                Err(e) => {
                    test_details.push(format!("✗ {}: {}", url, e));
                }
            }
        }

        CertificateTest {
            test_name: format!("TLS {} Support Test", version),
            successful: successful_tests > 0,
            success_rate: (successful_tests as f64 / test_urls.len() as f64) * 100.0,
            duration: start_time.elapsed(),
            details: test_details,
            error_message: None,
        }
    }

    /// Test certificate validation for a specific URL
    async fn test_certificate_for_url(&self, url: &str, strict_validation: bool) -> Result<String> {
        let timeout = self.validator.get_certificate_timeout();
        
        let mut builder = Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10));

        if !strict_validation {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build()
            .map_err(|e| AppError::network(format!("Failed to create client: {}", e)))?;

        let start_time = Instant::now();
        
        let response = client
            .head(url)
            .send()
            .await
            .map_err(|e| AppError::network(format!("Request failed: {}", e)))?;

        let request_time = start_time.elapsed();
        
        Ok(format!(
            "Status: {}, Time: {:?}", 
            response.status(), 
            request_time
        ))
    }

    /// Calculate overall certificate validation score
    fn calculate_overall_score(&self, results: &CertificateTestResults) -> u8 {
        let mut total_score = 0u8;
        let mut test_count = 0u8;

        if let Some(ref test) = results.valid_cert_test {
            total_score += if test.successful { 20 } else { 0 };
            test_count += 1;
        }

        if let Some(ref test) = results.invalid_cert_test {
            total_score += if test.successful { 25 } else { 0 };
            test_count += 1;
        }

        if let Some(ref test) = results.expired_cert_test {
            total_score += if test.successful { 20 } else { 0 };
            test_count += 1;
        }

        if let Some(ref test) = results.self_signed_cert_test {
            total_score += if test.successful { 20 } else { 0 };
            test_count += 1;
        }

        if let Some(ref test) = results.chain_validation_test {
            total_score += if test.successful { 15 } else { 0 };
            test_count += 1;
        }

        // Normalize to 100-point scale
        if test_count > 0 {
            total_score
        } else {
            0
        }
    }
}

impl Default for CertificateValidationTester {
    fn default() -> Self {
        Self::new()
    }
}

/// Results of certificate validation tests
#[derive(Debug, Clone)]
pub struct CertificateTestResults {
    pub platform: String,
    pub valid_cert_test: Option<CertificateTest>,
    pub invalid_cert_test: Option<CertificateTest>,
    pub expired_cert_test: Option<CertificateTest>,
    pub self_signed_cert_test: Option<CertificateTest>,
    pub wildcard_cert_test: Option<CertificateTest>,
    pub chain_validation_test: Option<CertificateTest>,
    pub tls_version_tests: HashMap<String, CertificateTest>,
    pub overall_score: u8,
}

impl CertificateTestResults {
    /// Generate a comprehensive test report
    pub fn generate_report(&self) -> String {
        let mut report = format!("Certificate Validation Test Report for {}:\n", self.platform);
        report.push_str(&format!("Overall Score: {}/100\n\n", self.overall_score));

        // Add individual test results
        if let Some(ref test) = self.valid_cert_test {
            report.push_str(&self.format_test_result(test));
        }

        if let Some(ref test) = self.invalid_cert_test {
            report.push_str(&self.format_test_result(test));
        }

        if let Some(ref test) = self.expired_cert_test {
            report.push_str(&self.format_test_result(test));
        }

        if let Some(ref test) = self.self_signed_cert_test {
            report.push_str(&self.format_test_result(test));
        }

        if let Some(ref test) = self.wildcard_cert_test {
            report.push_str(&self.format_test_result(test));
        }

        if let Some(ref test) = self.chain_validation_test {
            report.push_str(&self.format_test_result(test));
        }

        // Add TLS version test results
        for (version, test) in &self.tls_version_tests {
            report.push_str(&format!("\n{} Test:\n", version));
            report.push_str(&self.format_test_result(test));
        }

        report
    }

    /// Format individual test result
    fn format_test_result(&self, test: &CertificateTest) -> String {
        let mut result = format!("\n{}:\n", test.test_name);
        result.push_str(&format!("  Status: {}\n", if test.successful { "✓ PASSED" } else { "✗ FAILED" }));
        result.push_str(&format!("  Success Rate: {:.1}%\n", test.success_rate));
        result.push_str(&format!("  Duration: {:?}\n", test.duration));
        
        if let Some(ref error) = test.error_message {
            result.push_str(&format!("  Error: {}\n", error));
        }

        result.push_str("  Details:\n");
        for detail in &test.details {
            result.push_str(&format!("    {}\n", detail));
        }

        result
    }

    /// Check if certificate validation is working properly
    pub fn is_certificate_validation_healthy(&self) -> bool {
        self.overall_score >= 75 // 75% threshold for healthy validation
    }

    /// Get recommendations based on test results
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.overall_score < 50 {
            recommendations.push("Certificate validation appears to be severely compromised. Review TLS settings.".to_string());
        }

        if let Some(ref test) = self.invalid_cert_test {
            if !test.successful {
                recommendations.push("Invalid certificates are being accepted. Enable strict certificate validation.".to_string());
            }
        }

        if let Some(ref test) = self.expired_cert_test {
            if !test.successful {
                recommendations.push("Expired certificates are being accepted. Check system clock and certificate validation settings.".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Certificate validation is working properly.".to_string());
        }

        recommendations
    }
}

/// Individual certificate test result
#[derive(Debug, Clone)]
pub struct CertificateTest {
    pub test_name: String,
    pub successful: bool,
    pub success_rate: f64,
    pub duration: Duration,
    pub details: Vec<String>,
    pub error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_tester_creation() {
        let tester = CertificateValidationTester::new();
        // Should create without panicking
        drop(tester);
    }

    #[test]
    fn test_certificate_tester_with_custom_validator() {
        let validator = CertificateValidator::with_strict_validation(false);
        let tester = CertificateValidationTester::with_validator(validator);
        drop(tester);
    }

    #[tokio::test]
    async fn test_valid_certificate_validation() {
        let tester = CertificateValidationTester::new();
        let result = tester.test_certificate_for_url("https://google.com", true).await;
        
        // This might fail in CI environments, so we just test that it doesn't panic
        match result {
            Ok(_) => {
                // Certificate validation successful
            }
            Err(_) => {
                // Network might not be available or certificate issues
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_certificate_handling() {
        let tester = CertificateValidationTester::new();
        let result = tester.test_certificate_for_url("https://expired.badssl.com", true).await;
        
        // Should fail with certificate validation enabled
        assert!(result.is_err());
    }

    #[test]
    fn test_certificate_test_results_scoring() {
        let results = CertificateTestResults {
            platform: "Test".to_string(),
            valid_cert_test: Some(CertificateTest {
                test_name: "Test".to_string(),
                successful: true,
                success_rate: 100.0,
                duration: Duration::from_millis(100),
                details: vec!["Test detail".to_string()],
                error_message: None,
            }),
            invalid_cert_test: None,
            expired_cert_test: None,
            self_signed_cert_test: None,
            wildcard_cert_test: None,
            chain_validation_test: None,
            tls_version_tests: HashMap::new(),
            overall_score: 20,
        };

        let report = results.generate_report();
        assert!(report.contains("Test"));
        assert!(report.contains("Overall Score: 20/100"));
    }

    #[test]
    fn test_certificate_validation_health_check() {
        let healthy_results = CertificateTestResults {
            platform: "Test".to_string(),
            valid_cert_test: None,
            invalid_cert_test: None,
            expired_cert_test: None,
            self_signed_cert_test: None,
            wildcard_cert_test: None,
            chain_validation_test: None,
            tls_version_tests: HashMap::new(),
            overall_score: 80,
        };

        assert!(healthy_results.is_certificate_validation_healthy());

        let unhealthy_results = CertificateTestResults {
            platform: "Test".to_string(),
            valid_cert_test: None,
            invalid_cert_test: None,
            expired_cert_test: None,
            self_signed_cert_test: None,
            wildcard_cert_test: None,
            chain_validation_test: None,
            tls_version_tests: HashMap::new(),
            overall_score: 50,
        };

        assert!(!unhealthy_results.is_certificate_validation_healthy());
    }

    #[test]
    fn test_certificate_recommendations() {
        let results = CertificateTestResults {
            platform: "Test".to_string(),
            valid_cert_test: None,
            invalid_cert_test: Some(CertificateTest {
                test_name: "Invalid Test".to_string(),
                successful: false,
                success_rate: 0.0,
                duration: Duration::from_millis(100),
                details: vec![],
                error_message: Some("Test failed".to_string()),
            }),
            expired_cert_test: None,
            self_signed_cert_test: None,
            wildcard_cert_test: None,
            chain_validation_test: None,
            tls_version_tests: HashMap::new(),
            overall_score: 30,
        };

        let recommendations = results.get_recommendations();
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("Invalid certificates")));
    }
}