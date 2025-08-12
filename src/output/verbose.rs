//! Verbose mode output with detailed timing information
//!
//! This module provides comprehensive timing output for verbose mode,
//! integrating with the logging system to provide detailed request-level
//! timing information, performance analysis, and diagnostic data.

use crate::{
    error::{AppError, Result},
    logging::{Logger, PerformanceLogger},
    models::{Config, TestResult, TimingMetrics},
    stats::StatisticalAnalysis,
    executor::ExecutionResults,
    diagnostics::DiagnosticReport,
};
use std::{
    fmt::Write as _,
    collections::HashMap,
};
use colored::Colorize;

/// Verbose timing output formatter
pub struct VerboseTimingFormatter {
    /// Application configuration
    config: Config,
    /// Performance logger for timing analysis
    perf_logger: PerformanceLogger,
    /// General logger
    logger: Logger,
    /// Enable colored output
    use_color: bool,
}

impl VerboseTimingFormatter {
    /// Create a new verbose timing formatter
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            perf_logger: PerformanceLogger::new(config),
            logger: Logger::with_config("VERBOSE_OUTPUT".to_string(), config),
            use_color: config.enable_color,
        }
    }

    /// Format complete verbose output for execution results
    pub async fn format_verbose_results(&self, results: &ExecutionResults) -> Result<String> {
        let mut output = String::new();

        // Header with timing overview
        output.push_str(&self.format_verbose_header(results)?);
        output.push_str("\n\n");

        // Detailed timing breakdown for each configuration
        output.push_str(&self.format_detailed_timing_breakdown(results).await?);
        output.push_str("\n\n");

        // Individual request timing details
        output.push_str(&self.format_individual_request_timings(results).await?);
        output.push_str("\n\n");

        // Performance analysis
        if let Some(ref analysis) = results.statistical_analysis {
            output.push_str(&self.format_performance_analysis(analysis).await?);
            output.push_str("\n\n");
        }

        // Diagnostic timing information
        if let Some(ref diagnostics) = results.diagnostics_report {
            output.push_str(&self.format_diagnostic_timings(diagnostics).await?);
            output.push_str("\n\n");
        }

        // Timing recommendations
        output.push_str(&self.format_timing_recommendations(results).await?);

        Ok(output)
    }

    /// Format verbose header with execution overview
    fn format_verbose_header(&self, results: &ExecutionResults) -> Result<String> {
        let mut output = String::new();

        let title = if self.use_color {
            "🕐 DETAILED TIMING ANALYSIS".bright_cyan().bold()
        } else {
            "DETAILED TIMING ANALYSIS".to_string().into()
        };

        writeln!(output, "{}", title)
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        writeln!(output, "{}", "=".repeat(50))
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;

        // Execution timing overview
        let total_duration = results.execution_summary.total_duration;
        let avg_per_test = if results.execution_summary.total_tests > 0 {
            total_duration.as_secs_f64() / results.execution_summary.total_tests as f64
        } else {
            0.0
        };

        writeln!(output, "Total Execution Time:    {:.3}s", total_duration.as_secs_f64())
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        writeln!(output, "Average Time per Test:   {:.3}s", avg_per_test)
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        writeln!(output, "Total Tests Executed:    {}", results.execution_summary.total_tests)
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        writeln!(output, "Successful Tests:        {} ({:.1}%)", 
            results.execution_summary.successful_tests,
            results.execution_summary.success_rate)
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;

        Ok(output)
    }

    /// Format detailed timing breakdown by configuration
    async fn format_detailed_timing_breakdown(&self, results: &ExecutionResults) -> Result<String> {
        let mut output = String::new();

        let section_title = if self.use_color {
            "\n📊 TIMING BREAKDOWN BY CONFIGURATION".yellow().bold()
        } else {
            "\nTIMING BREAKDOWN BY CONFIGURATION".to_string().into()
        };

        writeln!(output, "{}", section_title)
            .map_err(|e| AppError::io(format!("Failed to format timing breakdown: {}", e)))?;
        writeln!(output, "{}", "-".repeat(45))
            .map_err(|e| AppError::io(format!("Failed to format timing breakdown: {}", e)))?;

        // Sort configurations by average performance
        let mut sorted_results: Vec<_> = results.test_results.iter().collect();
        sorted_results.sort_by(|a, b| {
            let a_avg = a.1.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
            let b_avg = b.1.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
            a_avg.partial_cmp(&b_avg).unwrap_or(std::cmp::Ordering::Equal)
        });

        for (config_name, test_result) in sorted_results {
            output.push_str(&self.format_configuration_timing_details(config_name, test_result).await?);
            output.push('\n');
        }

        Ok(output)
    }

    /// Format timing details for a specific configuration
    async fn format_configuration_timing_details(&self, config_name: &str, result: &TestResult) -> Result<String> {
        let mut output = String::new();

        let config_header = if self.use_color {
            format!("🔧 {}", config_name).green().bold()
        } else {
            format!("Configuration: {}", config_name).into()
        };

        writeln!(output, "{}", config_header)
            .map_err(|e| AppError::io(format!("Failed to format config timing: {}", e)))?;

        if let Some(ref stats) = result.statistics {
            // Timing component breakdown
            writeln!(output, "  DNS Resolution:     {:.3}ms (avg)", stats.dns_avg_ms)
                .map_err(|e| AppError::io(format!("Failed to format config timing: {}", e)))?;
            writeln!(output, "  TCP Connection:     {:.3}ms (avg)", stats.tcp_avg_ms)
                .map_err(|e| AppError::io(format!("Failed to format config timing: {}", e)))?;
            writeln!(output, "  First Byte:         {:.3}ms (avg)", stats.first_byte_avg_ms)
                .map_err(|e| AppError::io(format!("Failed to format config timing: {}", e)))?;
            writeln!(output, "  Total Response:     {:.3}ms (avg ± {:.3}ms)", 
                stats.total_avg_ms, stats.total_std_dev_ms)
                .map_err(|e| AppError::io(format!("Failed to format config timing: {}", e)))?;
            writeln!(output, "  Response Range:     {:.3}ms - {:.3}ms", 
                stats.total_min_ms, stats.total_max_ms)
                .map_err(|e| AppError::io(format!("Failed to format config timing: {}", e)))?;

            // Performance assessment
            let performance_level = result.performance_level()
                .map(|p| format!("{:?}", p))
                .unwrap_or_else(|| "Unknown".to_string());
            
            let performance_color = if self.use_color {
                match result.performance_level() {
                    Some(crate::types::PerformanceLevel::Good) => performance_level.green(),
                    Some(crate::types::PerformanceLevel::Moderate) => performance_level.yellow(),
                    Some(crate::types::PerformanceLevel::Poor) => performance_level.red(),
                    _ => performance_level.into(),
                }
            } else {
                performance_level.into()
            };

            writeln!(output, "  Performance Level:  {}", performance_color)
                .map_err(|e| AppError::io(format!("Failed to format config timing: {}", e)))?;
            writeln!(output, "  Success Rate:       {:.1}% ({}/{} tests)", 
                result.success_rate(), result.success_count, result.total_count)
                .map_err(|e| AppError::io(format!("Failed to format config timing: {}", e)))?;

            // Log performance metrics
            self.perf_logger.log_operation_complete(
                config_name,
                std::time::Duration::from_millis(stats.total_avg_ms as u64),
                stats.sample_count,
                result.success_rate(),
                Some(&format!("Config performance summary: avg={:.3}ms, range={:.3}-{:.3}ms", 
                    stats.total_avg_ms, stats.total_min_ms, stats.total_max_ms))
            ).await;
        } else {
            let error_msg = if self.use_color {
                "  No successful tests - timing data unavailable".red()
            } else {
                "  No successful tests - timing data unavailable".to_string().into()
            };
            writeln!(output, "{}", error_msg)
                .map_err(|e| AppError::io(format!("Failed to format config timing: {}", e)))?;
        }

        Ok(output)
    }

    /// Format individual request timing details
    async fn format_individual_request_timings(&self, results: &ExecutionResults) -> Result<String> {
        let mut output = String::new();

        let section_title = if self.use_color {
            "\n🔍 INDIVIDUAL REQUEST TIMINGS".blue().bold()
        } else {
            "\nINDIVIDUAL REQUEST TIMINGS".to_string().into()
        };

        writeln!(output, "{}", section_title)
            .map_err(|e| AppError::io(format!("Failed to format individual timings: {}", e)))?;
        writeln!(output, "{}", "-".repeat(35))
            .map_err(|e| AppError::io(format!("Failed to format individual timings: {}", e)))?;

        for (config_name, test_result) in &results.test_results {
            if test_result.individual_results.is_empty() {
                continue;
            }

            let config_header = if self.use_color {
                format!("\n📋 {} - Individual Test Results:", config_name).cyan().bold()
            } else {
                format!("\n{} - Individual Test Results:", config_name).into()
            };
            writeln!(output, "{}", config_header)
                .map_err(|e| AppError::io(format!("Failed to format individual timings: {}", e)))?;

            // Table header
            writeln!(output, "{:>4} {:>12} {:>12} {:>12} {:>12} {:>12} {:>8} {:>20}", 
                "Test", "DNS (ms)", "TCP (ms)", "TLS (ms)", "FirstByte", "Total (ms)", "Status", "Timestamp")
                .map_err(|e| AppError::io(format!("Failed to format individual timings: {}", e)))?;
            writeln!(output, "{}", "-".repeat(100))
                .map_err(|e| AppError::io(format!("Failed to format individual timings: {}", e)))?;

            // Individual test results
            for (i, timing) in test_result.individual_results.iter().enumerate() {
                output.push_str(&self.format_individual_timing_row(i + 1, timing).await?);
                output.push('\n');
            }

            // Log individual timing analysis
            self.logger.info(&format!("Individual timing analysis for {}: {} tests, {} successful", 
                config_name, test_result.total_count, test_result.success_count))
                .field("config_name", config_name)
                .field("total_tests", test_result.total_count)
                .field("successful_tests", test_result.success_count)
                .log().await;
        }

        Ok(output)
    }

    /// Format a single timing measurement row
    async fn format_individual_timing_row(&self, test_num: usize, timing: &TimingMetrics) -> Result<String> {
        let dns_str = if timing.dns_resolution.as_millis() > 0 {
            format!("{:.3}", timing.dns_ms())
        } else {
            "N/A".to_string()
        };

        let tcp_str = if timing.tcp_connection.as_millis() > 0 {
            format!("{:.3}", timing.tcp_ms())
        } else {
            "N/A".to_string()
        };

        let tls_str = if let Some(tls_ms) = timing.tls_ms() {
            format!("{:.3}", tls_ms)
        } else {
            "N/A".to_string()
        };

        let first_byte_str = if timing.first_byte.as_millis() > 0 {
            format!("{:.3}", timing.first_byte_ms())
        } else {
            "N/A".to_string()
        };

        let status_str = match timing.status {
            crate::types::TestStatus::Success => {
                if self.use_color {
                    format!("✓ {}", timing.http_status).green()
                } else {
                    format!("OK {}", timing.http_status).into()
                }
            },
            crate::types::TestStatus::Failed => {
                if self.use_color {
                    "✗ FAIL".red()
                } else {
                    "FAIL".to_string().into()
                }
            },
            crate::types::TestStatus::Timeout => {
                if self.use_color {
                    "⏰ TIMEOUT".yellow()
                } else {
                    "TIMEOUT".to_string().into()
                }
            },
            crate::types::TestStatus::Skipped => {
                if self.use_color {
                    "⏩ SKIP".blue()
                } else {
                    "SKIP".to_string().into()
                }
            },
        };

        let timestamp_str = timing.timestamp.format("%H:%M:%S%.3f").to_string();

        Ok(format!("{:>4} {:>12} {:>12} {:>12} {:>12} {:>12} {:>8} {:>20}", 
            test_num,
            dns_str,
            tcp_str,
            tls_str,
            first_byte_str,
            format!("{:.3}", timing.total_ms()),
            status_str,
            timestamp_str
        ))
    }

    /// Format performance analysis with timing insights
    async fn format_performance_analysis(&self, analysis: &StatisticalAnalysis) -> Result<String> {
        let mut output = String::new();

        let section_title = if self.use_color {
            "\n⚡ PERFORMANCE TIMING ANALYSIS".magenta().bold()
        } else {
            "\nPERFORMANCE TIMING ANALYSIS".to_string().into()
        };

        writeln!(output, "{}", section_title)
            .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;
        writeln!(output, "{}", "-".repeat(40))
            .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;

        // Overall timing statistics
        let total_avg = analysis.basic_stats.values()
            .map(|stats| stats.basic.total_avg_ms)
            .sum::<f64>() / analysis.basic_stats.len() as f64;
            
        let fastest_config = analysis.basic_stats.iter()
            .min_by(|a, b| a.1.basic.total_avg_ms.partial_cmp(&b.1.basic.total_avg_ms).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.as_str());
            
        let slowest_config = analysis.basic_stats.iter()
            .max_by(|a, b| a.1.basic.total_avg_ms.partial_cmp(&b.1.basic.total_avg_ms).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.as_str());

        writeln!(output, "Average Response Time:   {:.3}ms", total_avg)
            .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;
        
        if let Some(fastest) = fastest_config {
            let fastest_time = analysis.basic_stats.get(fastest).unwrap().basic.total_avg_ms;
            let fastest_display = if self.use_color {
                format!("{} ({:.3}ms)", fastest, fastest_time).green().bold()
            } else {
                format!("{} ({:.3}ms)", fastest, fastest_time).into()
            };
            writeln!(output, "Fastest Configuration:   {}", fastest_display)
                .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;
        }

        if let Some(slowest) = slowest_config {
            let slowest_time = analysis.basic_stats.get(slowest).unwrap().basic.total_avg_ms;
            let slowest_display = if self.use_color {
                format!("{} ({:.3}ms)", slowest, slowest_time).red()
            } else {
                format!("{} ({:.3}ms)", slowest, slowest_time).into()
            };
            writeln!(output, "Slowest Configuration:   {}", slowest_display)
                .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;
        }

        // Timing component analysis
        writeln!(output, "\nTiming Component Analysis:")
            .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;

        let avg_dns = analysis.basic_stats.values()
            .map(|stats| stats.basic.dns_avg_ms)
            .sum::<f64>() / analysis.basic_stats.len() as f64;
        let avg_tcp = analysis.basic_stats.values()
            .map(|stats| stats.basic.tcp_avg_ms)
            .sum::<f64>() / analysis.basic_stats.len() as f64;
        let avg_first_byte = analysis.basic_stats.values()
            .map(|stats| stats.basic.first_byte_avg_ms)
            .sum::<f64>() / analysis.basic_stats.len() as f64;

        writeln!(output, "  Average DNS Resolution:  {:.3}ms ({:.1}% of total)", avg_dns, (avg_dns / total_avg) * 100.0)
            .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;
        writeln!(output, "  Average TCP Connection:  {:.3}ms ({:.1}% of total)", avg_tcp, (avg_tcp / total_avg) * 100.0)
            .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;
        writeln!(output, "  Average First Byte:      {:.3}ms ({:.1}% of total)", avg_first_byte, (avg_first_byte / total_avg) * 100.0)
            .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;

        // Performance insights
        writeln!(output, "\nPerformance Insights:")
            .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;

        if avg_dns > total_avg * 0.3 {
            let insight = if self.use_color {
                "• DNS resolution is a significant bottleneck (>30% of total time)".yellow()
            } else {
                "• DNS resolution is a significant bottleneck (>30% of total time)".to_string().into()
            };
            writeln!(output, "{}", insight)
                .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;
        }

        if avg_tcp > total_avg * 0.4 {
            let insight = if self.use_color {
                "• TCP connection time is high (>40% of total time) - check network latency".yellow()
            } else {
                "• TCP connection time is high (>40% of total time) - check network latency".to_string().into()
            };
            writeln!(output, "{}", insight)
                .map_err(|e| AppError::io(format!("Failed to format performance analysis: {}", e)))?;
        }

        // Log performance analysis
        self.perf_logger.log_batch_summary(
            analysis.basic_stats.len(),
            std::time::Duration::from_millis(total_avg as u64),
            Some(&format!("Performance analysis: avg={:.3}ms, DNS={:.3}ms, TCP={:.3}ms", 
                total_avg, avg_dns, avg_tcp))
        ).await;

        Ok(output)
    }

    /// Format diagnostic timing information
    async fn format_diagnostic_timings(&self, diagnostics: &DiagnosticReport) -> Result<String> {
        let mut output = String::new();

        let section_title = if self.use_color {
            "\n🔍 DIAGNOSTIC TIMING INFORMATION".cyan().bold()
        } else {
            "\nDIAGNOSTIC TIMING INFORMATION".to_string().into()
        };

        writeln!(output, "{}", section_title)
            .map_err(|e| AppError::io(format!("Failed to format diagnostic timings: {}", e)))?;
        writeln!(output, "{}", "-".repeat(40))
            .map_err(|e| AppError::io(format!("Failed to format diagnostic timings: {}", e)))?;

        // System health timing assessment
        match diagnostics.system_health.status {
            crate::diagnostics::HealthStatus::Healthy => {
                let status_msg = if self.use_color {
                    "✓ System timing performance is optimal".green()
                } else {
                    "System timing performance is optimal".to_string().into()
                };
                writeln!(output, "{}", status_msg)
                    .map_err(|e| AppError::io(format!("Failed to format diagnostic timings: {}", e)))?;
            },
            _ => {
                let status_msg = if self.use_color {
                    "⚠ System timing issues detected:".yellow().bold()
                } else {
                    "System timing issues detected:".to_string().into()
                };
                writeln!(output, "{}", status_msg)
                    .map_err(|e| AppError::io(format!("Failed to format diagnostic timings: {}", e)))?;

                for issue in &diagnostics.system_health.critical_issues {
                    writeln!(output, "  - CRITICAL: {}", issue)
                        .map_err(|e| AppError::io(format!("Failed to format diagnostic timings: {}", e)))?;
                }

                for issue in &diagnostics.system_health.warning_issues {
                    writeln!(output, "  - WARNING: {}", issue)
                        .map_err(|e| AppError::io(format!("Failed to format diagnostic timings: {}", e)))?;
                }
            }
        }

        // Target reachability with timing
        if !diagnostics.connectivity_diagnostics.target_reachability.is_empty() {
            writeln!(output, "\nTarget Connectivity Timings:")
                .map_err(|e| AppError::io(format!("Failed to format diagnostic timings: {}", e)))?;
            
            for (target, status) in &diagnostics.connectivity_diagnostics.target_reachability {
                let timing_info = if let Some(response_time) = status.response_time {
                    let time_ms = response_time.as_secs_f64() * 1000.0;
                    let timing_color = if self.use_color {
                        if time_ms < 100.0 {
                            format!("{:.3}ms", time_ms).green()
                        } else if time_ms < 500.0 {
                            format!("{:.3}ms", time_ms).yellow()
                        } else {
                            format!("{:.3}ms", time_ms).red()
                        }
                    } else {
                        format!("{:.3}ms", time_ms).into()
                    };
                    format!("✓ {} - {}", target, timing_color)
                } else {
                    let error_msg = if let Some(ref error) = status.error_message {
                        format!("✗ {} - Failed: {}", target, error)
                    } else {
                        format!("✗ {} - No timing data", target)
                    };
                    
                    if self.use_color {
                        error_msg.red().to_string()
                    } else {
                        error_msg
                    }
                };

                writeln!(output, "  {}", timing_info)
                    .map_err(|e| AppError::io(format!("Failed to format diagnostic timings: {}", e)))?;
            }
        }

        Ok(output)
    }

    /// Format timing-based recommendations
    async fn format_timing_recommendations(&self, results: &ExecutionResults) -> Result<String> {
        let mut output = String::new();

        let section_title = if self.use_color {
            "\n💡 TIMING OPTIMIZATION RECOMMENDATIONS".bright_yellow().bold()
        } else {
            "\nTIMING OPTIMIZATION RECOMMENDATIONS".to_string().into()
        };

        writeln!(output, "{}", section_title)
            .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        writeln!(output, "{}", "-".repeat(45))
            .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;

        let mut recommendations = Vec::new();

        // Analyze overall performance
        let overall_avg = results.test_results.values()
            .filter_map(|r| r.statistics.as_ref())
            .map(|s| s.total_avg_ms)
            .sum::<f64>() / results.test_results.len() as f64;

        if overall_avg > 1000.0 {
            recommendations.push("• Overall response times are high (>1s) - consider network optimization".to_string());
        } else if overall_avg > 500.0 {
            recommendations.push("• Response times are above optimal range (>500ms) - monitor network conditions".to_string());
        } else if overall_avg < 100.0 {
            recommendations.push("• Excellent response times (<100ms) - current configuration is optimal".to_string());
        }

        // DNS timing recommendations
        let avg_dns = results.test_results.values()
            .filter_map(|r| r.statistics.as_ref())
            .map(|s| s.dns_avg_ms)
            .sum::<f64>() / results.test_results.len() as f64;

        if avg_dns > 200.0 {
            recommendations.push("• DNS resolution is slow (>200ms) - consider using faster DNS servers".to_string());
        } else if avg_dns > 100.0 {
            recommendations.push("• DNS resolution could be optimized (>100ms) - test different DNS providers".to_string());
        }

        // Configuration-specific recommendations
        if let Some(fastest_config) = results.test_results.iter()
            .min_by(|a, b| {
                let a_time = a.1.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
                let b_time = b.1.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
                a_time.partial_cmp(&b_time).unwrap_or(std::cmp::Ordering::Equal)
            }) {
            recommendations.push(format!("• Use '{}' configuration for best performance", fastest_config.0));
        }

        // Success rate based timing recommendations
        if results.execution_summary.success_rate < 95.0 {
            recommendations.push("• Low success rate may indicate timeout issues - consider increasing timeout values".to_string());
        }

        let recommendation_count = recommendations.len();

        // Output recommendations
        if recommendations.is_empty() {
            writeln!(output, "No specific timing optimizations needed - performance is acceptable.")
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        } else {
            for recommendation in &recommendations {
                writeln!(output, "{}", recommendation)
                    .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
            }
        }

        // Log recommendations summary
        self.logger.info("Generated timing optimization recommendations")
            .field("recommendation_count", recommendation_count)
            .field("overall_avg_ms", overall_avg)
            .field("dns_avg_ms", avg_dns)
            .log().await;

        Ok(output)
    }

    /// Format timing summary for console output
    pub async fn format_console_timing_summary(&self, results: &ExecutionResults) -> Result<String> {
        let mut output = String::new();

        let best_config = results.best_config().unwrap_or("Unknown");
        let best_time = results.test_results.get(best_config)
            .and_then(|r| r.statistics.as_ref())
            .map(|s| format!("{:.1}ms", s.total_avg_ms))
            .unwrap_or_else(|| "N/A".to_string());

        let timing_summary = if self.use_color {
            format!("🚀 Best: {} ({}) | Avg: {:.1}ms | Success: {:.1}%",
                best_config, best_time,
                results.test_results.values()
                    .filter_map(|r| r.statistics.as_ref())
                    .map(|s| s.total_avg_ms)
                    .sum::<f64>() / results.test_results.len() as f64,
                results.execution_summary.success_rate
            ).bright_green()
        } else {
            format!("Best: {} ({}) | Avg: {:.1}ms | Success: {:.1}%",
                best_config, best_time,
                results.test_results.values()
                    .filter_map(|r| r.statistics.as_ref())
                    .map(|s| s.total_avg_ms)
                    .sum::<f64>() / results.test_results.len() as f64,
                results.execution_summary.success_rate
            ).into()
        };

        writeln!(output, "{}", timing_summary)
            .map_err(|e| AppError::io(format!("Failed to format console summary: {}", e)))?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{TimingMetrics, Statistics, TestResult},
        types::{DnsConfig, TestStatus},
        executor::{ExecutionSummary, ExecutionResults},
    };
    use std::time::Duration;
    use chrono::Utc;

    fn create_test_config() -> Config {
        Config {
            debug: false,
            verbose: true,
            enable_color: false,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_verbose_timing_formatter_creation() {
        let config = create_test_config();
        let formatter = VerboseTimingFormatter::new(&config);
        
        assert!(!formatter.use_color);
        assert_eq!(formatter.config.verbose, true);
    }

    #[test]
    fn test_format_individual_timing_row() {
        tokio_test::block_on(async {
            let config = create_test_config();
            let formatter = VerboseTimingFormatter::new(&config);
            
            let timing = TimingMetrics::success(
                Duration::from_millis(10),
                Duration::from_millis(20),
                Some(Duration::from_millis(30)),
                Duration::from_millis(50),
                Duration::from_millis(100),
                200,
            );
            
            let row = formatter.format_individual_timing_row(1, &timing).await.unwrap();
            
            assert!(row.contains("10.000"));  // DNS timing
            assert!(row.contains("20.000"));  // TCP timing
            assert!(row.contains("30.000"));  // TLS timing
            assert!(row.contains("100.000")); // Total timing
            assert!(row.contains("OK 200"));  // Status
        });
    }

    #[tokio::test]
    async fn test_verbose_header_formatting() {
        let config = create_test_config();
        let formatter = VerboseTimingFormatter::new(&config);
        
        let summary = ExecutionSummary {
            total_duration: Duration::from_secs(5),
            total_tests: 10,
            successful_tests: 9,
            failed_tests: 1,
            timeout_tests: 0,
            skipped_tests: 0,
            success_rate: 90.0,
            performance_summary: HashMap::new(),
        };
        
        let results = ExecutionResults {
            execution_summary: summary,
            test_results: HashMap::new(),
            statistical_analysis: None,
            diagnostics_report: None,
        };
        
        let header = formatter.format_verbose_header(&results).unwrap();
        
        assert!(header.contains("DETAILED TIMING ANALYSIS"));
        assert!(header.contains("5.000s"));  // Total execution time
        assert!(header.contains("0.500s"));  // Average per test
        assert!(header.contains("90.0%"));   // Success rate
    }

    #[tokio::test]
    async fn test_configuration_timing_details() {
        let config = create_test_config();
        let formatter = VerboseTimingFormatter::new(&config);
        
        let stats = Statistics {
            dns_avg_ms: 10.0,
            tcp_avg_ms: 20.0,
            first_byte_avg_ms: 50.0,
            total_avg_ms: 100.0,
            total_min_ms: 80.0,
            total_max_ms: 120.0,
            total_std_dev_ms: 15.0,
            success_rate: 100.0,
            sample_count: 5,
        };
        
        let mut result = TestResult::new(
            "Test Config".to_string(),
            DnsConfig::System,
            "https://example.com".to_string(),
        );
        result.statistics = Some(stats);
        result.success_count = 5;
        result.total_count = 5;
        
        let details = formatter.format_configuration_timing_details("Test Config", &result).await.unwrap();
        
        assert!(details.contains("DNS Resolution:     10.000ms"));
        assert!(details.contains("TCP Connection:     20.000ms"));
        assert!(details.contains("Total Response:     100.000ms (avg ± 15.000ms)"));
        assert!(details.contains("Response Range:     80.000ms - 120.000ms"));
        assert!(details.contains("Success Rate:       100.0% (5/5 tests)"));
    }

    #[tokio::test]
    async fn test_console_timing_summary() {
        let config = create_test_config();
        let formatter = VerboseTimingFormatter::new(&config);
        
        let stats = Statistics {
            dns_avg_ms: 10.0,
            tcp_avg_ms: 20.0,
            first_byte_avg_ms: 50.0,
            total_avg_ms: 100.0,
            total_min_ms: 80.0,
            total_max_ms: 120.0,
            total_std_dev_ms: 15.0,
            success_rate: 100.0,
            sample_count: 5,
        };
        
        let mut result = TestResult::new(
            "Fast Config".to_string(),
            DnsConfig::System,
            "https://example.com".to_string(),
        );
        result.statistics = Some(stats);
        
        let mut test_results = HashMap::new();
        test_results.insert("Fast Config".to_string(), result);
        
        let summary = ExecutionSummary {
            total_duration: Duration::from_secs(1),
            total_tests: 5,
            successful_tests: 5,
            failed_tests: 0,
            timeout_tests: 0,
            skipped_tests: 0,
            success_rate: 100.0,
            performance_summary: HashMap::new(),
        };
        
        let results = ExecutionResults {
            execution_summary: summary,
            test_results,
            statistical_analysis: None,
            diagnostics_report: None,
        };
        
        let console_summary = formatter.format_console_timing_summary(&results).await.unwrap();
        
        assert!(console_summary.contains("Fast Config"));
        assert!(console_summary.contains("100.0ms"));
        assert!(console_summary.contains("100.0%"));
    }
}