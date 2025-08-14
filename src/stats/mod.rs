//! Statistical analysis and calculation engine for network latency tests

pub mod optimized;

pub use optimized::{
    OptimizedStatisticsCalculator, RollingStats, StatisticsMemoryPool,
    BufferStats, PoolStats,
};

use crate::{
    error::{AppError, Result},
    types::PerformanceLevel,
    models::metrics::{TimingMetrics, TestResult, Statistics},
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Comprehensive statistics engine for network latency analysis
pub struct StatisticsEngine {
    /// Collected test results grouped by DNS configuration
    results: HashMap<String, Vec<TestResult>>,
    /// Configuration for statistical calculations
    config: StatisticsConfig,
}

/// Configuration for statistical calculations
#[derive(Debug, Clone)]
pub struct StatisticsConfig {
    /// Minimum number of samples required for reliable statistics
    pub min_samples: usize,
    /// Confidence level for statistical intervals (e.g., 0.95 for 95%)
    pub confidence_level: f64,
    /// Percentiles to calculate (e.g., 50th, 90th, 95th, 99th)
    pub percentiles: Vec<f64>,
    /// Whether to exclude outliers from calculations
    pub exclude_outliers: bool,
    /// Outlier detection method
    pub outlier_method: OutlierMethod,
}

/// Methods for detecting outliers in timing data
#[derive(Debug, Clone, Copy)]
pub enum OutlierMethod {
    /// Interquartile range method (1.5 * IQR)
    IQR,
    /// Standard deviation method (2 or 3 standard deviations)
    StandardDeviation { threshold: f64 },
    /// Modified Z-score using median absolute deviation
    ModifiedZScore { threshold: f64 },
}

/// Comprehensive statistical analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysis {
    /// Basic statistics for each DNS configuration
    pub basic_stats: HashMap<String, ExtendedStatistics>,
    /// Comparative analysis between configurations
    pub comparative_analysis: ComparativeAnalysis,
    /// Trend analysis over time
    pub trend_analysis: Option<TrendAnalysis>,
    /// Summary and recommendations
    pub summary: AnalysisSummary,
    /// When this analysis was generated
    pub generated_at: DateTime<Utc>,
}

/// Extended statistics with advanced metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedStatistics {
    /// Basic statistics (reused from existing)
    pub basic: Statistics,
    /// Percentile values
    pub percentiles: HashMap<String, f64>,
    /// Confidence intervals
    pub confidence_intervals: ConfidenceIntervals,
    /// Outlier analysis
    pub outlier_analysis: OutlierAnalysis,
    /// Performance classification distribution
    pub performance_distribution: PerformanceDistribution,
    /// Reliability metrics
    pub reliability: ReliabilityMetrics,
}

/// Confidence intervals for key metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceIntervals {
    /// Confidence level (e.g., 0.95)
    pub level: f64,
    /// Average response time interval
    pub avg_response_time: (f64, f64),
    /// Success rate interval
    pub success_rate: (f64, f64),
    /// DNS resolution time interval
    pub dns_resolution_time: (f64, f64),
}

/// Outlier detection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierAnalysis {
    /// Number of outliers detected
    pub outlier_count: usize,
    /// Percentage of samples that are outliers
    pub outlier_percentage: f64,
    /// Method used for detection
    pub detection_method: String,
    /// Threshold values used
    pub threshold_values: HashMap<String, f64>,
}

/// Performance level distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDistribution {
    /// Percentage of tests with good performance
    pub good_percentage: f64,
    /// Percentage of tests with moderate performance
    pub moderate_percentage: f64,
    /// Percentage of tests with poor performance
    pub poor_percentage: f64,
}

/// Reliability and consistency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    /// Success rate percentage
    pub success_rate: f64,
    /// Consistency score (lower is more consistent)
    pub consistency_score: f64,
    /// Jitter (variation in response times)
    pub jitter_ms: f64,
    /// Uptime percentage (if applicable)
    pub uptime_percentage: Option<f64>,
}

/// Comparative analysis between DNS configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparativeAnalysis {
    /// Best performing configuration by average response time
    pub fastest_config: Option<String>,
    /// Most reliable configuration by success rate
    pub most_reliable_config: Option<String>,
    /// Most consistent configuration by standard deviation
    pub most_consistent_config: Option<String>,
    /// Performance rankings
    pub performance_rankings: Vec<ConfigurationRanking>,
    /// Statistical significance tests
    pub significance_tests: Vec<SignificanceTest>,
}

/// Ranking of DNS configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationRanking {
    /// Configuration name
    pub config_name: String,
    /// Overall rank (1 = best)
    pub rank: usize,
    /// Overall score (0.0 - 1.0, higher is better)
    pub score: f64,
    /// Individual metric scores
    pub metric_scores: HashMap<String, f64>,
}

/// Statistical significance test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificanceTest {
    /// Test name (e.g., "t-test", "Mann-Whitney U")
    pub test_name: String,
    /// Configurations being compared
    pub configurations: (String, String),
    /// P-value
    pub p_value: f64,
    /// Is the difference statistically significant?
    pub is_significant: bool,
    /// Effect size
    pub effect_size: f64,
}

/// Trend analysis over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Time period covered
    pub time_period: (DateTime<Utc>, DateTime<Utc>),
    /// Trending direction for each configuration
    pub trends: HashMap<String, TrendDirection>,
    /// Correlation with time
    pub temporal_correlations: HashMap<String, f64>,
    /// Detected patterns
    pub patterns: Vec<String>,
}

/// Direction of performance trend
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Performance is improving over time
    Improving,
    /// Performance is degrading over time
    Degrading,
    /// Performance is stable
    Stable,
    /// No clear trend detected
    NoTrend,
}

/// Analysis summary and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Overall best DNS configuration
    pub recommended_config: Option<String>,
    /// Key findings
    pub key_findings: Vec<String>,
    /// Performance insights
    pub insights: Vec<String>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
    /// Warnings or concerns
    pub warnings: Vec<String>,
}

impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            min_samples: 5,
            confidence_level: 0.95,
            percentiles: vec![50.0, 90.0, 95.0, 99.0],
            exclude_outliers: false,
            outlier_method: OutlierMethod::IQR,
        }
    }
}

impl StatisticsEngine {
    /// Create a new statistics engine
    pub fn new(config: StatisticsConfig) -> Self {
        Self {
            results: HashMap::new(),
            config,
        }
    }

    /// Create a statistics engine with default configuration
    pub fn with_defaults() -> Self {
        Self::new(StatisticsConfig::default())
    }

    /// Add test results for analysis
    pub fn add_results(&mut self, results: Vec<TestResult>) {
        for result in results {
            let config_name = result.config_name.clone();
            self.results.entry(config_name).or_default().push(result);
        }
    }

    /// Add a single test result
    pub fn add_result(&mut self, result: TestResult) {
        let config_name = result.config_name.clone();
        self.results.entry(config_name).or_default().push(result);
    }

    /// Generate comprehensive statistical analysis
    pub fn analyze(&self) -> Result<StatisticalAnalysis> {
        if self.results.is_empty() {
            return Err(AppError::validation("No test results available for analysis"));
        }

        let mut basic_stats = HashMap::new();
        
        // Calculate extended statistics for each configuration
        for (config_name, results) in &self.results {
            if results.is_empty() {
                continue;
            }

            let extended_stats = self.calculate_extended_statistics(results)?;
            basic_stats.insert(config_name.clone(), extended_stats);
        }

        // Perform comparative analysis
        let comparative_analysis = self.perform_comparative_analysis(&basic_stats)?;

        // Perform trend analysis if we have temporal data
        let trend_analysis = self.perform_trend_analysis()?;

        // Generate summary and recommendations
        let summary = self.generate_summary(&basic_stats, &comparative_analysis)?;

        Ok(StatisticalAnalysis {
            basic_stats,
            comparative_analysis,
            trend_analysis,
            summary,
            generated_at: Utc::now(),
        })
    }

    /// Calculate extended statistics for a set of test results
    fn calculate_extended_statistics(&self, results: &[TestResult]) -> Result<ExtendedStatistics> {
        if results.is_empty() {
            return Err(AppError::validation("No results provided for statistics calculation"));
        }

        // Collect all successful timing measurements
        let mut all_timings = Vec::new();
        for result in results {
            for timing in &result.individual_results {
                if timing.is_successful() {
                    all_timings.push(timing);
                }
            }
        }

        if all_timings.is_empty() {
            return Err(AppError::validation("No successful measurements for statistics calculation"));
        }

        // Calculate basic statistics using existing implementation
        let basic = Statistics::from_measurements(&all_timings);

        // Calculate percentiles
        let percentiles = self.calculate_percentiles(&all_timings)?;

        // Calculate confidence intervals
        let confidence_intervals = self.calculate_confidence_intervals(&all_timings)?;

        // Perform outlier analysis
        let outlier_analysis = self.detect_outliers(&all_timings)?;

        // Calculate performance distribution
        let performance_distribution = self.calculate_performance_distribution(&all_timings);

        // Calculate reliability metrics
        let reliability = self.calculate_reliability_metrics(results, &all_timings);

        Ok(ExtendedStatistics {
            basic,
            percentiles,
            confidence_intervals,
            outlier_analysis,
            performance_distribution,
            reliability,
        })
    }

    /// Calculate percentiles for timing measurements
    fn calculate_percentiles(&self, timings: &[&TimingMetrics]) -> Result<HashMap<String, f64>> {
        let mut total_times: Vec<f64> = timings.iter().map(|t| t.total_ms()).collect();
        total_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut percentiles = HashMap::new();
        
        for &p in &self.config.percentiles {
            let value = self.calculate_percentile(&total_times, p);
            percentiles.insert(format!("p{:.0}", p), value);
        }

        Ok(percentiles)
    }

    /// Calculate a specific percentile
    fn calculate_percentile(&self, sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (percentile / 100.0) * (sorted_values.len() as f64 - 1.0);
        let lower_index = index.floor() as usize;
        let upper_index = index.ceil() as usize;

        if lower_index == upper_index {
            sorted_values[lower_index]
        } else {
            let lower_value = sorted_values[lower_index];
            let upper_value = sorted_values[upper_index];
            let weight = index - lower_index as f64;
            lower_value + weight * (upper_value - lower_value)
        }
    }

    /// Calculate confidence intervals for key metrics
    fn calculate_confidence_intervals(&self, timings: &[&TimingMetrics]) -> Result<ConfidenceIntervals> {
        if timings.len() < self.config.min_samples {
            return Ok(ConfidenceIntervals {
                level: self.config.confidence_level,
                avg_response_time: (0.0, 0.0),
                success_rate: (0.0, 0.0),
                dns_resolution_time: (0.0, 0.0),
            });
        }

        let n = timings.len() as f64;
        let z_score = self.get_z_score(self.config.confidence_level);

        // Response time confidence interval
        let response_times: Vec<f64> = timings.iter().map(|t| t.total_ms()).collect();
        let response_mean = response_times.iter().sum::<f64>() / n;
        let response_std = self.calculate_standard_deviation(&response_times, response_mean);
        let response_margin = z_score * response_std / n.sqrt();

        // DNS resolution time confidence interval
        let dns_times: Vec<f64> = timings.iter().map(|t| t.dns_ms()).collect();
        let dns_mean = dns_times.iter().sum::<f64>() / n;
        let dns_std = self.calculate_standard_deviation(&dns_times, dns_mean);
        let dns_margin = z_score * dns_std / n.sqrt();

        // Success rate is 100% for successful measurements (these are pre-filtered)
        let success_margin = z_score * (100.0 * 0.0 / n).sqrt(); // No variation in success rate

        Ok(ConfidenceIntervals {
            level: self.config.confidence_level,
            avg_response_time: (response_mean - response_margin, response_mean + response_margin),
            success_rate: (100.0 - success_margin, 100.0 + success_margin),
            dns_resolution_time: (dns_mean - dns_margin, dns_mean + dns_margin),
        })
    }

    /// Get Z-score for given confidence level
    fn get_z_score(&self, confidence_level: f64) -> f64 {
        // Common Z-scores for confidence levels
        match confidence_level {
            level if (level - 0.90).abs() < 0.01 => 1.645,
            level if (level - 0.95).abs() < 0.01 => 1.96,
            level if (level - 0.99).abs() < 0.01 => 2.576,
            _ => 1.96, // Default to 95%
        }
    }

    /// Calculate standard deviation
    fn calculate_standard_deviation(&self, values: &[f64], mean: f64) -> f64 {
        if values.len() <= 1 {
            return 0.0;
        }

        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;
        
        variance.sqrt()
    }

    /// Detect outliers in timing measurements
    fn detect_outliers(&self, timings: &[&TimingMetrics]) -> Result<OutlierAnalysis> {
        let total_times: Vec<f64> = timings.iter().map(|t| t.total_ms()).collect();
        
        let outlier_count = match self.config.outlier_method {
            OutlierMethod::IQR => self.detect_outliers_iqr(&total_times),
            OutlierMethod::StandardDeviation { threshold } => 
                self.detect_outliers_std_dev(&total_times, threshold),
            OutlierMethod::ModifiedZScore { threshold } => 
                self.detect_outliers_modified_z_score(&total_times, threshold),
        };

        let outlier_percentage = if total_times.is_empty() {
            0.0
        } else {
            (outlier_count as f64 / total_times.len() as f64) * 100.0
        };

        let mut threshold_values = HashMap::new();
        match self.config.outlier_method {
            OutlierMethod::StandardDeviation { threshold } => {
                threshold_values.insert("std_dev_threshold".to_string(), threshold);
            }
            OutlierMethod::ModifiedZScore { threshold } => {
                threshold_values.insert("z_score_threshold".to_string(), threshold);
            }
            OutlierMethod::IQR => {
                threshold_values.insert("iqr_multiplier".to_string(), 1.5);
            }
        }

        Ok(OutlierAnalysis {
            outlier_count,
            outlier_percentage,
            detection_method: format!("{:?}", self.config.outlier_method),
            threshold_values,
        })
    }

    /// Detect outliers using interquartile range method
    fn detect_outliers_iqr(&self, values: &[f64]) -> usize {
        if values.len() < 4 {
            return 0;
        }

        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let q1 = self.calculate_percentile(&sorted_values, 25.0);
        let q3 = self.calculate_percentile(&sorted_values, 75.0);
        let iqr = q3 - q1;
        
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        values.iter()
            .filter(|&&x| x < lower_bound || x > upper_bound)
            .count()
    }

    /// Detect outliers using standard deviation method
    fn detect_outliers_std_dev(&self, values: &[f64], threshold: f64) -> usize {
        if values.is_empty() {
            return 0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let std_dev = self.calculate_standard_deviation(values, mean);

        values.iter()
            .filter(|&&x| (x - mean).abs() > threshold * std_dev)
            .count()
    }

    /// Detect outliers using modified Z-score method
    fn detect_outliers_modified_z_score(&self, values: &[f64], threshold: f64) -> usize {
        if values.is_empty() {
            return 0;
        }

        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let median = self.calculate_percentile(&sorted_values, 50.0);
        let mad = {
            let deviations: Vec<f64> = values.iter()
                .map(|&x| (x - median).abs())
                .collect();
            let mut sorted_deviations = deviations;
            sorted_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            self.calculate_percentile(&sorted_deviations, 50.0)
        };

        if mad == 0.0 {
            return 0;
        }

        values.iter()
            .filter(|&&x| {
                let modified_z_score = 0.6745 * (x - median) / mad;
                modified_z_score.abs() > threshold
            })
            .count()
    }

    /// Calculate performance distribution
    fn calculate_performance_distribution(&self, timings: &[&TimingMetrics]) -> PerformanceDistribution {
        if timings.is_empty() {
            return PerformanceDistribution {
                good_percentage: 0.0,
                moderate_percentage: 0.0,
                poor_percentage: 0.0,
            };
        }

        let total = timings.len() as f64;
        let mut good_count = 0;
        let mut moderate_count = 0;
        let mut poor_count = 0;

        for timing in timings {
            match timing.performance_level() {
                PerformanceLevel::Good => good_count += 1,
                PerformanceLevel::Moderate => moderate_count += 1,
                PerformanceLevel::Poor => poor_count += 1,
            }
        }

        PerformanceDistribution {
            good_percentage: (good_count as f64 / total) * 100.0,
            moderate_percentage: (moderate_count as f64 / total) * 100.0,
            poor_percentage: (poor_count as f64 / total) * 100.0,
        }
    }

    /// Calculate reliability metrics
    fn calculate_reliability_metrics(&self, results: &[TestResult], successful_timings: &[&TimingMetrics]) -> ReliabilityMetrics {
        let total_attempts: u32 = results.iter().map(|r| r.total_count).sum();
        let successful_attempts: u32 = results.iter().map(|r| r.success_count).sum();

        let success_rate = if total_attempts > 0 {
            (successful_attempts as f64 / total_attempts as f64) * 100.0
        } else {
            0.0
        };

        // Calculate consistency score (coefficient of variation)
        let response_times: Vec<f64> = successful_timings.iter().map(|t| t.total_ms()).collect();
        let consistency_score = if !response_times.is_empty() {
            let mean = response_times.iter().sum::<f64>() / response_times.len() as f64;
            if mean > 0.0 {
                let std_dev = self.calculate_standard_deviation(&response_times, mean);
                std_dev / mean
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate jitter (standard deviation of response times)
        let jitter_ms = if response_times.len() > 1 {
            let mean = response_times.iter().sum::<f64>() / response_times.len() as f64;
            self.calculate_standard_deviation(&response_times, mean)
        } else {
            0.0
        };

        ReliabilityMetrics {
            success_rate,
            consistency_score,
            jitter_ms,
            uptime_percentage: None, // Not applicable for individual tests
        }
    }

    /// Perform comparative analysis between configurations
    fn perform_comparative_analysis(&self, stats: &HashMap<String, ExtendedStatistics>) -> Result<ComparativeAnalysis> {
        if stats.len() < 2 {
            return Ok(ComparativeAnalysis {
                fastest_config: stats.keys().next().cloned(),
                most_reliable_config: stats.keys().next().cloned(),
                most_consistent_config: stats.keys().next().cloned(),
                performance_rankings: Vec::new(),
                significance_tests: Vec::new(),
            });
        }

        // Find best configurations by different metrics
        let fastest_config = stats.iter()
            .min_by(|a, b| a.1.basic.total_avg_ms.partial_cmp(&b.1.basic.total_avg_ms).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone());

        let most_reliable_config = stats.iter()
            .max_by(|a, b| a.1.reliability.success_rate.partial_cmp(&b.1.reliability.success_rate).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone());

        let most_consistent_config = stats.iter()
            .min_by(|a, b| a.1.reliability.consistency_score.partial_cmp(&b.1.reliability.consistency_score).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone());

        // Create performance rankings
        let performance_rankings = self.calculate_performance_rankings(stats);

        // Perform significance tests (simplified implementation)
        let significance_tests = self.perform_significance_tests(stats);

        Ok(ComparativeAnalysis {
            fastest_config,
            most_reliable_config,
            most_consistent_config,
            performance_rankings,
            significance_tests,
        })
    }

    /// Calculate performance rankings for all configurations
    fn calculate_performance_rankings(&self, stats: &HashMap<String, ExtendedStatistics>) -> Vec<ConfigurationRanking> {
        let mut rankings = Vec::new();

        for (config_name, config_stats) in stats {
            let mut metric_scores = HashMap::new();
            
            // Calculate normalized scores (0.0 to 1.0, higher is better)
            let speed_score = self.calculate_speed_score(config_stats, stats);
            let reliability_score = config_stats.reliability.success_rate / 100.0;
            let consistency_score = 1.0 - config_stats.reliability.consistency_score.min(1.0);

            metric_scores.insert("speed".to_string(), speed_score);
            metric_scores.insert("reliability".to_string(), reliability_score);
            metric_scores.insert("consistency".to_string(), consistency_score);

            // Calculate overall score (weighted average)
            let overall_score = (speed_score * 0.4) + (reliability_score * 0.35) + (consistency_score * 0.25);

            rankings.push(ConfigurationRanking {
                config_name: config_name.clone(),
                rank: 0, // Will be set after sorting
                score: overall_score,
                metric_scores,
            });
        }

        // Sort by score and assign ranks
        rankings.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        for (index, ranking) in rankings.iter_mut().enumerate() {
            ranking.rank = index + 1;
        }

        rankings
    }

    /// Calculate speed score relative to other configurations
    fn calculate_speed_score(&self, config_stats: &ExtendedStatistics, all_stats: &HashMap<String, ExtendedStatistics>) -> f64 {
        let avg_time = config_stats.basic.total_avg_ms;
        let min_time = all_stats.values()
            .map(|s| s.basic.total_avg_ms)
            .fold(f64::INFINITY, f64::min);
        let max_time = all_stats.values()
            .map(|s| s.basic.total_avg_ms)
            .fold(f64::NEG_INFINITY, f64::max);

        if max_time <= min_time {
            1.0
        } else {
            1.0 - ((avg_time - min_time) / (max_time - min_time))
        }
    }

    /// Perform statistical significance tests (simplified)
    fn perform_significance_tests(&self, stats: &HashMap<String, ExtendedStatistics>) -> Vec<SignificanceTest> {
        let mut tests = Vec::new();
        let config_names: Vec<_> = stats.keys().collect();

        // Perform pairwise comparisons
        for i in 0..config_names.len() {
            for j in (i + 1)..config_names.len() {
                let config_a = config_names[i];
                let config_b = config_names[j];
                
                let stats_a = &stats[config_a];
                let stats_b = &stats[config_b];

                // Simplified significance test based on confidence intervals
                let mean_diff = (stats_a.basic.total_avg_ms - stats_b.basic.total_avg_ms).abs();
                let combined_std = (stats_a.basic.total_std_dev_ms + stats_b.basic.total_std_dev_ms) / 2.0;
                
                let t_statistic = if combined_std > 0.0 {
                    mean_diff / combined_std
                } else {
                    0.0
                };

                let p_value = if t_statistic > 1.96 { 0.05 } else { 0.1 }; // Simplified
                let is_significant = p_value < 0.05;
                let effect_size = if combined_std > 0.0 { mean_diff / combined_std } else { 0.0 };

                tests.push(SignificanceTest {
                    test_name: "Simplified t-test".to_string(),
                    configurations: (config_a.clone(), config_b.clone()),
                    p_value,
                    is_significant,
                    effect_size,
                });
            }
        }

        tests
    }

    /// Perform trend analysis over time
    fn perform_trend_analysis(&self) -> Result<Option<TrendAnalysis>> {
        // Check if we have enough temporal data
        let has_temporal_data = self.results.values()
            .any(|results| results.len() > 1);

        if !has_temporal_data {
            return Ok(None);
        }

        let mut trends = HashMap::new();
        let mut temporal_correlations = HashMap::new();
        let mut all_timestamps = Vec::new();

        for (config_name, results) in &self.results {
            if results.len() < 3 {
                trends.insert(config_name.clone(), TrendDirection::NoTrend);
                temporal_correlations.insert(config_name.clone(), 0.0);
                continue;
            }

            // Extract timestamps and average response times
            let mut data_points: Vec<(DateTime<Utc>, f64)> = results.iter()
                .filter_map(|r| {
                    r.statistics.as_ref().map(|s| {
                        let timestamp = r.completed_at.unwrap_or(r.started_at);
                        (timestamp, s.total_avg_ms)
                    })
                })
                .collect();

            data_points.sort_by_key(|&(timestamp, _)| timestamp);
            
            if data_points.len() < 3 {
                trends.insert(config_name.clone(), TrendDirection::NoTrend);
                temporal_correlations.insert(config_name.clone(), 0.0);
                continue;
            }

            // Collect all timestamps
            all_timestamps.extend(data_points.iter().map(|&(ts, _)| ts));

            // Calculate trend direction
            let trend_direction = self.calculate_trend_direction(&data_points);
            trends.insert(config_name.clone(), trend_direction);

            // Calculate temporal correlation
            let correlation = self.calculate_temporal_correlation(&data_points);
            temporal_correlations.insert(config_name.clone(), correlation);
        }

        let time_period = if all_timestamps.is_empty() {
            (Utc::now(), Utc::now())
        } else {
            let min_time = *all_timestamps.iter().min().unwrap();
            let max_time = *all_timestamps.iter().max().unwrap();
            (min_time, max_time)
        };

        let patterns = self.detect_patterns(&trends);

        Ok(Some(TrendAnalysis {
            time_period,
            trends,
            temporal_correlations,
            patterns,
        }))
    }

    /// Calculate trend direction from data points
    fn calculate_trend_direction(&self, data_points: &[(DateTime<Utc>, f64)]) -> TrendDirection {
        if data_points.len() < 2 {
            return TrendDirection::NoTrend;
        }

        // Simple linear regression slope
        let n = data_points.len() as f64;
        let x_values: Vec<f64> = (0..data_points.len()).map(|i| i as f64).collect();
        let y_values: Vec<f64> = data_points.iter().map(|&(_, y)| y).collect();

        let sum_x = x_values.iter().sum::<f64>();
        let sum_y = y_values.iter().sum::<f64>();
        let sum_xy = x_values.iter().zip(&y_values).map(|(&x, &y)| x * y).sum::<f64>();
        let sum_x_sq = x_values.iter().map(|&x| x * x).sum::<f64>();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_sq - sum_x * sum_x);

        match slope {
            s if s > 10.0 => TrendDirection::Degrading, // Performance getting worse (higher response times)
            s if s < -10.0 => TrendDirection::Improving, // Performance getting better (lower response times)
            s if s.abs() <= 10.0 => TrendDirection::Stable,
            _ => TrendDirection::NoTrend,
        }
    }

    /// Calculate temporal correlation
    fn calculate_temporal_correlation(&self, data_points: &[(DateTime<Utc>, f64)]) -> f64 {
        if data_points.len() < 2 {
            return 0.0;
        }

        // Simplified correlation calculation
        let y_values: Vec<f64> = data_points.iter().map(|&(_, y)| y).collect();
        let mean_y = y_values.iter().sum::<f64>() / y_values.len() as f64;
        
        // Standard deviation of response times
        let std_y = self.calculate_standard_deviation(&y_values, mean_y);
        
        // Return inverse of coefficient of variation as a correlation proxy
        if std_y > 0.0 && mean_y > 0.0 {
            1.0 - (std_y / mean_y).min(1.0)
        } else {
            0.0
        }
    }

    /// Detect patterns in trends
    fn detect_patterns(&self, trends: &HashMap<String, TrendDirection>) -> Vec<String> {
        let mut patterns = Vec::new();

        let improving_count = trends.values().filter(|&t| matches!(t, TrendDirection::Improving)).count();
        let degrading_count = trends.values().filter(|&t| matches!(t, TrendDirection::Degrading)).count();
        let stable_count = trends.values().filter(|&t| matches!(t, TrendDirection::Stable)).count();

        if improving_count > trends.len() / 2 {
            patterns.push("Most configurations showing performance improvements".to_string());
        }

        if degrading_count > trends.len() / 2 {
            patterns.push("Most configurations showing performance degradation".to_string());
        }

        if stable_count > trends.len() / 2 {
            patterns.push("Most configurations showing stable performance".to_string());
        }

        if patterns.is_empty() {
            patterns.push("Mixed performance trends across configurations".to_string());
        }

        patterns
    }

    /// Generate analysis summary and recommendations
    fn generate_summary(&self, stats: &HashMap<String, ExtendedStatistics>, comparative: &ComparativeAnalysis) -> Result<AnalysisSummary> {
        let mut key_findings = Vec::new();
        let mut insights = Vec::new();
        let mut recommendations = Vec::new();
        let mut warnings = Vec::new();

        // Generate key findings
        if let Some(fastest) = &comparative.fastest_config {
            if let Some(fastest_stats) = stats.get(fastest) {
                key_findings.push(format!("Fastest configuration: {} ({:.1}ms average)", 
                    fastest, fastest_stats.basic.total_avg_ms));
            }
        }

        if let Some(most_reliable) = &comparative.most_reliable_config {
            if let Some(reliable_stats) = stats.get(most_reliable) {
                key_findings.push(format!("Most reliable configuration: {} ({:.1}% success rate)", 
                    most_reliable, reliable_stats.reliability.success_rate));
            }
        }

        // Generate insights
        let total_configs = stats.len();
        let good_performance_configs = stats.values()
            .filter(|s| s.performance_distribution.good_percentage > 50.0)
            .count();

        insights.push(format!("{} of {} configurations show good performance (>50% of tests under 1 second)", 
            good_performance_configs, total_configs));

        // Generate recommendations
        if let Some(recommended) = comparative.performance_rankings.first() {
            recommendations.push(format!("Recommended DNS configuration: {} (overall score: {:.2})", 
                recommended.config_name, recommended.score));
        }

        // Check for performance issues
        let avg_success_rate: f64 = stats.values().map(|s| s.reliability.success_rate).sum::<f64>() / stats.len() as f64;
        if avg_success_rate < 95.0 {
            warnings.push(format!("Low overall success rate detected: {:.1}%", avg_success_rate));
        }

        // Check for high variability
        let high_jitter_configs: Vec<String> = stats.iter()
            .filter(|(_, s)| s.reliability.jitter_ms > 100.0)
            .map(|(name, _)| name.clone())
            .collect();

        if !high_jitter_configs.is_empty() {
            warnings.push(format!("High response time variability detected in: {}", high_jitter_configs.join(", ")));
        }

        Ok(AnalysisSummary {
            recommended_config: comparative.performance_rankings.first().map(|r| r.config_name.clone()),
            key_findings,
            insights,
            recommendations,
            warnings,
        })
    }

    /// Export statistical analysis to JSON
    pub fn export_json(&self, analysis: &StatisticalAnalysis) -> Result<String> {
        serde_json::to_string_pretty(analysis)
            .map_err(|e| AppError::io(format!("Failed to export analysis to JSON: {}", e)))
    }

    /// Clear all collected results
    pub fn clear(&mut self) {
        self.results.clear();
    }

    /// Get the number of configurations analyzed
    pub fn config_count(&self) -> usize {
        self.results.len()
    }

    /// Get the total number of test results
    pub fn total_results(&self) -> usize {
        self.results.values().map(|r| r.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DnsConfig;
    use std::time::Duration;

    #[test]
    fn test_statistics_engine_creation() {
        let engine = StatisticsEngine::with_defaults();
        assert_eq!(engine.config_count(), 0);
        assert_eq!(engine.total_results(), 0);
    }

    #[test]
    fn test_add_results() {
        let mut engine = StatisticsEngine::with_defaults();
        
        let mut test_result = TestResult::new(
            "Test Config".to_string(),
            DnsConfig::System,
            "https://example.com".to_string(),
        );
        
        test_result.add_measurement(TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            None,
            Duration::from_millis(50),
            Duration::from_millis(100),
            200,
        ));
        
        engine.add_result(test_result);
        
        assert_eq!(engine.config_count(), 1);
        assert_eq!(engine.total_results(), 1);
    }

    #[test]
    fn test_percentile_calculation() {
        let engine = StatisticsEngine::with_defaults();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        
        assert_eq!(engine.calculate_percentile(&values, 50.0), 5.5);
        assert_eq!(engine.calculate_percentile(&values, 90.0), 9.1);
        assert_eq!(engine.calculate_percentile(&values, 100.0), 10.0);
    }

    #[test]
    fn test_outlier_detection_iqr() {
        let engine = StatisticsEngine::with_defaults();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0]; // 100.0 is an outlier
        
        let outlier_count = engine.detect_outliers_iqr(&values);
        assert_eq!(outlier_count, 1);
    }

    #[test]
    fn test_outlier_detection_std_dev() {
        let engine = StatisticsEngine::with_defaults();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0]; // 100.0 is an outlier
        
        let outlier_count = engine.detect_outliers_std_dev(&values, 2.0);
        assert_eq!(outlier_count, 1);
    }

    #[test]
    fn test_performance_distribution() {
        let engine = StatisticsEngine::with_defaults();
        
        let timing1 = TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            None,
            Duration::from_millis(50),
            Duration::from_millis(500), // Good performance
            200,
        );
        
        let timing2 = TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            None,
            Duration::from_millis(50),
            Duration::from_millis(2000), // Moderate performance
            200,
        );
        
        let timing3 = TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            None,
            Duration::from_millis(50),
            Duration::from_millis(5000), // Poor performance
            200,
        );
        
        let timings = vec![&timing1, &timing2, &timing3];
        let distribution = engine.calculate_performance_distribution(&timings);
        
        assert!((distribution.good_percentage - 33.33).abs() < 0.1);
        assert!((distribution.moderate_percentage - 33.33).abs() < 0.1);
        assert!((distribution.poor_percentage - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_trend_direction_calculation() {
        let engine = StatisticsEngine::with_defaults();
        
        // Improving trend (decreasing response times)
        let improving_data = vec![
            (Utc::now() - chrono::Duration::hours(3), 1000.0),
            (Utc::now() - chrono::Duration::hours(2), 800.0),
            (Utc::now() - chrono::Duration::hours(1), 600.0),
            (Utc::now(), 400.0),
        ];
        
        let trend = engine.calculate_trend_direction(&improving_data);
        assert!(matches!(trend, TrendDirection::Improving));
        
        // Degrading trend (increasing response times)
        let degrading_data = vec![
            (Utc::now() - chrono::Duration::hours(3), 400.0),
            (Utc::now() - chrono::Duration::hours(2), 600.0),
            (Utc::now() - chrono::Duration::hours(1), 800.0),
            (Utc::now(), 1000.0),
        ];
        
        let trend = engine.calculate_trend_direction(&degrading_data);
        assert!(matches!(trend, TrendDirection::Degrading));
    }

    #[test]
    fn test_z_score_calculation() {
        let engine = StatisticsEngine::with_defaults();
        
        assert_eq!(engine.get_z_score(0.90), 1.645);
        assert_eq!(engine.get_z_score(0.95), 1.96);
        assert_eq!(engine.get_z_score(0.99), 2.576);
    }

    #[test]
    fn test_empty_analysis() {
        let engine = StatisticsEngine::with_defaults();
        let result = engine.analyze();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[test]
    fn test_configuration_ranking() {
        let mut stats = HashMap::new();
        
        let good_stats = ExtendedStatistics {
            basic: Statistics {
                total_avg_ms: 100.0,
                total_std_dev_ms: 10.0,
                success_rate: 100.0,
                sample_count: 100,
                dns_avg_ms: 10.0,
                tcp_avg_ms: 20.0,
                first_byte_avg_ms: 50.0,
                total_min_ms: 80.0,
                total_max_ms: 120.0,
            },
            percentiles: HashMap::new(),
            confidence_intervals: ConfidenceIntervals {
                level: 0.95,
                avg_response_time: (95.0, 105.0),
                success_rate: (98.0, 100.0),
                dns_resolution_time: (8.0, 12.0),
            },
            outlier_analysis: OutlierAnalysis {
                outlier_count: 0,
                outlier_percentage: 0.0,
                detection_method: "IQR".to_string(),
                threshold_values: HashMap::new(),
            },
            performance_distribution: PerformanceDistribution {
                good_percentage: 100.0,
                moderate_percentage: 0.0,
                poor_percentage: 0.0,
            },
            reliability: ReliabilityMetrics {
                success_rate: 100.0,
                consistency_score: 0.1,
                jitter_ms: 10.0,
                uptime_percentage: None,
            },
        };
        
        stats.insert("Good Config".to_string(), good_stats);
        
        let engine = StatisticsEngine::with_defaults();
        let rankings = engine.calculate_performance_rankings(&stats);
        
        assert_eq!(rankings.len(), 1);
        assert_eq!(rankings[0].config_name, "Good Config");
        assert_eq!(rankings[0].rank, 1);
        assert!(rankings[0].score > 0.8); // Should have high score
    }
}

// Additional comprehensive tests in separate module
#[cfg(test)]
mod comprehensive_tests;