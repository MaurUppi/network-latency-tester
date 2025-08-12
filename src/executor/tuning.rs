//! Concurrent execution tuning based on system resources
//!
//! This module provides intelligent tuning of concurrent execution parameters
//! based on system capabilities, network conditions, and performance characteristics.

use crate::{
    error::Result,
    executor::optimized::SystemResources,
    models::{Config, TimingMetrics},
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
    collections::VecDeque,
};
use tokio::sync::RwLock;

/// Intelligent concurrency tuner that adapts execution parameters in real-time
pub struct ConcurrencyTuner {
    /// Current system resources snapshot
    system_resources: SystemResources,
    /// Performance monitoring data
    performance_monitor: Arc<RwLock<PerformanceMonitor>>,
    /// Tuning configuration
    tuning_config: TuningConfig,
    /// Current execution parameters
    current_params: Arc<RwLock<ExecutionParameters>>,
    /// Adaptive learning state
    learning_state: Arc<RwLock<AdaptiveLearningState>>,
}

/// Performance monitoring for adaptive tuning
struct PerformanceMonitor {
    /// Recent performance samples (sliding window)
    recent_samples: VecDeque<PerformanceSample>,
    /// Maximum samples to keep in memory
    max_samples: usize,
    /// Current performance metrics
    current_metrics: PerformanceMetrics,
    /// Performance history for trend analysis
    performance_history: VecDeque<PerformanceSnapshot>,
    /// Last update timestamp
    last_update: Instant,
}

/// Single performance measurement sample
#[derive(Debug, Clone)]
struct PerformanceSample {
    /// Timestamp of the measurement
    timestamp: Instant,
    /// Response time in milliseconds
    response_time_ms: f64,
    /// Success indicator
    success: bool,
    /// Concurrency level when this sample was taken
    concurrency_level: usize,
    /// System load indicator (0.0 to 1.0)
    system_load: f64,
}

/// Current performance metrics
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    /// Average response time over recent samples
    avg_response_time: f64,
    /// Success rate percentage
    success_rate: f64,
    /// Throughput (successful requests per second)
    throughput: f64,
    /// Performance efficiency (throughput / concurrency)
    efficiency: f64,
    /// System resource utilization
    resource_utilization: f64,
    /// Performance stability score
    stability_score: f64,
}

/// Historical performance snapshot
#[derive(Debug, Clone)]
struct PerformanceSnapshot {
    /// When this snapshot was taken
    timestamp: Instant,
    /// Performance metrics at this time
    metrics: PerformanceMetrics,
    /// Execution parameters that were active
    execution_params: ExecutionParameters,
}

/// Execution parameters that can be tuned
#[derive(Debug, Clone)]
pub struct ExecutionParameters {
    /// Maximum concurrent requests
    pub max_concurrency: usize,
    /// Request timeout duration
    pub request_timeout: Duration,
    /// Batch size for request grouping
    pub batch_size: usize,
    /// Delay between batches
    pub batch_delay: Duration,
    /// Connection pool size
    pub connection_pool_size: usize,
    /// Retry attempts for failed requests
    pub retry_attempts: usize,
    /// Backoff multiplier for retries
    pub backoff_multiplier: f64,
    /// Circuit breaker threshold
    pub circuit_breaker_threshold: f64,
}

/// Configuration for the tuning algorithm
#[derive(Debug, Clone)]
pub struct TuningConfig {
    /// Enable adaptive learning
    pub enable_adaptive_learning: bool,
    /// Performance sampling interval
    pub sampling_interval: Duration,
    /// Minimum samples required before tuning
    pub min_samples_for_tuning: usize,
    /// Tuning sensitivity (0.0 to 1.0)
    pub tuning_sensitivity: f64,
    /// Maximum concurrency adjustment per iteration
    pub max_concurrency_step: usize,
    /// Performance threshold for scaling up
    pub scale_up_threshold: f64,
    /// Performance threshold for scaling down
    pub scale_down_threshold: f64,
    /// Stability requirement for parameter changes
    pub stability_requirement: f64,
    /// Conservative mode (slower but safer adjustments)
    pub conservative_mode: bool,
}

/// Adaptive learning state for continuous improvement
struct AdaptiveLearningState {
    /// Best performing parameters discovered so far
    best_parameters: Option<(ExecutionParameters, PerformanceMetrics)>,
    /// Parameters currently being explored
    exploration_parameters: Vec<ExecutionParameters>,
    /// Learning rate for parameter adjustments
    learning_rate: f64,
    /// Exploration vs exploitation balance
    exploration_factor: f64,
    /// Number of tuning iterations performed
    iteration_count: usize,
    /// Performance improvement trend
    improvement_trend: ImprovementTrend,
}

/// Trend in performance improvements
#[derive(Debug, Clone, Copy)]
enum ImprovementTrend {
    /// Performance is improving
    Improving,
    /// Performance is degrading
    Degrading,
    /// Performance is stable
    Stable,
    /// Not enough data to determine trend
    Unknown,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            enable_adaptive_learning: true,
            sampling_interval: Duration::from_secs(5),
            min_samples_for_tuning: 10,
            tuning_sensitivity: 0.3,
            max_concurrency_step: 5,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.5,
            stability_requirement: 0.9,
            conservative_mode: false,
        }
    }
}

impl Default for ExecutionParameters {
    fn default() -> Self {
        Self {
            max_concurrency: 10,
            request_timeout: Duration::from_secs(10),
            batch_size: 5,
            batch_delay: Duration::from_millis(100),
            connection_pool_size: 20,
            retry_attempts: 3,
            backoff_multiplier: 1.5,
            circuit_breaker_threshold: 0.5,
        }
    }
}

impl ConcurrencyTuner {
    /// Create a new concurrency tuner
    pub async fn new(config: &Config, tuning_config: TuningConfig) -> Result<Self> {
        let system_resources = SystemResources::detect();
        
        // Calculate initial execution parameters based on system resources
        let initial_params = Self::calculate_initial_parameters(&system_resources, config)?;
        
        let performance_monitor = Arc::new(RwLock::new(PerformanceMonitor {
            recent_samples: VecDeque::with_capacity(1000),
            max_samples: 1000,
            current_metrics: PerformanceMetrics::default(),
            performance_history: VecDeque::with_capacity(100),
            last_update: Instant::now(),
        }));
        
        let learning_state = Arc::new(RwLock::new(AdaptiveLearningState {
            best_parameters: None,
            exploration_parameters: Vec::new(),
            learning_rate: 0.1,
            exploration_factor: 0.2,
            iteration_count: 0,
            improvement_trend: ImprovementTrend::Unknown,
        }));
        
        Ok(Self {
            system_resources,
            performance_monitor,
            tuning_config,
            current_params: Arc::new(RwLock::new(initial_params)),
            learning_state,
        })
    }
    
    /// Calculate initial execution parameters based on system resources
    fn calculate_initial_parameters(resources: &SystemResources, config: &Config) -> Result<ExecutionParameters> {
        let base_concurrency = resources.optimal_concurrency;
        let base_timeout = Duration::from_secs(config.timeout_seconds);
        
        // Adjust based on URL count - more URLs need less concurrency per URL
        let url_count = config.target_urls.len().max(1);
        let adjusted_concurrency = if url_count > 5 {
            base_concurrency / 2
        } else {
            base_concurrency
        };
        
        // Adjust connection pool size based on concurrency and URL count
        let pool_size = (adjusted_concurrency * url_count * 2).min(resources.max_concurrent_connections);
        
        // Calculate batch size based on test count and concurrency
        let batch_size = if config.test_count <= 10 {
            config.test_count as usize
        } else {
            (config.test_count as usize / adjusted_concurrency).max(1).min(10)
        };
        
        Ok(ExecutionParameters {
            max_concurrency: adjusted_concurrency,
            request_timeout: base_timeout,
            batch_size,
            batch_delay: Duration::from_millis(50),
            connection_pool_size: pool_size,
            retry_attempts: if config.test_count > 20 { 2 } else { 3 },
            backoff_multiplier: 1.5,
            circuit_breaker_threshold: 0.6,
        })
    }
    
    /// Record a performance measurement for tuning analysis
    pub async fn record_performance(&self, timing: &TimingMetrics, concurrency_level: usize) -> Result<()> {
        let sample = PerformanceSample {
            timestamp: Instant::now(),
            response_time_ms: timing.total_ms(),
            success: timing.is_successful(),
            concurrency_level,
            system_load: self.estimate_system_load().await,
        };
        
        let mut monitor = self.performance_monitor.write().await;
        monitor.add_sample(sample);
        monitor.update_metrics();
        
        // Check if we should trigger a tuning adjustment
        if monitor.should_trigger_tuning(&self.tuning_config) {
            drop(monitor);
            self.perform_tuning_iteration().await?;
        }
        
        Ok(())
    }
    
    /// Get current execution parameters
    pub async fn get_current_parameters(&self) -> ExecutionParameters {
        self.current_params.read().await.clone()
    }
    
    /// Perform a tuning iteration to optimize parameters
    async fn perform_tuning_iteration(&self) -> Result<()> {
        let monitor = self.performance_monitor.read().await;
        let current_metrics = monitor.current_metrics.clone();
        let samples_count = monitor.recent_samples.len();
        drop(monitor);
        
        if samples_count < self.tuning_config.min_samples_for_tuning {
            return Ok(()); // Not enough data yet
        }
        
        let current_params = self.current_params.read().await.clone();
        
        // Determine if we should scale up or down based on performance
        let scaling_decision = self.determine_scaling_decision(&current_metrics).await?;
        
        match scaling_decision {
            ScalingDecision::ScaleUp => {
                let new_params = self.scale_up_parameters(&current_params).await?;
                self.apply_parameters(new_params).await?;
            }
            ScalingDecision::ScaleDown => {
                let new_params = self.scale_down_parameters(&current_params).await?;
                self.apply_parameters(new_params).await?;
            }
            ScalingDecision::Maintain => {
                // Parameters are optimal, no changes needed
            }
            ScalingDecision::Explore => {
                if self.tuning_config.enable_adaptive_learning {
                    let new_params = self.explore_parameters(&current_params).await?;
                    self.apply_parameters(new_params).await?;
                }
            }
        }
        
        // Update learning state
        let mut learning = self.learning_state.write().await;
        learning.iteration_count += 1;
        learning.update_trend(&current_metrics);
        
        // Check if current parameters are the best so far
        if learning.is_best_performance(&current_metrics) {
            learning.best_parameters = Some((current_params, current_metrics));
        }
        
        Ok(())
    }
    
    /// Determine the appropriate scaling decision
    async fn determine_scaling_decision(&self, metrics: &PerformanceMetrics) -> Result<ScalingDecision> {
        let efficiency = metrics.efficiency;
        let utilization = metrics.resource_utilization;
        let stability = metrics.stability_score;
        
        // Require stability before making changes
        if stability < self.tuning_config.stability_requirement {
            return Ok(ScalingDecision::Maintain);
        }
        
        // Check for exploration opportunity
        let learning = self.learning_state.read().await;
        if learning.should_explore() {
            return Ok(ScalingDecision::Explore);
        }
        
        // Performance-based scaling decisions
        if efficiency > self.tuning_config.scale_up_threshold && utilization < 0.8 {
            Ok(ScalingDecision::ScaleUp)
        } else if efficiency < self.tuning_config.scale_down_threshold || utilization > 0.9 {
            Ok(ScalingDecision::ScaleDown)
        } else {
            Ok(ScalingDecision::Maintain)
        }
    }
    
    /// Scale up execution parameters for better performance
    async fn scale_up_parameters(&self, current: &ExecutionParameters) -> Result<ExecutionParameters> {
        let max_system_concurrency = self.system_resources.max_concurrent_connections;
        let step_size = if self.tuning_config.conservative_mode { 1 } else { self.tuning_config.max_concurrency_step };
        
        let new_concurrency = (current.max_concurrency + step_size).min(max_system_concurrency);
        let new_pool_size = (current.connection_pool_size + step_size * 2).min(max_system_concurrency * 2);
        
        Ok(ExecutionParameters {
            max_concurrency: new_concurrency,
            connection_pool_size: new_pool_size,
            batch_size: (current.batch_size + 1).min(20),
            // Reduce timeout slightly for faster turnaround at higher concurrency
            request_timeout: current.request_timeout.mul_f64(0.95),
            ..current.clone()
        })
    }
    
    /// Scale down execution parameters to reduce resource usage
    async fn scale_down_parameters(&self, current: &ExecutionParameters) -> Result<ExecutionParameters> {
        let step_size = if self.tuning_config.conservative_mode { 1 } else { self.tuning_config.max_concurrency_step };
        
        let new_concurrency = (current.max_concurrency.saturating_sub(step_size)).max(1);
        let new_pool_size = (current.connection_pool_size.saturating_sub(step_size)).max(new_concurrency);
        
        Ok(ExecutionParameters {
            max_concurrency: new_concurrency,
            connection_pool_size: new_pool_size,
            batch_size: (current.batch_size.saturating_sub(1)).max(1),
            // Increase timeout to be more patient at lower concurrency
            request_timeout: current.request_timeout.mul_f64(1.1),
            ..current.clone()
        })
    }
    
    /// Explore new parameter configurations for learning
    async fn explore_parameters(&self, current: &ExecutionParameters) -> Result<ExecutionParameters> {
        let mut learning = self.learning_state.write().await;
        
        // Generate exploration parameters based on current best knowledge
        let exploration_variance = learning.exploration_factor;
        let concurrency_variance = ((current.max_concurrency as f64) * exploration_variance) as usize;
        
        let new_concurrency = if learning.iteration_count % 2 == 0 {
            // Explore higher concurrency
            current.max_concurrency + concurrency_variance
        } else {
            // Explore lower concurrency
            current.max_concurrency.saturating_sub(concurrency_variance).max(1)
        };
        
        let new_params = ExecutionParameters {
            max_concurrency: new_concurrency.min(self.system_resources.max_concurrent_connections),
            connection_pool_size: (new_concurrency * 2).min(self.system_resources.max_concurrent_connections * 2),
            batch_size: if new_concurrency > current.max_concurrency {
                (current.batch_size + 1).min(15)
            } else {
                (current.batch_size.saturating_sub(1)).max(1)
            },
            request_timeout: if new_concurrency > current.max_concurrency {
                current.request_timeout.mul_f64(0.9)
            } else {
                current.request_timeout.mul_f64(1.1)
            },
            ..current.clone()
        };
        
        learning.exploration_parameters.push(new_params.clone());
        Ok(new_params)
    }
    
    /// Apply new execution parameters
    async fn apply_parameters(&self, new_params: ExecutionParameters) -> Result<()> {
        let mut current = self.current_params.write().await;
        *current = new_params;
        
        // Take a performance snapshot for history
        let monitor = self.performance_monitor.read().await;
        let snapshot = PerformanceSnapshot {
            timestamp: Instant::now(),
            metrics: monitor.current_metrics.clone(),
            execution_params: current.clone(),
        };
        drop(monitor);
        
        let mut monitor_write = self.performance_monitor.write().await;
        monitor_write.add_snapshot(snapshot);
        
        Ok(())
    }
    
    /// Estimate current system load
    async fn estimate_system_load(&self) -> f64 {
        // Simplified system load estimation
        // In a real implementation, this would query system metrics
        let monitor = self.performance_monitor.read().await;
        
        if monitor.recent_samples.is_empty() {
            return 0.5; // Default moderate load
        }
        
        // Estimate load based on recent response times and success rates
        let recent_avg_time = monitor.current_metrics.avg_response_time;
        let success_rate = monitor.current_metrics.success_rate;
        
        // Higher response times and lower success rates indicate higher system load
        let time_load_factor = (recent_avg_time / 1000.0).min(1.0); // Normalize to 0-1
        let success_load_factor = 1.0 - (success_rate / 100.0);
        
        (time_load_factor + success_load_factor) / 2.0
    }
    
    /// Get comprehensive tuning statistics
    pub async fn get_tuning_statistics(&self) -> TuningStatistics {
        let monitor = self.performance_monitor.read().await;
        let learning = self.learning_state.read().await;
        let current_params = self.current_params.read().await;
        
        TuningStatistics {
            system_resources: self.system_resources.clone(),
            current_parameters: current_params.clone(),
            current_metrics: monitor.current_metrics.clone(),
            iteration_count: learning.iteration_count,
            improvement_trend: learning.improvement_trend,
            best_parameters: learning.best_parameters.clone(),
            samples_collected: monitor.recent_samples.len(),
            tuning_effectiveness: self.calculate_tuning_effectiveness(&*monitor, &*learning),
        }
    }
    
    /// Calculate the effectiveness of the tuning process
    fn calculate_tuning_effectiveness(&self, monitor: &PerformanceMonitor, learning: &AdaptiveLearningState) -> f64 {
        if learning.iteration_count == 0 || monitor.performance_history.len() < 2 {
            return 0.0;
        }
        
        // Compare current performance to initial performance
        let initial_performance = &monitor.performance_history[0].metrics;
        let current_performance = &monitor.current_metrics;
        
        let efficiency_improvement = current_performance.efficiency - initial_performance.efficiency;
        let throughput_improvement = current_performance.throughput - initial_performance.throughput;
        
        // Normalize improvements to a 0-1 scale
        let efficiency_score = (efficiency_improvement + 1.0) / 2.0;
        let throughput_score = (throughput_improvement / initial_performance.throughput.max(1.0)).max(0.0).min(1.0);
        
        (efficiency_score + throughput_score) / 2.0
    }
    
    /// Reset the tuner to initial state
    pub async fn reset(&self) -> Result<()> {
        let mut monitor = self.performance_monitor.write().await;
        monitor.recent_samples.clear();
        monitor.performance_history.clear();
        monitor.current_metrics = PerformanceMetrics::default();
        
        let mut learning = self.learning_state.write().await;
        learning.best_parameters = None;
        learning.exploration_parameters.clear();
        learning.iteration_count = 0;
        learning.improvement_trend = ImprovementTrend::Unknown;
        
        Ok(())
    }
}

/// Decision for parameter scaling
#[derive(Debug, Clone, Copy)]
enum ScalingDecision {
    /// Increase concurrency and resources
    ScaleUp,
    /// Decrease concurrency and resources
    ScaleDown,
    /// Maintain current parameters
    Maintain,
    /// Explore different parameter configurations
    Explore,
}

/// Comprehensive tuning statistics
#[derive(Debug, Clone)]
pub struct TuningStatistics {
    pub system_resources: SystemResources,
    pub current_parameters: ExecutionParameters,
    pub current_metrics: PerformanceMetrics,
    pub iteration_count: usize,
    pub improvement_trend: ImprovementTrend,
    pub best_parameters: Option<(ExecutionParameters, PerformanceMetrics)>,
    pub samples_collected: usize,
    pub tuning_effectiveness: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_response_time: 0.0,
            success_rate: 100.0,
            throughput: 0.0,
            efficiency: 0.0,
            resource_utilization: 0.0,
            stability_score: 1.0,
        }
    }
}

impl PerformanceMonitor {
    /// Add a performance sample
    fn add_sample(&mut self, sample: PerformanceSample) {
        self.recent_samples.push_back(sample);
        
        // Maintain sliding window size
        if self.recent_samples.len() > self.max_samples {
            self.recent_samples.pop_front();
        }
        
        self.last_update = Instant::now();
    }
    
    /// Add a performance snapshot to history
    fn add_snapshot(&mut self, snapshot: PerformanceSnapshot) {
        self.performance_history.push_back(snapshot);
        
        // Keep history bounded
        if self.performance_history.len() > 100 {
            self.performance_history.pop_front();
        }
    }
    
    /// Update current performance metrics based on recent samples
    fn update_metrics(&mut self) {
        if self.recent_samples.is_empty() {
            return;
        }
        
        let window_duration = Duration::from_secs(30);
        let cutoff_time = Instant::now().checked_sub(window_duration).unwrap_or_else(Instant::now);
        
        // Filter to recent samples within the window
        let recent: Vec<&PerformanceSample> = self.recent_samples
            .iter()
            .filter(|s| s.timestamp > cutoff_time)
            .collect();
        
        if recent.is_empty() {
            return;
        }
        
        // Calculate metrics
        let total_samples = recent.len() as f64;
        let successful_samples: Vec<&PerformanceSample> = recent.iter()
            .filter(|s| s.success)
            .cloned()
            .collect();
        
        let avg_response_time = recent.iter()
            .map(|s| s.response_time_ms)
            .sum::<f64>() / total_samples;
        
        let success_rate = (successful_samples.len() as f64 / total_samples) * 100.0;
        
        let throughput = successful_samples.len() as f64 / window_duration.as_secs_f64();
        
        let avg_concurrency = recent.iter()
            .map(|s| s.concurrency_level as f64)
            .sum::<f64>() / total_samples;
        
        let efficiency = if avg_concurrency > 0.0 {
            throughput / avg_concurrency
        } else {
            0.0
        };
        
        let resource_utilization = recent.iter()
            .map(|s| s.system_load)
            .sum::<f64>() / total_samples;
        
        // Calculate stability score based on response time variance
        let response_times: Vec<f64> = recent.iter().map(|s| s.response_time_ms).collect();
        let variance = if response_times.len() > 1 {
            let mean = avg_response_time;
            let sum_squared_diff: f64 = response_times.iter()
                .map(|&x| (x - mean).powi(2))
                .sum();
            sum_squared_diff / response_times.len() as f64
        } else {
            0.0
        };
        
        let coefficient_of_variation = if avg_response_time > 0.0 {
            variance.sqrt() / avg_response_time
        } else {
            0.0
        };
        
        let stability_score = (1.0 - coefficient_of_variation.min(1.0)).max(0.0);
        
        self.current_metrics = PerformanceMetrics {
            avg_response_time,
            success_rate,
            throughput,
            efficiency,
            resource_utilization,
            stability_score,
        };
    }
    
    /// Check if tuning should be triggered
    fn should_trigger_tuning(&self, config: &TuningConfig) -> bool {
        let samples_sufficient = self.recent_samples.len() >= config.min_samples_for_tuning;
        let time_elapsed = self.last_update.elapsed() >= config.sampling_interval;
        
        samples_sufficient && time_elapsed
    }
}

impl AdaptiveLearningState {
    /// Check if exploration should be performed
    fn should_explore(&self) -> bool {
        if self.iteration_count == 0 {
            return false;
        }
        
        // Explore periodically based on exploration factor
        let exploration_frequency = (1.0 / self.exploration_factor) as usize;
        self.iteration_count % exploration_frequency == 0
    }
    
    /// Update performance improvement trend
    fn update_trend(&mut self, current_metrics: &PerformanceMetrics) {
        if let Some((_, best_metrics)) = &self.best_parameters {
            if current_metrics.efficiency > best_metrics.efficiency * 1.05 {
                self.improvement_trend = ImprovementTrend::Improving;
            } else if current_metrics.efficiency < best_metrics.efficiency * 0.95 {
                self.improvement_trend = ImprovementTrend::Degrading;
            } else {
                self.improvement_trend = ImprovementTrend::Stable;
            }
        } else {
            self.improvement_trend = ImprovementTrend::Unknown;
        }
    }
    
    /// Check if current metrics represent the best performance so far
    fn is_best_performance(&self, current_metrics: &PerformanceMetrics) -> bool {
        match &self.best_parameters {
            Some((_, best_metrics)) => {
                current_metrics.efficiency > best_metrics.efficiency &&
                current_metrics.success_rate >= best_metrics.success_rate * 0.95
            }
            None => true, // First measurement is always the best so far
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_concurrency_tuner_creation() {
        let config = Config::default();
        let tuning_config = TuningConfig::default();
        
        let tuner = ConcurrencyTuner::new(&config, tuning_config).await;
        assert!(tuner.is_ok());
        
        let tuner = tuner.unwrap();
        let params = tuner.get_current_parameters().await;
        assert!(params.max_concurrency > 0);
        assert!(params.connection_pool_size >= params.max_concurrency);
    }
    
    #[tokio::test]
    async fn test_performance_recording() {
        let config = Config::default();
        let tuning_config = TuningConfig::default();
        let tuner = ConcurrencyTuner::new(&config, tuning_config).await.unwrap();
        
        let timing = TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            None,
            Duration::from_millis(50),
            Duration::from_millis(100),
            200,
        );
        
        let result = tuner.record_performance(&timing, 5).await;
        assert!(result.is_ok());
        
        let stats = tuner.get_tuning_statistics().await;
        assert_eq!(stats.samples_collected, 1);
    }
    
    #[tokio::test]
    async fn test_parameter_scaling() {
        let config = Config::default();
        let tuning_config = TuningConfig::default();
        let tuner = ConcurrencyTuner::new(&config, tuning_config).await.unwrap();
        
        let initial_params = tuner.get_current_parameters().await;
        
        // Simulate good performance to trigger scale-up
        for _ in 0..15 {
            let timing = TimingMetrics::success(
                Duration::from_millis(5),
                Duration::from_millis(10),
                None,
                Duration::from_millis(25),
                Duration::from_millis(50),
                200,
            );
            tuner.record_performance(&timing, initial_params.max_concurrency).await.unwrap();
        }
        
        let updated_params = tuner.get_current_parameters().await;
        // Parameters should have been adjusted
        assert!(updated_params.max_concurrency != initial_params.max_concurrency);
    }
    
    #[tokio::test]
    async fn test_system_load_estimation() {
        let config = Config::default();
        let tuning_config = TuningConfig::default();
        let tuner = ConcurrencyTuner::new(&config, tuning_config).await.unwrap();
        
        let load = tuner.estimate_system_load().await;
        assert!((0.0..=1.0).contains(&load));
    }
    
    #[tokio::test]
    async fn test_tuning_statistics() {
        let config = Config::default();
        let tuning_config = TuningConfig::default();
        let tuner = ConcurrencyTuner::new(&config, tuning_config).await.unwrap();
        
        let stats = tuner.get_tuning_statistics().await;
        assert_eq!(stats.iteration_count, 0);
        assert_eq!(stats.samples_collected, 0);
        assert!(stats.best_parameters.is_none());
    }
}