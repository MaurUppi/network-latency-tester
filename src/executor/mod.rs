//! Test execution engine with performance optimization
//!
//! This module contains the core test execution components including:
//! - Optimized executor with connection pooling and system resource detection
//! - Concurrent execution tuning based on real-time performance feedback
//! - Memory-efficient execution strategies

pub mod optimized;
pub mod tuning;

pub use optimized::{
    OptimizedExecutor, ClientPool, PoolConfig, SystemResources,
    ExecutorStats, PoolStats,
};

pub use tuning::{
    ConcurrencyTuner, ExecutionParameters, TuningConfig, TuningStatistics,
};

// Re-export new execution result types - no need for self:: since they're defined in this module

use crate::{
    error::Result,
    models::{Config, TestResult},
    types::DnsConfig,
    stats::StatisticalAnalysis,
    diagnostics::DiagnosticReport,
};
use std::time::Duration;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Basic execution configuration for the test executor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Number of test iterations to perform
    pub test_count: u32,
    /// Timeout for individual requests
    pub timeout: Duration,
    /// Enable verbose output during execution
    pub verbose: bool,
    /// Enable debug output during execution
    pub debug: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            test_count: 5,
            timeout: Duration::from_secs(10),
            verbose: false,
            debug: false,
        }
    }
}

/// Summary of test execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// Total execution duration
    pub total_duration: Duration,
    /// Total number of tests executed
    pub total_tests: u32,
    /// Number of successful tests
    pub successful_tests: u32,
    /// Number of failed tests
    pub failed_tests: u32,
    /// Number of timed out tests
    pub timeout_tests: u32,
    /// Number of skipped tests
    pub skipped_tests: u32,
    /// Overall success rate (percentage)
    pub success_rate: f64,
    /// Performance summary by configuration
    pub performance_summary: HashMap<String, ConfigPerformance>,
}

/// Performance metrics for a specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPerformance {
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// Success rate for this configuration
    pub success_rate: f64,
    /// Number of tests for this configuration
    pub test_count: u32,
}

/// Complete execution results including all analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResults {
    /// High-level execution summary
    pub execution_summary: ExecutionSummary,
    /// Detailed test results by configuration
    pub test_results: HashMap<String, TestResult>,
    /// Statistical analysis (if generated)
    pub statistical_analysis: Option<StatisticalAnalysis>,
    /// Network diagnostics report (if generated)
    pub diagnostics_report: Option<DiagnosticReport>,
}

impl ExecutionResults {
    /// Create a new ExecutionResults with the given summary and test results
    pub fn new(execution_summary: ExecutionSummary, test_results: HashMap<String, TestResult>) -> Self {
        Self {
            execution_summary,
            test_results,
            statistical_analysis: None,
            diagnostics_report: None,
        }
    }

    /// Get the best performing configuration based on average response time
    pub fn best_config(&self) -> Option<&str> {
        self.test_results
            .iter()
            .filter(|(_, result)| result.success_count > 0)
            .min_by(|a, b| {
                let a_time = a.1.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
                let b_time = b.1.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
                a_time.partial_cmp(&b_time).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, result)| result.config_name.as_str())
    }
    
    /// Get the worst performing configuration based on average response time
    pub fn worst_config(&self) -> Option<&str> {
        self.test_results
            .iter()
            .filter(|(_, result)| result.success_count > 0)
            .max_by(|a, b| {
                let a_time = a.1.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(0.0);
                let b_time = b.1.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(0.0);
                a_time.partial_cmp(&b_time).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, _)| name.as_str())
    }
    
    /// Check if execution had any failures
    pub fn has_failures(&self) -> bool {
        self.execution_summary.failed_tests > 0 || 
        self.execution_summary.timeout_tests > 0 ||
        self.execution_summary.success_rate < 95.0
    }
}

impl From<&Config> for ExecutionConfig {
    fn from(config: &Config) -> Self {
        Self {
            test_count: config.test_count,
            timeout: Duration::from_secs(config.timeout_seconds),
            verbose: config.verbose,
            debug: config.debug,
        }
    }
}

/// High-level test executor interface
#[async_trait]
pub trait TestExecutor {
    /// Execute tests for the given URLs and DNS configurations
    async fn execute_tests(
        &self,
        urls: &[String],
        dns_configs: &[DnsConfig],
    ) -> Result<Vec<TestResult>>;
    
    /// Get executor performance statistics
    fn get_statistics(&self) -> ExecutorStatistics;
    
    /// Reset the executor to initial state
    async fn reset(&self) -> Result<()>;
}

/// Basic executor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorStatistics {
    /// Total number of tests executed
    pub total_tests_executed: u64,
    /// Total number of successful tests
    pub successful_tests: u64,
    /// Total number of failed tests
    pub failed_tests: u64,
    /// Average execution time per test
    pub avg_execution_time_ms: f64,
    /// Total execution duration
    pub total_execution_duration: Duration,
    /// Current memory usage (if available)
    pub memory_usage_bytes: Option<usize>,
}

impl Default for ExecutorStatistics {
    fn default() -> Self {
        Self {
            total_tests_executed: 0,
            successful_tests: 0,
            failed_tests: 0,
            avg_execution_time_ms: 0.0,
            total_execution_duration: Duration::ZERO,
            memory_usage_bytes: None,
        }
    }
}

/// Factory for creating different types of test executors
pub struct TestExecutorFactory;

impl TestExecutorFactory {
    /// Create a basic test executor
    pub async fn create_basic_executor(config: &Config) -> Result<BasicTestExecutor> {
        BasicTestExecutor::new(config).await
    }
    
    /// Create an optimized test executor with connection pooling
    pub async fn create_optimized_executor(config: &Config) -> Result<OptimizedExecutor> {
        OptimizedExecutor::new(config).await
    }
    
    /// Create a tuned executor with adaptive concurrency management
    pub async fn create_tuned_executor(
        config: &Config,
        tuning_config: Option<TuningConfig>,
    ) -> Result<TunedTestExecutor> {
        let tuning_config = tuning_config.unwrap_or_default();
        TunedTestExecutor::new(config, tuning_config).await
    }
}

/// Basic test executor without optimizations
pub struct BasicTestExecutor {
    config: ExecutionConfig,
    statistics: ExecutorStatistics,
}

impl BasicTestExecutor {
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            config: ExecutionConfig::from(config),
            statistics: ExecutorStatistics::default(),
        })
    }
}

#[async_trait]
impl TestExecutor for BasicTestExecutor {
    async fn execute_tests(
        &self,
        _urls: &[String],
        _dns_configs: &[DnsConfig],
    ) -> Result<Vec<TestResult>> {
        // Basic implementation - this would be implemented with simple HTTP requests
        // For now, return empty results as this is primarily for the interface
        Ok(Vec::new())
    }
    
    fn get_statistics(&self) -> ExecutorStatistics {
        self.statistics.clone()
    }
    
    async fn reset(&self) -> Result<()> {
        Ok(())
    }
}

/// Advanced test executor with both optimization and adaptive tuning
pub struct TunedTestExecutor {
    optimized_executor: OptimizedExecutor,
    tuner: ConcurrencyTuner,
    config: ExecutionConfig,
}

impl TunedTestExecutor {
    pub async fn new(config: &Config, tuning_config: TuningConfig) -> Result<Self> {
        let optimized_executor = OptimizedExecutor::new(config).await?;
        let tuner = ConcurrencyTuner::new(config, tuning_config).await?;
        
        Ok(Self {
            optimized_executor,
            tuner,
            config: ExecutionConfig::from(config),
        })
    }
    
    /// Execute tests with adaptive tuning
    pub async fn execute_tuned_tests(
        &self,
        urls: &[String],
        dns_configs: &[DnsConfig],
    ) -> Result<Vec<TestResult>> {
        let mut all_results = Vec::new();
        
        for url in urls {
            for dns_config in dns_configs {
                // Get current execution parameters from tuner
                let execution_params = self.tuner.get_current_parameters().await;
                
                // Execute tests using optimized executor with tuned parameters
                let results = self.execute_tuned_batch(
                    url,
                    dns_config,
                    &execution_params,
                ).await?;
                
                // Record performance metrics for tuning
                for result in &results {
                    for timing in &result.individual_results {
                        self.tuner.record_performance(
                            timing,
                            execution_params.max_concurrency,
                        ).await?;
                    }
                }
                
                all_results.extend(results);
            }
        }
        
        Ok(all_results)
    }
    
    /// Execute a tuned batch of tests for a single URL/DNS combination
    async fn execute_tuned_batch(
        &self,
        url: &str,
        dns_config: &DnsConfig,
        params: &ExecutionParameters,
    ) -> Result<Vec<TestResult>> {
        // Execute tests using the optimized executor
        // This is a simplified implementation - in practice, you'd integrate
        // the tuning parameters into the optimized executor's execution
        self.optimized_executor
            .execute_optimized_tests(&[url.to_string()], &[dns_config.clone()])
            .await
    }
    
    /// Get comprehensive tuning and execution statistics
    pub async fn get_comprehensive_statistics(&self) -> TuningStatistics {
        self.tuner.get_tuning_statistics().await
    }
    
    /// Reset both the executor and tuner
    pub async fn reset_all(&self) -> Result<()> {
        self.tuner.reset().await?;
        Ok(())
    }
}

#[async_trait]
impl TestExecutor for TunedTestExecutor {
    async fn execute_tests(
        &self,
        urls: &[String],
        dns_configs: &[DnsConfig],
    ) -> Result<Vec<TestResult>> {
        self.execute_tuned_tests(urls, dns_configs).await
    }
    
    fn get_statistics(&self) -> ExecutorStatistics {
        // Get basic statistics from the optimized executor
        let executor_stats = self.optimized_executor.performance_stats();
        
        ExecutorStatistics {
            total_tests_executed: 0, // Would be tracked in implementation
            successful_tests: 0,     // Would be tracked in implementation
            failed_tests: 0,         // Would be tracked in implementation
            avg_execution_time_ms: 0.0, // Would be calculated from results
            total_execution_duration: Duration::ZERO, // Would be tracked
            memory_usage_bytes: Some(
                executor_stats.pool_stats.total_clients * std::mem::size_of::<reqwest::Client>()
            ),
        }
    }
    
    async fn reset(&self) -> Result<()> {
        self.reset_all().await
    }
}

/// Execution mode selection for different performance requirements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Basic execution without optimizations
    Basic,
    /// Optimized execution with connection pooling
    Optimized,
    /// Adaptive execution with real-time tuning
    Adaptive,
    /// High-performance mode with all optimizations
    HighPerformance,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Optimized
    }
}

/// Create the appropriate executor based on execution mode
pub async fn create_executor_for_mode(
    config: &Config,
    mode: ExecutionMode,
) -> Result<Box<dyn TestExecutor + Send + Sync>> {
    match mode {
        ExecutionMode::Basic => {
            let executor = TestExecutorFactory::create_basic_executor(config).await?;
            Ok(Box::new(executor))
        }
        ExecutionMode::Optimized => {
            let executor = TestExecutorFactory::create_optimized_executor(config).await?;
            Ok(Box::new(executor))
        }
        ExecutionMode::Adaptive | ExecutionMode::HighPerformance => {
            let tuning_config = if matches!(mode, ExecutionMode::HighPerformance) {
                TuningConfig {
                    enable_adaptive_learning: true,
                    tuning_sensitivity: 0.5,
                    conservative_mode: false,
                    ..Default::default()
                }
            } else {
                TuningConfig::default()
            };
            
            let executor = TestExecutorFactory::create_tuned_executor(config, Some(tuning_config)).await?;
            Ok(Box::new(executor))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_executor_creation() {
        let config = Config::default();
        let executor = TestExecutorFactory::create_basic_executor(&config).await;
        assert!(executor.is_ok());
    }
    
    #[tokio::test]
    async fn test_optimized_executor_creation() {
        let config = Config::default();
        let executor = TestExecutorFactory::create_optimized_executor(&config).await;
        assert!(executor.is_ok());
    }
    
    #[tokio::test]
    async fn test_tuned_executor_creation() {
        let config = Config::default();
        let executor = TestExecutorFactory::create_tuned_executor(&config, None).await;
        assert!(executor.is_ok());
    }
    
    #[tokio::test]
    async fn test_execution_mode_factory() {
        let config = Config::default();
        
        for mode in [
            ExecutionMode::Basic,
            ExecutionMode::Optimized,
            ExecutionMode::Adaptive,
            ExecutionMode::HighPerformance,
        ] {
            let executor = create_executor_for_mode(&config, mode).await;
            assert!(executor.is_ok(), "Failed to create executor for mode: {:?}", mode);
        }
    }
    
    #[test]
    fn test_execution_config_from_config() {
        let config = Config {
            test_count: 10,
            timeout_seconds: 15,
            verbose: true,
            debug: true,
            ..Default::default()
        };
        
        let exec_config = ExecutionConfig::from(&config);
        assert_eq!(exec_config.test_count, 10);
        assert_eq!(exec_config.timeout, Duration::from_secs(15));
        assert!(exec_config.verbose);
        assert!(exec_config.debug);
    }
}