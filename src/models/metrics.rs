//! Timing metrics and test result data models

use crate::types::{DnsConfig, TestStatus, PerformanceLevel};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};

/// Detailed timing metrics for a single HTTP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMetrics {
    /// Time taken for DNS resolution
    pub dns_resolution: Duration,
    
    /// Time taken to establish TCP connection
    pub tcp_connection: Duration,
    
    /// Time taken for TLS handshake (if HTTPS)
    pub tls_handshake: Option<Duration>,
    
    /// Time from request sent to first byte received
    pub first_byte: Duration,
    
    /// Total request duration
    pub total_duration: Duration,
    
    /// HTTP status code received
    pub http_status: u16,
    
    /// Test execution status
    pub status: TestStatus,
    
    /// Timestamp when the test was executed
    pub timestamp: DateTime<Utc>,
    
    /// Error message if the test failed
    pub error_message: Option<String>,
}

impl TimingMetrics {
    /// Create a new successful timing metrics instance
    pub fn success(
        dns_resolution: Duration,
        tcp_connection: Duration,
        tls_handshake: Option<Duration>,
        first_byte: Duration,
        total_duration: Duration,
        http_status: u16,
    ) -> Self {
        Self {
            dns_resolution,
            tcp_connection,
            tls_handshake,
            first_byte,
            total_duration,
            http_status,
            status: TestStatus::Success,
            timestamp: Utc::now(),
            error_message: None,
        }
    }
    
    /// Create a new failed timing metrics instance
    pub fn failed(error_message: String) -> Self {
        Self {
            dns_resolution: Duration::ZERO,
            tcp_connection: Duration::ZERO,
            tls_handshake: None,
            first_byte: Duration::ZERO,
            total_duration: Duration::ZERO,
            http_status: 0,
            status: TestStatus::Failed,
            timestamp: Utc::now(),
            error_message: Some(error_message),
        }
    }
    
    /// Create a new timeout timing metrics instance
    pub fn timeout(timeout_duration: Duration) -> Self {
        Self {
            dns_resolution: Duration::ZERO,
            tcp_connection: Duration::ZERO,
            tls_handshake: None,
            first_byte: Duration::ZERO,
            total_duration: timeout_duration,
            http_status: 0,
            status: TestStatus::Timeout,
            timestamp: Utc::now(),
            error_message: Some(format!("Request timed out after {}s", timeout_duration.as_secs())),
        }
    }
    
    /// Create a skipped test instance
    pub fn skipped(reason: String) -> Self {
        Self {
            dns_resolution: Duration::ZERO,
            tcp_connection: Duration::ZERO,
            tls_handshake: None,
            first_byte: Duration::ZERO,
            total_duration: Duration::ZERO,
            http_status: 0,
            status: TestStatus::Skipped,
            timestamp: Utc::now(),
            error_message: Some(reason),
        }
    }
    
    /// Check if this test was successful
    pub fn is_successful(&self) -> bool {
        matches!(self.status, TestStatus::Success) && self.http_status >= 200 && self.http_status < 400
    }
    
    /// Get the performance level based on total duration
    pub fn performance_level(&self) -> PerformanceLevel {
        PerformanceLevel::from_duration(self.total_duration)
    }
    
    /// Format timing as milliseconds
    pub fn dns_ms(&self) -> f64 {
        self.dns_resolution.as_secs_f64() * 1000.0
    }
    
    pub fn tcp_ms(&self) -> f64 {
        self.tcp_connection.as_secs_f64() * 1000.0
    }
    
    pub fn tls_ms(&self) -> Option<f64> {
        self.tls_handshake.map(|d| d.as_secs_f64() * 1000.0)
    }
    
    pub fn first_byte_ms(&self) -> f64 {
        self.first_byte.as_secs_f64() * 1000.0
    }
    
    pub fn total_ms(&self) -> f64 {
        self.total_duration.as_secs_f64() * 1000.0
    }
}

/// Results from testing a single DNS configuration against a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Human-readable name of the DNS configuration
    pub config_name: String,
    
    /// DNS configuration that was tested
    pub dns_config: DnsConfig,
    
    /// Target URL that was tested
    pub url: String,
    
    /// Individual test results (one per iteration)
    pub individual_results: Vec<TimingMetrics>,
    
    /// Calculated statistics from successful tests
    pub statistics: Option<Statistics>,
    
    /// Number of successful tests
    pub success_count: u32,
    
    /// Total number of tests attempted
    pub total_count: u32,
    
    /// When the test batch started
    pub started_at: DateTime<Utc>,
    
    /// When the test batch completed
    pub completed_at: Option<DateTime<Utc>>,
}

impl TestResult {
    /// Create a new test result
    pub fn new(config_name: String, dns_config: DnsConfig, url: String) -> Self {
        Self {
            config_name,
            dns_config,
            url,
            individual_results: Vec::new(),
            statistics: None,
            success_count: 0,
            total_count: 0,
            started_at: Utc::now(),
            completed_at: None,
        }
    }
    
    /// Add a timing measurement to this result
    pub fn add_measurement(&mut self, metrics: TimingMetrics) {
        if metrics.is_successful() {
            self.success_count += 1;
        }
        self.total_count += 1;
        self.individual_results.push(metrics);
    }
    
    /// Calculate and update statistics from successful measurements
    pub fn calculate_statistics(&mut self) {
        let successful_results: Vec<&TimingMetrics> = self
            .individual_results
            .iter()
            .filter(|m| m.is_successful())
            .collect();
        
        if !successful_results.is_empty() {
            self.statistics = Some(Statistics::from_measurements(&successful_results));
        }
        
        self.completed_at = Some(Utc::now());
    }
    
    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.total_count as f64) * 100.0
        }
    }
    
    /// Get overall performance level
    pub fn performance_level(&self) -> Option<PerformanceLevel> {
        self.statistics.as_ref().map(|s| s.performance_level())
    }
    
    /// Check if any tests were skipped
    pub fn has_skipped_tests(&self) -> bool {
        self.individual_results
            .iter()
            .any(|m| matches!(m.status, TestStatus::Skipped))
    }
}

/// Statistical analysis of timing measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    /// Average DNS resolution time (milliseconds)
    pub dns_avg_ms: f64,
    
    /// Average TCP connection time (milliseconds)
    pub tcp_avg_ms: f64,
    
    /// Average first byte time (milliseconds)
    pub first_byte_avg_ms: f64,
    
    /// Average total time (milliseconds)
    pub total_avg_ms: f64,
    
    /// Minimum total time (milliseconds)
    pub total_min_ms: f64,
    
    /// Maximum total time (milliseconds)
    pub total_max_ms: f64,
    
    /// Standard deviation of total times (milliseconds)
    pub total_std_dev_ms: f64,
    
    /// Success rate percentage (0.0-100.0)
    pub success_rate: f64,
    
    /// Number of successful measurements included in statistics
    pub sample_count: usize,
}

impl Statistics {
    /// Calculate statistics from a collection of successful timing measurements
    pub fn from_measurements(measurements: &[&TimingMetrics]) -> Self {
        let count = measurements.len();
        
        if count == 0 {
            return Self::empty();
        }
        
        // Calculate averages
        let dns_sum: f64 = measurements.iter().map(|m| m.dns_ms()).sum();
        let tcp_sum: f64 = measurements.iter().map(|m| m.tcp_ms()).sum();
        let first_byte_sum: f64 = measurements.iter().map(|m| m.first_byte_ms()).sum();
        let total_sum: f64 = measurements.iter().map(|m| m.total_ms()).sum();
        
        let dns_avg = dns_sum / count as f64;
        let tcp_avg = tcp_sum / count as f64;
        let first_byte_avg = first_byte_sum / count as f64;
        let total_avg = total_sum / count as f64;
        
        // Calculate min and max for total time
        let total_times: Vec<f64> = measurements.iter().map(|m| m.total_ms()).collect();
        let total_min = total_times.iter().cloned().fold(f64::INFINITY, f64::min);
        let total_max = total_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        // Calculate standard deviation for total time
        let variance = if count > 1 {
            let sum_squared_diff: f64 = total_times
                .iter()
                .map(|&x| (x - total_avg).powi(2))
                .sum();
            sum_squared_diff / count as f64
        } else {
            0.0
        };
        let std_dev = variance.sqrt();
        
        Self {
            dns_avg_ms: dns_avg,
            tcp_avg_ms: tcp_avg,
            first_byte_avg_ms: first_byte_avg,
            total_avg_ms: total_avg,
            total_min_ms: total_min,
            total_max_ms: total_max,
            total_std_dev_ms: std_dev,
            success_rate: 100.0, // All measurements passed in are successful
            sample_count: count,
        }
    }
    
    /// Create empty statistics
    pub fn empty() -> Self {
        Self {
            dns_avg_ms: 0.0,
            tcp_avg_ms: 0.0,
            first_byte_avg_ms: 0.0,
            total_avg_ms: 0.0,
            total_min_ms: 0.0,
            total_max_ms: 0.0,
            total_std_dev_ms: 0.0,
            success_rate: 0.0,
            sample_count: 0,
        }
    }
    
    /// Get performance level based on average total time
    pub fn performance_level(&self) -> PerformanceLevel {
        PerformanceLevel::from_duration(Duration::from_millis(self.total_avg_ms as u64))
    }
    
    /// Check if statistics indicate poor success rate
    pub fn has_poor_success_rate(&self) -> bool {
        self.success_rate < 80.0
    }
    
    /// Format average total time for display
    pub fn format_avg_total(&self) -> String {
        format!("{:.1}ms", self.total_avg_ms)
    }
}

/// Helper for building timing measurements during HTTP requests
#[derive(Debug)]
pub struct TimingBuilder {
    start_time: Instant,
    dns_start: Option<Instant>,
    dns_end: Option<Instant>,
    connect_start: Option<Instant>,
    connect_end: Option<Instant>,
    tls_start: Option<Instant>,
    tls_end: Option<Instant>,
    first_byte_time: Option<Instant>,
}

impl TimingBuilder {
    /// Create a new timing builder
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            dns_start: None,
            dns_end: None,
            connect_start: None,
            connect_end: None,
            tls_start: None,
            tls_end: None,
            first_byte_time: None,
        }
    }
    
    /// Mark DNS resolution start
    pub fn dns_start(&mut self) {
        self.dns_start = Some(Instant::now());
    }
    
    /// Mark DNS resolution end
    pub fn dns_end(&mut self) {
        self.dns_end = Some(Instant::now());
    }
    
    /// Mark TCP connection start
    pub fn connect_start(&mut self) {
        self.connect_start = Some(Instant::now());
    }
    
    /// Mark TCP connection end
    pub fn connect_end(&mut self) {
        self.connect_end = Some(Instant::now());
    }
    
    /// Mark TLS handshake start
    pub fn tls_start(&mut self) {
        self.tls_start = Some(Instant::now());
    }
    
    /// Mark TLS handshake end
    pub fn tls_end(&mut self) {
        self.tls_end = Some(Instant::now());
    }
    
    /// Mark first byte received
    pub fn first_byte(&mut self) {
        self.first_byte_time = Some(Instant::now());
    }
    
    /// Build the final timing metrics
    pub fn build(self, http_status: u16) -> TimingMetrics {
        let end_time = Instant::now();
        
        let dns_duration = match (self.dns_start, self.dns_end) {
            (Some(start), Some(end)) => end.duration_since(start),
            _ => Duration::ZERO,
        };
        
        let tcp_duration = match (self.connect_start, self.connect_end) {
            (Some(start), Some(end)) => end.duration_since(start),
            _ => Duration::ZERO,
        };
        
        let tls_duration = match (self.tls_start, self.tls_end) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        };
        
        let first_byte_duration = match self.first_byte_time {
            Some(first_byte) => first_byte.duration_since(self.start_time),
            None => Duration::ZERO,
        };
        
        let total_duration = end_time.duration_since(self.start_time);
        
        TimingMetrics::success(
            dns_duration,
            tcp_duration,
            tls_duration,
            first_byte_duration,
            total_duration,
            http_status,
        )
    }
}

impl Default for TimingBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_timing_metrics_success() {
        let metrics = TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            Some(Duration::from_millis(30)),
            Duration::from_millis(50),
            Duration::from_millis(100),
            200,
        );
        
        assert!(metrics.is_successful());
        assert_eq!(metrics.performance_level(), PerformanceLevel::Good);
        assert_eq!(metrics.dns_ms(), 10.0);
        assert_eq!(metrics.total_ms(), 100.0);
    }
    
    #[test]
    fn test_timing_metrics_failed() {
        let metrics = TimingMetrics::failed("Connection refused".to_string());
        
        assert!(!metrics.is_successful());
        assert_eq!(metrics.status, TestStatus::Failed);
        assert!(metrics.error_message.is_some());
    }
    
    #[test]
    fn test_test_result_statistics() {
        let mut result = TestResult::new(
            "Test Config".to_string(),
            DnsConfig::System,
            "https://example.com".to_string(),
        );
        
        // Add some successful measurements
        result.add_measurement(TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            None,
            Duration::from_millis(50),
            Duration::from_millis(100),
            200,
        ));
        
        result.add_measurement(TimingMetrics::success(
            Duration::from_millis(15),
            Duration::from_millis(25),
            None,
            Duration::from_millis(60),
            Duration::from_millis(120),
            200,
        ));
        
        result.calculate_statistics();
        
        assert_eq!(result.success_count, 2);
        assert_eq!(result.total_count, 2);
        assert_eq!(result.success_rate(), 100.0);
        
        let stats = result.statistics.as_ref().unwrap();
        assert_eq!(stats.total_avg_ms, 110.0);
        assert_eq!(stats.sample_count, 2);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let m1 = TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            None,
            Duration::from_millis(50),
            Duration::from_millis(100),
            200,
        );
        
        let m2 = TimingMetrics::success(
            Duration::from_millis(15),
            Duration::from_millis(25),
            None,
            Duration::from_millis(60),
            Duration::from_millis(200),
            200,
        );
        
        let measurements = vec![&m1, &m2];
        let stats = Statistics::from_measurements(&measurements);
        
        assert_eq!(stats.total_avg_ms, 150.0);
        assert_eq!(stats.total_min_ms, 100.0);
        assert_eq!(stats.total_max_ms, 200.0);
        assert_eq!(stats.sample_count, 2);
    }
    
    #[test]
    fn test_timing_builder() {
        let mut builder = TimingBuilder::new();
        
        builder.dns_start();
        thread::sleep(Duration::from_millis(1));
        builder.dns_end();
        
        builder.connect_start();
        thread::sleep(Duration::from_millis(1));
        builder.connect_end();
        
        builder.first_byte();
        
        let metrics = builder.build(200);
        
        assert!(metrics.is_successful());
        assert!(metrics.dns_resolution > Duration::ZERO);
        assert!(metrics.tcp_connection > Duration::ZERO);
        assert!(metrics.total_duration > Duration::ZERO);
    }
}