//! Platform-specific timeout behavior handling and optimization
//!
//! This module provides adaptive timeout management that accounts for
//! platform-specific networking characteristics and performance variations.

use crate::{
    client::platform::{PlatformNetworkConfig, NetworkOperation},
    types::DnsConfig,
};
use std::{
    time::Duration,
    collections::HashMap,
};

/// Platform-specific timeout manager
pub struct TimeoutManager {
    config: PlatformNetworkConfig,
    adaptive_timeouts: HashMap<String, AdaptiveTimeout>,
    baseline_measurements: HashMap<NetworkOperation, Duration>,
}

impl TimeoutManager {
    /// Create a new timeout manager with platform-specific defaults
    pub fn new() -> Self {
        let config = PlatformNetworkConfig::for_current_platform();
        Self {
            config,
            adaptive_timeouts: HashMap::new(),
            baseline_measurements: HashMap::new(),
        }
    }

    /// Create with custom platform configuration
    pub fn with_config(config: PlatformNetworkConfig) -> Self {
        Self {
            config,
            adaptive_timeouts: HashMap::new(),
            baseline_measurements: HashMap::new(),
        }
    }

    /// Get platform-optimized timeout for a specific operation and target
    pub fn get_timeout(&mut self, operation: NetworkOperation, target: &str, dns_config: &DnsConfig) -> Duration {
        let base_timeout = self.config.get_timeout_for_operation(operation);
        
        // Apply DNS-specific adjustments
        let dns_adjusted = self.adjust_for_dns_config(base_timeout, dns_config);
        
        // Apply adaptive adjustments based on historical performance
        let adaptive_key = format!("{}:{:?}", target, operation);
        if let Some(adaptive_timeout) = self.adaptive_timeouts.get(&adaptive_key) {
            adaptive_timeout.get_recommended_timeout(dns_adjusted)
        } else {
            // Apply platform-specific multipliers for first-time targets
            self.apply_platform_multipliers(dns_adjusted, operation, target)
        }
    }

    /// Record actual operation duration for timeout adaptation
    pub fn record_operation_time(&mut self, operation: NetworkOperation, target: &str, actual_duration: Duration, success: bool) {
        let adaptive_key = format!("{}:{:?}", target, operation);
        
        let adaptive_timeout = self.adaptive_timeouts
            .entry(adaptive_key)
            .or_insert_with(|| AdaptiveTimeout::new(operation));
        
        adaptive_timeout.record_measurement(actual_duration, success);
        
        // Update baseline measurements
        self.update_baseline_measurements(operation, actual_duration);
    }

    /// Adjust timeout based on DNS configuration type
    fn adjust_for_dns_config(&self, base_timeout: Duration, dns_config: &DnsConfig) -> Duration {
        match dns_config {
            DnsConfig::System => base_timeout,
            DnsConfig::Custom { .. } => {
                // Custom DNS servers might be slower
                base_timeout + Duration::from_millis(500)
            }
            DnsConfig::DoH { .. } => {
                // DNS-over-HTTPS has HTTPS overhead
                base_timeout * 2
            }
        }
    }

    /// Apply platform-specific timeout multipliers
    fn apply_platform_multipliers(&self, base_timeout: Duration, operation: NetworkOperation, target: &str) -> Duration {
        let mut timeout = base_timeout;

        // Platform-specific adjustments
        #[cfg(target_os = "windows")]
        {
            timeout = self.apply_windows_multipliers(timeout, operation);
        }

        #[cfg(target_os = "macos")]
        {
            timeout = self.apply_macos_multipliers(timeout, operation);
        }

        #[cfg(target_os = "linux")]
        {
            timeout = self.apply_linux_multipliers(timeout, operation);
        }

        // Target-specific adjustments
        timeout = self.apply_target_adjustments(timeout, target);

        timeout
    }

    /// Apply Windows-specific timeout multipliers
    #[cfg(target_os = "windows")]
    fn apply_windows_multipliers(&self, timeout: Duration, operation: NetworkOperation) -> Duration {
        match operation {
            NetworkOperation::DnsResolution => timeout * 2, // Windows DNS can be slow
            NetworkOperation::TcpConnection => timeout + Duration::from_secs(1),
            NetworkOperation::TlsHandshake => timeout + Duration::from_secs(2),
            NetworkOperation::HttpRequest => timeout + Duration::from_secs(1),
            NetworkOperation::DataTransfer => timeout,
        }
    }

    /// Apply macOS-specific timeout multipliers
    #[cfg(target_os = "macos")]
    fn apply_macos_multipliers(&self, timeout: Duration, operation: NetworkOperation) -> Duration {
        match operation {
            NetworkOperation::DnsResolution => timeout, // macOS DNS is generally fast
            NetworkOperation::TcpConnection => timeout,
            NetworkOperation::TlsHandshake => timeout,
            NetworkOperation::HttpRequest => timeout,
            NetworkOperation::DataTransfer => timeout,
        }
    }

    /// Apply Linux-specific timeout multipliers
    #[cfg(target_os = "linux")]
    fn apply_linux_multipliers(&self, timeout: Duration, operation: NetworkOperation) -> Duration {
        match operation {
            NetworkOperation::DnsResolution => timeout, // Linux DNS is generally fast
            NetworkOperation::TcpConnection => timeout,
            NetworkOperation::TlsHandshake => timeout,
            NetworkOperation::HttpRequest => timeout,
            NetworkOperation::DataTransfer => timeout,
        }
    }

    /// Apply target-specific timeout adjustments
    fn apply_target_adjustments(&self, timeout: Duration, target: &str) -> Duration {
        // Adjust for known slow or fast targets
        if target.contains("localhost") || target.contains("127.0.0.1") || target.contains("::1") {
            // Local targets should be fast
            Duration::min(timeout, Duration::from_secs(2))
        } else if target.contains("amazonaws.com") || target.contains("cloudflare.com") {
            // Cloud providers are usually fast
            timeout
        } else if target.contains("github.com") || target.contains("google.com") {
            // Major services are usually reliable
            timeout
        } else {
            // Unknown targets might need more time
            timeout + Duration::from_secs(1)
        }
    }

    /// Update baseline measurements for an operation type
    fn update_baseline_measurements(&mut self, operation: NetworkOperation, duration: Duration) {
        let current_baseline = self.baseline_measurements.entry(operation).or_insert(duration);
        
        // Use exponential moving average to update baseline
        let alpha = 0.1; // Smoothing factor
        let new_baseline_secs = (1.0 - alpha) * current_baseline.as_secs_f64() + alpha * duration.as_secs_f64();
        *current_baseline = Duration::from_secs_f64(new_baseline_secs);
    }

    /// Get timeout statistics for analysis
    pub fn get_timeout_statistics(&self) -> TimeoutStatistics {
        TimeoutStatistics {
            platform: crate::dns::platform::get_platform_name(),
            baseline_measurements: self.baseline_measurements.clone(),
            adaptive_timeout_count: self.adaptive_timeouts.len(),
            total_measurements: self.adaptive_timeouts.values()
                .map(|t| t.measurement_count)
                .sum(),
            average_success_rate: if !self.adaptive_timeouts.is_empty() {
                self.adaptive_timeouts.values()
                    .map(|t| t.success_rate)
                    .sum::<f64>() / self.adaptive_timeouts.len() as f64
            } else {
                0.0
            },
        }
    }

    /// Reset adaptive timeouts (useful for testing)
    pub fn reset_adaptive_timeouts(&mut self) {
        self.adaptive_timeouts.clear();
    }

    /// Get recommended timeout for a new, unknown target
    pub fn get_conservative_timeout(&self, operation: NetworkOperation) -> Duration {
        let base_timeout = self.config.get_timeout_for_operation(operation);
        
        // Apply platform-specific conservative multipliers
        #[cfg(target_os = "windows")]
        {
            base_timeout * 2
        }
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            base_timeout + Duration::from_secs(2)
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            base_timeout * 3
        }
    }
}

impl Default for TimeoutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Adaptive timeout for a specific operation and target
struct AdaptiveTimeout {
    #[allow(dead_code)]
    operation: NetworkOperation,
    measurements: Vec<Duration>,
    success_count: usize,
    failure_count: usize,
    last_successful_timeout: Option<Duration>,
    measurement_count: usize,
    success_rate: f64,
}

impl AdaptiveTimeout {
    /// Create a new adaptive timeout tracker
    fn new(operation: NetworkOperation) -> Self {
        Self {
            operation,
            measurements: Vec::new(),
            success_count: 0,
            failure_count: 0,
            last_successful_timeout: None,
            measurement_count: 0,
            success_rate: 0.0,
        }
    }

    /// Record a measurement
    fn record_measurement(&mut self, duration: Duration, success: bool) {
        self.measurements.push(duration);
        self.measurement_count += 1;

        if success {
            self.success_count += 1;
            self.last_successful_timeout = Some(duration);
        } else {
            self.failure_count += 1;
        }

        // Keep only recent measurements (last 50)
        if self.measurements.len() > 50 {
            self.measurements.remove(0);
        }

        // Update success rate
        self.success_rate = self.success_count as f64 / self.measurement_count as f64;
    }

    /// Get recommended timeout based on historical data
    fn get_recommended_timeout(&self, fallback_timeout: Duration) -> Duration {
        if self.measurements.is_empty() {
            return fallback_timeout;
        }

        // Calculate percentile-based timeout (95th percentile)
        let mut sorted_measurements = self.measurements.clone();
        sorted_measurements.sort();
        
        let percentile_95_index = ((sorted_measurements.len() as f64) * 0.95) as usize;
        let percentile_95_timeout = sorted_measurements.get(percentile_95_index.min(sorted_measurements.len() - 1))
            .copied()
            .unwrap_or(fallback_timeout);

        // Apply success rate adjustment
        let success_rate_multiplier = if self.success_rate < 0.5 {
            2.0 // Double timeout if success rate is low
        } else if self.success_rate < 0.8 {
            1.5 // 50% increase if success rate is moderate
        } else {
            1.1 // Small increase if success rate is good
        };

        let adjusted_timeout = Duration::from_secs_f64(
            percentile_95_timeout.as_secs_f64() * success_rate_multiplier
        );

        // Ensure timeout is within reasonable bounds
        let max_timeout = fallback_timeout * 5;
        let min_timeout = fallback_timeout / 2;

        Duration::min(max_timeout, Duration::max(min_timeout, adjusted_timeout))
    }
}

/// Statistics about timeout behavior
#[derive(Debug, Clone)]
pub struct TimeoutStatistics {
    pub platform: String,
    pub baseline_measurements: HashMap<NetworkOperation, Duration>,
    pub adaptive_timeout_count: usize,
    pub total_measurements: usize,
    pub average_success_rate: f64,
}

impl TimeoutStatistics {
    /// Generate a statistics report
    pub fn generate_report(&self) -> String {
        let mut report = format!("Timeout Statistics for {}:\n", self.platform);
        report.push_str(&format!("  Adaptive Timeouts: {}\n", self.adaptive_timeout_count));
        report.push_str(&format!("  Total Measurements: {}\n", self.total_measurements));
        report.push_str(&format!("  Average Success Rate: {:.1}%\n", self.average_success_rate * 100.0));
        
        report.push_str("\n  Baseline Measurements:\n");
        for (operation, duration) in &self.baseline_measurements {
            report.push_str(&format!("    {:?}: {:?}\n", operation, duration));
        }

        report
    }

    /// Check if timeout performance is good
    pub fn has_good_timeout_performance(&self) -> bool {
        self.average_success_rate >= 0.8 && self.total_measurements >= 10
    }
}

/// Timeout optimization recommendations
pub struct TimeoutOptimizer {
    #[allow(dead_code)]
    manager: TimeoutManager,
}

impl TimeoutOptimizer {
    /// Create a new timeout optimizer
    pub fn new() -> Self {
        Self {
            manager: TimeoutManager::new(),
        }
    }

    /// Analyze timeout patterns and provide recommendations
    pub fn analyze_and_recommend(&self, statistics: &TimeoutStatistics) -> TimeoutRecommendations {
        let mut recommendations = TimeoutRecommendations {
            platform: statistics.platform.clone(),
            recommendations: Vec::new(),
            suggested_adjustments: HashMap::new(),
        };

        // Analyze success rate
        if statistics.average_success_rate < 0.5 {
            recommendations.recommendations.push(
                "Success rate is very low (<50%). Consider increasing timeouts significantly.".to_string()
            );
        } else if statistics.average_success_rate < 0.8 {
            recommendations.recommendations.push(
                "Success rate is moderate (<80%). Consider slight timeout increases.".to_string()
            );
        } else {
            recommendations.recommendations.push(
                "Success rate is good (≥80%). Current timeouts appear appropriate.".to_string()
            );
        }

        // Analyze measurement count
        if statistics.total_measurements < 10 {
            recommendations.recommendations.push(
                "Not enough measurements for reliable timeout adaptation. Continue testing.".to_string()
            );
        }

        // Platform-specific recommendations
        recommendations.recommendations.extend(self.get_platform_specific_recommendations(&statistics.platform));

        // Suggest baseline adjustments based on measurements
        for (operation, baseline) in &statistics.baseline_measurements {
            let suggested_timeout = if statistics.average_success_rate < 0.7 {
                *baseline + Duration::from_secs(2)
            } else {
                *baseline
            };
            
            recommendations.suggested_adjustments.insert(*operation, suggested_timeout);
        }

        recommendations
    }

    /// Get platform-specific timeout recommendations
    fn get_platform_specific_recommendations(&self, platform: &str) -> Vec<String> {
        match platform {
            "Windows" => vec![
                "Windows networking can be inconsistent. Use higher timeouts for reliability.".to_string(),
                "Consider Windows Firewall and antivirus impacts on network performance.".to_string(),
            ],
            "macOS" => vec![
                "macOS has good networking performance. Timeouts can be more aggressive.".to_string(),
                "IPv6 connectivity is generally excellent on macOS.".to_string(),
            ],
            "Linux" => vec![
                "Linux networking is generally fast and reliable.".to_string(),
                "Consider systemd-resolved configuration for DNS optimization.".to_string(),
            ],
            _ => vec![
                "Unknown platform. Use conservative timeout values.".to_string(),
            ],
        }
    }
}

impl Default for TimeoutOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Timeout optimization recommendations
#[derive(Debug, Clone)]
pub struct TimeoutRecommendations {
    pub platform: String,
    pub recommendations: Vec<String>,
    pub suggested_adjustments: HashMap<NetworkOperation, Duration>,
}

impl TimeoutRecommendations {
    /// Generate a recommendations report
    pub fn generate_report(&self) -> String {
        let mut report = format!("Timeout Recommendations for {}:\n\n", self.platform);
        
        report.push_str("General Recommendations:\n");
        for (i, recommendation) in self.recommendations.iter().enumerate() {
            report.push_str(&format!("  {}. {}\n", i + 1, recommendation));
        }

        if !self.suggested_adjustments.is_empty() {
            report.push_str("\nSuggested Timeout Adjustments:\n");
            for (operation, timeout) in &self.suggested_adjustments {
                report.push_str(&format!("  {:?}: {:?}\n", operation, timeout));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_manager_creation() {
        let manager = TimeoutManager::new();
        drop(manager);
    }

    #[test]
    fn test_timeout_calculation() {
        let mut manager = TimeoutManager::new();
        
        let timeout = manager.get_timeout(
            NetworkOperation::HttpRequest,
            "https://example.com",
            &DnsConfig::System
        );
        
        assert!(timeout > Duration::from_secs(0));
    }

    #[test]
    fn test_dns_config_timeout_adjustments() {
        let mut manager = TimeoutManager::new();
        
        let system_timeout = manager.get_timeout(
            NetworkOperation::DnsResolution,
            "example.com",
            &DnsConfig::System
        );
        
        let custom_timeout = manager.get_timeout(
            NetworkOperation::DnsResolution,
            "example.com",
            &DnsConfig::Custom { servers: vec!["8.8.8.8".parse().unwrap()] }
        );
        
        let doh_timeout = manager.get_timeout(
            NetworkOperation::DnsResolution,
            "example.com",
            &DnsConfig::DoH { url: "https://dns.google/dns-query".to_string() }
        );

        assert!(custom_timeout >= system_timeout);
        assert!(doh_timeout > system_timeout);
    }

    #[test]
    fn test_adaptive_timeout_recording() {
        let mut manager = TimeoutManager::new();
        
        // Record some successful operations
        manager.record_operation_time(
            NetworkOperation::HttpRequest,
            "example.com",
            Duration::from_millis(200),
            true
        );
        
        manager.record_operation_time(
            NetworkOperation::HttpRequest,
            "example.com",
            Duration::from_millis(300),
            true
        );
        
        // Second call should use adaptive timeout
        let timeout = manager.get_timeout(
            NetworkOperation::HttpRequest,
            "example.com",
            &DnsConfig::System
        );
        
        assert!(timeout > Duration::from_secs(0));
        
        let stats = manager.get_timeout_statistics();
        assert_eq!(stats.total_measurements, 2);
        assert!(stats.average_success_rate > 0.0);
    }

    #[test]
    fn test_conservative_timeout() {
        let manager = TimeoutManager::new();
        let timeout = manager.get_conservative_timeout(NetworkOperation::HttpRequest);
        
        let base_timeout = manager.config.get_timeout_for_operation(NetworkOperation::HttpRequest);
        assert!(timeout >= base_timeout);
    }

    #[test]
    fn test_adaptive_timeout_internal() {
        let mut adaptive = AdaptiveTimeout::new(NetworkOperation::HttpRequest);
        
        adaptive.record_measurement(Duration::from_millis(100), true);
        adaptive.record_measurement(Duration::from_millis(200), true);
        adaptive.record_measurement(Duration::from_millis(150), false);
        
        assert_eq!(adaptive.success_count, 2);
        assert_eq!(adaptive.failure_count, 1);
        assert_eq!(adaptive.measurement_count, 3);
        
        let recommended = adaptive.get_recommended_timeout(Duration::from_secs(5));
        assert!(recommended > Duration::from_secs(0));
    }

    #[test]
    fn test_timeout_statistics() {
        let stats = TimeoutStatistics {
            platform: "Test".to_string(),
            baseline_measurements: {
                let mut map = HashMap::new();
                map.insert(NetworkOperation::HttpRequest, Duration::from_secs(5));
                map
            },
            adaptive_timeout_count: 5,
            total_measurements: 50,
            average_success_rate: 0.85,
        };

        let report = stats.generate_report();
        assert!(report.contains("Test"));
        assert!(report.contains("85.0%"));
        assert!(stats.has_good_timeout_performance());
        
        let poor_stats = TimeoutStatistics {
            platform: "Test".to_string(),
            baseline_measurements: HashMap::new(),
            adaptive_timeout_count: 0,
            total_measurements: 5,
            average_success_rate: 0.5,
        };

        assert!(!poor_stats.has_good_timeout_performance());
    }

    #[test]
    fn test_timeout_optimizer() {
        let optimizer = TimeoutOptimizer::new();
        let stats = TimeoutStatistics {
            platform: "Linux".to_string(),
            baseline_measurements: HashMap::new(),
            adaptive_timeout_count: 10,
            total_measurements: 100,
            average_success_rate: 0.9,
        };

        let recommendations = optimizer.analyze_and_recommend(&stats);
        assert!(recommendations.recommendations.len() > 0);
        
        let report = recommendations.generate_report();
        assert!(report.contains("Linux"));
    }

    #[test]
    fn test_timeout_manager_reset() {
        let mut manager = TimeoutManager::new();
        
        manager.record_operation_time(
            NetworkOperation::HttpRequest,
            "example.com",
            Duration::from_millis(100),
            true
        );
        
        let stats_before = manager.get_timeout_statistics();
        assert!(stats_before.total_measurements > 0);
        
        manager.reset_adaptive_timeouts();
        
        let stats_after = manager.get_timeout_statistics();
        assert_eq!(stats_after.adaptive_timeout_count, 0);
    }

    #[test]
    fn test_platform_specific_multipliers() {
        let manager = TimeoutManager::new();
        let base_timeout = Duration::from_secs(5);
        
        // Test local target adjustment
        let local_timeout = manager.apply_target_adjustments(base_timeout, "localhost");
        assert!(local_timeout <= Duration::from_secs(2));
        
        // Test unknown target adjustment
        let unknown_timeout = manager.apply_target_adjustments(base_timeout, "unknown.example.com");
        assert!(unknown_timeout > base_timeout);
    }
}