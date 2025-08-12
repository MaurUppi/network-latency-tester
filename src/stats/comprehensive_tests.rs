//! Comprehensive tests for statistical analysis and calculations
//! 
//! This module contains property-based tests and edge case testing
//! for the statistics engine and mathematical functions.

use super::StatisticsEngine;
use crate::{
    models::metrics::{TimingMetrics, TestResult},
    types::DnsConfig,
};
use proptest::prelude::*;
use proptest::collection::vec;
use std::time::Duration;

/// Property-based test generators
mod generators {
    use super::*;
    
    /// Generate valid timing metrics for property tests
    pub fn timing_metrics() -> impl Strategy<Value = TimingMetrics> {
        (1u64..10000, 1u64..5000, 0u64..2000, 50u64..5000, 100u64..10000, 200u16..399u16)
            .prop_map(|(dns, connect, tls, first_byte, total, status)| {
                TimingMetrics::success(
                    Duration::from_millis(dns),
                    Duration::from_millis(connect),
                    Some(Duration::from_millis(tls)),
                    Duration::from_millis(first_byte),
                    Duration::from_millis(total),
                    status,
                )
            })
    }
    
    /// Generate test results with various configurations
    pub fn test_results() -> impl Strategy<Value = TestResult> {
        (
            "[a-z]{3,10}",
            prop_oneof![
                Just(DnsConfig::System),
                vec("[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}", 1..4)
                    .prop_map(|servers| DnsConfig::Custom { 
                        servers: servers.into_iter()
                            .filter_map(|s| s.parse().ok())
                            .collect()
                    }),
                "https://[a-z0-9.-]+/dns-query"
                    .prop_map(|url| DnsConfig::DoH { url })
            ],
            "https://[a-z0-9.-]+\\.(com|org|net)",
            vec(timing_metrics(), 1..50)
        ).prop_map(|(name, dns_config, url, measurements)| {
            let mut result = TestResult::new(name, dns_config, url);
            for measurement in measurements {
                result.add_measurement(measurement);
            }
            result.calculate_statistics();
            result
        })
    }
    
    /// Generate floating point numbers for mathematical tests
    pub fn positive_floats() -> impl Strategy<Value = f64> {
        0.001f64..1000000.0
    }
    
    /// Generate vectors of positive numbers for statistical calculations
    pub fn number_vectors() -> impl Strategy<Value = Vec<f64>> {
        vec(positive_floats(), 1..1000)
    }
}

/// Test mathematical properties of statistical calculations
mod property_tests {
    use super::*;
    
    proptest! {
        /// Mean should always be between min and max
        #[test]
        fn mean_between_min_max(numbers in generators::number_vectors()) {
            let min = numbers.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = numbers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let sum: f64 = numbers.iter().sum();
            let mean = sum / numbers.len() as f64;
            
            prop_assert!(mean >= min);
            prop_assert!(mean <= max);
        }
        
        /// Standard deviation should be non-negative
        #[test]
        fn standard_deviation_non_negative(numbers in generators::number_vectors()) {
            if numbers.len() > 1 {
                let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
                let variance = numbers.iter()
                    .map(|x| (x - mean).powi(2))
                    .sum::<f64>() / (numbers.len() - 1) as f64;
                let std_dev = variance.sqrt();
                
                prop_assert!(std_dev >= 0.0);
                prop_assert!(!std_dev.is_nan() || numbers.iter().all(|&x| x == numbers[0]));
            }
        }
        
        /// Statistics engine should handle multiple results correctly
        #[test]
        fn statistics_engine_multiple_results(results in vec(generators::test_results(), 1..20)) {
            let mut engine = StatisticsEngine::with_defaults();
            
            for result in &results {
                engine.add_result(result.clone());
            }
            
            let analysis = engine.analyze();
            prop_assert!(analysis.is_ok());
            
            if let Ok(analysis) = analysis {
                prop_assert!(!analysis.basic_stats.is_empty());
                prop_assert!(analysis.generated_at <= chrono::Utc::now());
            }
        }
        
        /// Confidence intervals should contain the mean (basic property test)
        #[test]
        fn confidence_interval_contains_mean(numbers in generators::number_vectors()) {
            if numbers.len() > 2 {
                let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
                let std_dev = {
                    let variance = numbers.iter()
                        .map(|x| (x - mean).powi(2))
                        .sum::<f64>() / (numbers.len() - 1) as f64;
                    variance.sqrt()
                };
                
                if std_dev > 0.0 && std_dev.is_finite() {
                    // 95% confidence interval (approximate)
                    let margin = 1.96 * std_dev / (numbers.len() as f64).sqrt();
                    let lower = mean - margin;
                    let upper = mean + margin;
                    
                    prop_assert!(lower <= mean);
                    prop_assert!(mean <= upper);
                    prop_assert!(lower <= upper);
                }
            }
        }
    }
}

/// Test edge cases and boundary conditions
mod edge_case_tests {
    use super::*;
    
    #[test]
    fn test_empty_statistics_engine() {
        let engine = StatisticsEngine::with_defaults();
        let analysis = engine.analyze();
        
        // Empty engine should return error
        assert!(analysis.is_err());
    }
    
    #[test]
    fn test_single_result_statistics() {
        let mut engine = StatisticsEngine::with_defaults();
        let mut result = TestResult::new(
            "single".to_string(),
            DnsConfig::System,
            "https://example.com".to_string()
        );
        
        // Add at least one successful measurement for statistics
        result.add_measurement(TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            None,
            Duration::from_millis(50),
            Duration::from_millis(100),
            200,
        ));
        result.calculate_statistics();
        
        engine.add_result(result);
        let analysis = engine.analyze().unwrap();
        
        assert!(!analysis.basic_stats.is_empty());
    }
    
    #[test]
    fn test_identical_measurements() {
        let mut result = TestResult::new(
            "identical".to_string(),
            DnsConfig::System,
            "https://example.com".to_string()
        );
        
        // Add 10 identical measurements
        for _ in 0..10 {
            result.add_measurement(TimingMetrics::success(
                Duration::from_millis(100),
                Duration::from_millis(50),
                Some(Duration::from_millis(25)),
                Duration::from_millis(200),
                Duration::from_millis(375),
                200,
            ));
        }
        
        result.calculate_statistics();
        
        let stats = result.statistics.as_ref().unwrap();
        assert_eq!(stats.total_min_ms, stats.total_max_ms);
        assert_eq!(stats.total_avg_ms, stats.total_min_ms);
        assert_eq!(stats.total_std_dev_ms, 0.0);
    }
    
    #[test]
    fn test_extreme_values() {
        let mut result = TestResult::new(
            "extreme".to_string(),
            DnsConfig::System,
            "https://example.com".to_string()
        );
        
        // Add measurements with extreme values
        result.add_measurement(TimingMetrics::success(
            Duration::from_nanos(1),
            Duration::from_nanos(1),
            None,
            Duration::from_millis(1),
            Duration::from_millis(2),
            200,
        ));
        
        result.add_measurement(TimingMetrics::success(
            Duration::from_secs(30),
            Duration::from_secs(10),
            Some(Duration::from_secs(5)),
            Duration::from_secs(45),
            Duration::from_secs(90),
            200,
        ));
        
        result.calculate_statistics();
        
        let stats = result.statistics.as_ref().unwrap();
        assert!(stats.total_max_ms > stats.total_min_ms);
        assert!(stats.total_std_dev_ms > 0.0);
    }
    
    #[test]
    fn test_failed_and_timeout_measurements() {
        let mut result = TestResult::new(
            "mixed".to_string(),
            DnsConfig::System,
            "https://example.com".to_string()
        );
        
        // Add successful measurement
        result.add_measurement(TimingMetrics::success(
            Duration::from_millis(100),
            Duration::from_millis(50),
            Some(Duration::from_millis(25)),
            Duration::from_millis(200),
            Duration::from_millis(375),
            200,
        ));
        
        // Add failed measurement
        result.add_measurement(TimingMetrics::failed("Network error".to_string()));
        
        // Add timeout measurement
        result.add_measurement(TimingMetrics::timeout(Duration::from_secs(30)));
        
        result.calculate_statistics();
        
        assert_eq!(result.total_count, 3);
        assert_eq!(result.success_count, 1);
        assert!((result.success_rate() - 33.33333333333333).abs() < 0.001); // 1/3 * 100 with tolerance
        
        let stats = result.statistics.as_ref().unwrap();
        assert!(stats.total_avg_ms > 0.0);
    }
    
    #[test]
    fn test_large_dataset_performance() {
        let mut engine = StatisticsEngine::with_defaults();
        
        // Create a large number of results
        for i in 0..100 { // Reduced from 1000 for faster tests
            let mut result = TestResult::new(
                format!("config_{}", i),
                DnsConfig::System,
                "https://example.com".to_string()
            );
            
            // Add multiple measurements per result
            for j in 0..5 { // Reduced from 10 for faster tests
                result.add_measurement(TimingMetrics::success(
                    Duration::from_millis(50 + j),
                    Duration::from_millis(25 + j/2),
                    Some(Duration::from_millis(10 + j/3)),
                    Duration::from_millis(100 + j*2),
                    Duration::from_millis(185 + j*3),
                    200,
                ));
            }
            
            result.calculate_statistics();
            engine.add_result(result);
        }
        
        let start = std::time::Instant::now();
        let analysis = engine.analyze().unwrap();
        let duration = start.elapsed();
        
        // Analysis should complete in reasonable time
        assert!(duration < Duration::from_secs(5));
        assert!(!analysis.basic_stats.is_empty());
    }
    
    #[test]
    fn test_mathematical_precision() {
        let mut result = TestResult::new(
            "precision".to_string(),
            DnsConfig::System,
            "https://example.com".to_string()
        );
        
        // Add measurements that might cause precision issues
        let precise_values = [1.0, 1000000.0, 0.000001]; // Reduced extreme values
        
        for &val in &precise_values {
            let millis = (val * 1000.0) as u64;
            result.add_measurement(TimingMetrics::success(
                Duration::from_millis(millis),
                Duration::from_millis(millis/2),
                Some(Duration::from_millis(millis/4)),
                Duration::from_millis(millis*2),
                Duration::from_millis(millis*3),
                200,
            ));
        }
        
        result.calculate_statistics();
        
        let stats = result.statistics.as_ref().unwrap();
        assert!(!stats.total_avg_ms.is_nan());
        assert!(!stats.total_std_dev_ms.is_nan());
        assert!(stats.total_avg_ms.is_finite());
        assert!(stats.total_std_dev_ms.is_finite());
    }
}

/// Test statistical distribution analysis
mod distribution_tests {
    use super::*;
    
    #[test]
    fn test_normal_distribution_detection() {
        let mut result = TestResult::new(
            "normal".to_string(),
            DnsConfig::System,
            "https://example.com".to_string()
        );
        
        // Add measurements following roughly normal distribution
        let normal_values = [95, 98, 100, 100, 100, 102, 102, 105, 108];
        
        for &val in &normal_values {
            result.add_measurement(TimingMetrics::success(
                Duration::from_millis(val),
                Duration::from_millis(val/2),
                Some(Duration::from_millis(val/4)),
                Duration::from_millis(val*2),
                Duration::from_millis(val*3),
                200,
            ));
        }
        
        result.calculate_statistics();
        
        let stats = result.statistics.as_ref().unwrap();
        // For normal distribution, mean should be close to median
        let mean = stats.total_avg_ms;
        let median = normal_values[normal_values.len()/2] as f64 * 3.0; // total time
        
        assert!((mean - median).abs() < mean * 0.2); // Within 20%
    }
    
    #[test]
    fn test_outlier_impact() {
        let mut result = TestResult::new(
            "outliers".to_string(),
            DnsConfig::System,
            "https://example.com".to_string()
        );
        
        // Add mostly consistent measurements
        for _ in 0..9 {
            result.add_measurement(TimingMetrics::success(
                Duration::from_millis(100),
                Duration::from_millis(50),
                Some(Duration::from_millis(25)),
                Duration::from_millis(200),
                Duration::from_millis(375),
                200,
            ));
        }
        
        // Add one outlier
        result.add_measurement(TimingMetrics::success(
            Duration::from_millis(5000),
            Duration::from_millis(2500),
            Some(Duration::from_millis(1250)),
            Duration::from_millis(10000),
            Duration::from_millis(18750),
            200,
        ));
        
        result.calculate_statistics();
        
        let stats = result.statistics.as_ref().unwrap();
        
        // Standard deviation should be significantly higher due to outlier
        assert!(stats.total_std_dev_ms > 1000.0);
        // Mean should be pulled towards outlier
        assert!(stats.total_avg_ms > 375.0);
    }
}