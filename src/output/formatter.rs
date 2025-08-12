//! Core formatting traits and implementations
//!
//! This module defines the output formatting interface and provides
//! a plain text implementation with table formatting capabilities.

use crate::{
    error::{AppError, Result},
    executor::{ExecutionResults, ExecutionSummary},
    models::metrics::TestResult,
    stats::StatisticalAnalysis,
    diagnostics::DiagnosticReport,
};
use std::collections::HashMap;
use std::fmt::Write as _;

/// Main trait for output formatting
pub trait OutputFormatter {
    /// Format a header section
    fn format_header(&self, title: &str) -> Result<String>;
    
    /// Format execution summary
    fn format_execution_summary(&self, summary: &ExecutionSummary) -> Result<String>;
    
    /// Format performance results as a table
    fn format_performance_table(&self, results: &HashMap<String, TestResult>) -> Result<String>;
    
    /// Format statistical analysis
    fn format_statistical_analysis(&self, analysis: &StatisticalAnalysis) -> Result<String>;
    
    /// Format diagnostics report
    fn format_diagnostics_report(&self, report: &DiagnosticReport) -> Result<String>;
    
    /// Format recommendations
    fn format_recommendations(&self, results: &ExecutionResults) -> Result<String>;
    
    /// Format a quick summary for progress updates
    fn format_quick_summary(&self, results: &ExecutionResults) -> Result<String>;
    
    /// Format individual test result
    fn format_test_result(&self, result: &TestResult) -> Result<String>;
    
    /// Format error messages
    fn format_error(&self, error: &str) -> Result<String>;
    
    /// Format warning messages
    fn format_warning(&self, warning: &str) -> Result<String>;
    
    /// Format success messages
    fn format_success(&self, message: &str) -> Result<String>;
}

/// Configuration options for formatting
#[derive(Debug, Clone)]
pub struct FormattingOptions {
    /// Enable colored output
    pub enable_color: bool,
    /// Enable verbose mode with detailed information
    pub verbose_mode: bool,
    /// Show individual test results
    pub show_individual_results: bool,
    /// Show table borders
    pub table_borders: bool,
    /// Maximum output width
    pub max_width: usize,
    /// Enable compact output mode
    pub compact_mode: bool,
}

impl Default for FormattingOptions {
    fn default() -> Self {
        Self {
            enable_color: true,
            verbose_mode: false,
            show_individual_results: false,
            table_borders: true,
            max_width: 120,
            compact_mode: false,
        }
    }
}

/// Table formatting configuration
#[derive(Debug, Clone)]
pub struct TableFormat {
    /// Column definitions
    pub columns: Vec<Column>,
    /// Show borders around table
    pub show_borders: bool,
    /// Show header row
    pub show_header: bool,
    /// Minimum column width
    pub min_column_width: usize,
    /// Maximum column width
    pub max_column_width: usize,
    /// Cell padding
    pub padding: usize,
}

/// Column definition for table formatting
#[derive(Debug, Clone)]
pub struct Column {
    /// Column header
    pub header: String,
    /// Column alignment
    pub alignment: Alignment,
    /// Minimum width
    pub min_width: usize,
    /// Maximum width
    pub max_width: usize,
    /// Whether column is flexible in width
    pub flexible: bool,
}

/// Text alignment options
#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

/// Row data for table formatting
pub type RowData = Vec<String>;

/// Plain text formatter implementation
pub struct PlainFormatter {
    options: FormattingOptions,
}

impl PlainFormatter {
    /// Create a new plain formatter with options
    pub fn new(options: FormattingOptions) -> Self {
        Self { options }
    }

    /// Create a table with the given format and data
    fn create_table(&self, format: &TableFormat, rows: &[RowData]) -> Result<String> {
        if rows.is_empty() {
            return Ok(String::new());
        }

        // Calculate column widths
        let column_widths = self.calculate_column_widths(&format, rows)?;
        
        let mut output = String::new();
        
        // Header
        if format.show_header && !format.columns.is_empty() {
            if format.show_borders {
                output.push_str(&self.create_horizontal_border(&column_widths));
                output.push('\n');
            }
            
            let headers: Vec<String> = format.columns.iter().map(|c| c.header.clone()).collect();
            output.push_str(&self.create_row(&headers, &column_widths, &format));
            output.push('\n');
            
            if format.show_borders {
                output.push_str(&self.create_horizontal_border(&column_widths));
                output.push('\n');
            }
        }
        
        // Data rows
        for row in rows {
            output.push_str(&self.create_row(row, &column_widths, &format));
            output.push('\n');
        }
        
        // Bottom border
        if format.show_borders {
            output.push_str(&self.create_horizontal_border(&column_widths));
        }
        
        Ok(output)
    }

    /// Calculate optimal column widths
    fn calculate_column_widths(&self, format: &TableFormat, rows: &[RowData]) -> Result<Vec<usize>> {
        let mut widths = Vec::new();
        let num_columns = format.columns.len().max(
            rows.iter().map(|r| r.len()).max().unwrap_or(0)
        );

        for col_idx in 0..num_columns {
            let mut max_width = if col_idx < format.columns.len() {
                format.columns[col_idx].min_width.max(format.columns[col_idx].header.len())
            } else {
                format.min_column_width
            };

            // Find maximum content width in this column
            for row in rows {
                if col_idx < row.len() {
                    max_width = max_width.max(row[col_idx].len());
                }
            }

            // Apply column constraints
            if col_idx < format.columns.len() {
                let col = &format.columns[col_idx];
                max_width = max_width.min(col.max_width);
            } else {
                max_width = max_width.min(format.max_column_width);
            }

            widths.push(max_width);
        }

        Ok(widths)
    }

    /// Create a table row
    fn create_row(&self, data: &[String], widths: &[usize], format: &TableFormat) -> String {
        let mut row = String::new();
        
        if format.show_borders {
            row.push('|');
        }
        
        for (idx, (cell, &width)) in data.iter().zip(widths.iter()).enumerate() {
            let alignment = if idx < format.columns.len() {
                &format.columns[idx].alignment
            } else {
                &Alignment::Left
            };
            
            let padded_cell = self.align_text(cell, width, alignment);
            
            if format.show_borders {
                row.push(' ');
            }
            row.push_str(&padded_cell);
            if format.show_borders {
                row.push(' ');
                row.push('|');
            } else {
                row.push_str("  ");
            }
        }
        
        row.trim_end().to_string()
    }

    /// Create horizontal border for table
    fn create_horizontal_border(&self, widths: &[usize]) -> String {
        let mut border = String::new();
        
        if !widths.is_empty() {
            border.push('+');
            for &width in widths {
                border.push_str(&"-".repeat(width + 2));
                border.push('+');
            }
        }
        
        border
    }

    /// Align text within specified width
    fn align_text(&self, text: &str, width: usize, alignment: &Alignment) -> String {
        if text.len() >= width {
            return text.chars().take(width).collect();
        }

        let padding = width - text.len();
        match alignment {
            Alignment::Left => format!("{}{}", text, " ".repeat(padding)),
            Alignment::Right => format!("{}{}", " ".repeat(padding), text),
            Alignment::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
            }
        }
    }

    /// Format duration in human-readable format
    fn format_duration(&self, duration_ms: f64) -> String {
        if duration_ms < 1.0 {
            format!("{:.2}μs", duration_ms * 1000.0)
        } else if duration_ms < 1000.0 {
            format!("{:.1}ms", duration_ms)
        } else if duration_ms < 60000.0 {
            format!("{:.2}s", duration_ms / 1000.0)
        } else {
            let minutes = (duration_ms / 60000.0) as u32;
            let seconds = (duration_ms % 60000.0) / 1000.0;
            format!("{}m{:.1}s", minutes, seconds)
        }
    }

    /// Format percentage with appropriate precision
    fn format_percentage(&self, percentage: f64) -> String {
        if percentage >= 99.95 {
            "100.0%".to_string()
        } else if percentage < 0.05 {
            "0.0%".to_string()
        } else {
            format!("{:.1}%", percentage)
        }
    }
}

impl OutputFormatter for PlainFormatter {
    fn format_header(&self, title: &str) -> Result<String> {
        let mut output = String::new();
        let border = "=".repeat(title.len() + 4);
        
        writeln!(output, "{}", border)
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        writeln!(output, "  {}  ", title)
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        write!(output, "{}", border)
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        
        Ok(output)
    }

    fn format_execution_summary(&self, summary: &ExecutionSummary) -> Result<String> {
        let mut output = String::new();
        
        writeln!(output, "Execution Summary:")
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        writeln!(output, "-----------------")
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        writeln!(output, "Total Duration:   {}", self.format_duration(summary.total_duration.as_secs_f64() * 1000.0))
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        writeln!(output, "Total Tests:      {}", summary.total_tests)
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        writeln!(output, "Successful:       {} ({})", summary.successful_tests, self.format_percentage(summary.success_rate))
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        writeln!(output, "Failed:           {}", summary.failed_tests)
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        writeln!(output, "Timeout:          {}", summary.timeout_tests)
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        write!(output, "Skipped:          {}", summary.skipped_tests)
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        
        Ok(output)
    }

    fn format_performance_table(&self, results: &HashMap<String, TestResult>) -> Result<String> {
        if results.is_empty() {
            return Ok("No test results available.".to_string());
        }

        let table_format = TableFormat {
            columns: vec![
                Column {
                    header: "Configuration".to_string(),
                    alignment: Alignment::Left,
                    min_width: 15,
                    max_width: 40,
                    flexible: true,
                },
                Column {
                    header: "Success Rate".to_string(),
                    alignment: Alignment::Right,
                    min_width: 12,
                    max_width: 12,
                    flexible: false,
                },
                Column {
                    header: "Avg Response".to_string(),
                    alignment: Alignment::Right,
                    min_width: 12,
                    max_width: 12,
                    flexible: false,
                },
                Column {
                    header: "Min/Max".to_string(),
                    alignment: Alignment::Right,
                    min_width: 15,
                    max_width: 15,
                    flexible: false,
                },
                Column {
                    header: "Performance".to_string(),
                    alignment: Alignment::Center,
                    min_width: 12,
                    max_width: 12,
                    flexible: false,
                },
            ],
            show_borders: self.options.table_borders,
            show_header: true,
            min_column_width: 8,
            max_column_width: 50,
            padding: 1,
        };

        let mut rows = Vec::new();
        for result in results.values() {
            let success_rate = self.format_percentage(result.success_rate());
            let avg_response = if let Some(ref stats) = result.statistics {
                self.format_duration(stats.total_avg_ms)
            } else {
                "N/A".to_string()
            };
            
            let min_max = if let Some(ref stats) = result.statistics {
                format!("{}/{}", 
                    self.format_duration(stats.total_min_ms), 
                    self.format_duration(stats.total_max_ms))
            } else {
                "N/A".to_string()
            };
            
            let performance = result.performance_level()
                .map(|p| format!("{:?}", p))
                .unwrap_or_else(|| "Unknown".to_string());

            rows.push(vec![
                result.config_name.clone(),
                success_rate,
                avg_response,
                min_max,
                performance,
            ]);
        }

        // Sort by average response time (fastest first)
        rows.sort_by(|a, b| {
            let a_time = results.get(&a[0]).and_then(|r| r.statistics.as_ref()).map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
            let b_time = results.get(&b[0]).and_then(|r| r.statistics.as_ref()).map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
            a_time.partial_cmp(&b_time).unwrap_or(std::cmp::Ordering::Equal)
        });

        self.create_table(&table_format, &rows)
    }

    fn format_statistical_analysis(&self, analysis: &StatisticalAnalysis) -> Result<String> {
        let mut output = String::new();
        
        writeln!(output, "Statistical Analysis:")
            .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
        writeln!(output, "--------------------")
            .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
        
        if let Some(ref recommended) = analysis.summary.recommended_config {
            writeln!(output, "Recommended Configuration: {}", recommended)
                .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
        }
        
        // Calculate overall metrics from basic stats
        let total_success_rate = analysis.basic_stats.values()
            .map(|stats| stats.reliability.success_rate)
            .sum::<f64>() / analysis.basic_stats.len() as f64;
        let avg_response_time = analysis.basic_stats.values()
            .map(|stats| stats.basic.total_avg_ms)
            .sum::<f64>() / analysis.basic_stats.len() as f64;
        
        writeln!(output, "Overall Success Rate:      {}", self.format_percentage(total_success_rate))
            .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
        writeln!(output, "Average Response Time:     {}", self.format_duration(avg_response_time))
            .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
        
        if self.options.verbose_mode {
            writeln!(output, "\nDetailed Statistics:")
                .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
            for (config_name, config_stats) in &analysis.basic_stats {
                writeln!(output, "  {}:", config_name)
                    .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
                writeln!(output, "    Success Rate: {}", self.format_percentage(config_stats.reliability.success_rate))
                    .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
                writeln!(output, "    Response Time: {} ± {}", 
                    self.format_duration(config_stats.basic.total_avg_ms), 
                    self.format_duration(config_stats.basic.total_std_dev_ms))
                    .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
            }
        }
        
        Ok(output)
    }

    fn format_diagnostics_report(&self, report: &DiagnosticReport) -> Result<String> {
        let mut output = String::new();
        
        writeln!(output, "Network Diagnostics:")
            .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
        writeln!(output, "-------------------")
            .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
        
        match report.system_health.status {
            crate::diagnostics::HealthStatus::Healthy => {
                writeln!(output, "Overall Network Health: GOOD")
                    .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
            }
            _ => {
                writeln!(output, "Overall Network Health: ISSUES DETECTED")
                    .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
                
                for issue in &report.system_health.critical_issues {
                    writeln!(output, "  - CRITICAL: {}", issue)
                        .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
                }
                
                for issue in &report.system_health.warning_issues {
                    writeln!(output, "  - WARNING: {}", issue)
                        .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
                }
            }
        }
        
        if self.options.verbose_mode && !report.connectivity_diagnostics.target_reachability.is_empty() {
            writeln!(output, "\nTarget Reachability:")
                .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
            for (target, status) in &report.connectivity_diagnostics.target_reachability {
                let status_str = if status.reachable {
                    if let Some(time) = status.response_time {
                        format!("✓ Connected ({})", self.format_duration(time.as_secs_f64() * 1000.0))
                    } else {
                        "✓ Connected".to_string()
                    }
                } else {
                    if let Some(ref error) = status.error_message {
                        format!("✗ Failed: {}", error)
                    } else {
                        "✗ Disconnected".to_string()
                    }
                };
                writeln!(output, "  {} - {}", target, status_str)
                    .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
            }
        }
        
        Ok(output)
    }

    fn format_recommendations(&self, results: &ExecutionResults) -> Result<String> {
        let mut output = String::new();
        
        writeln!(output, "Recommendations:")
            .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        writeln!(output, "---------------")
            .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        
        if let Some(best) = results.best_config() {
            writeln!(output, "• Use '{}' for optimal performance", best)
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        }
        
        if results.execution_summary.success_rate < 95.0 {
            writeln!(output, "• Success rate is below 95% - consider network troubleshooting")
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        }
        
        // Performance-based recommendations
        let mut fast_configs = 0;
        let mut slow_configs = 0;
        for perf in results.execution_summary.performance_summary.values() {
            if perf.avg_response_time < 100.0 {
                fast_configs += 1;
            } else if perf.avg_response_time > 500.0 {
                slow_configs += 1;
            }
        }
        
        if fast_configs == 0 {
            writeln!(output, "• All configurations are responding slowly - check network conditions")
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        } else if slow_configs > fast_configs {
            writeln!(output, "• Consider using faster DNS providers for better performance")
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        }
        
        Ok(output)
    }

    fn format_quick_summary(&self, results: &ExecutionResults) -> Result<String> {
        Ok(format!(
            "Tests: {}/{} successful ({:.1}%) | Best: {} | Duration: {:.2}s",
            results.execution_summary.successful_tests,
            results.execution_summary.total_tests,
            results.execution_summary.success_rate,
            results.best_config().unwrap_or("Unknown"),
            results.execution_summary.total_duration.as_secs_f64()
        ))
    }

    fn format_test_result(&self, result: &TestResult) -> Result<String> {
        let success_rate = self.format_percentage(result.success_rate());
        let avg_response = if let Some(ref stats) = result.statistics {
            self.format_duration(stats.total_avg_ms)
        } else {
            "N/A".to_string()
        };

        Ok(format!(
            "{}: {} success, {} avg response",
            result.config_name,
            success_rate,
            avg_response
        ))
    }

    fn format_error(&self, error: &str) -> Result<String> {
        Ok(format!("ERROR: {}", error))
    }

    fn format_warning(&self, warning: &str) -> Result<String> {
        Ok(format!("WARNING: {}", warning))
    }

    fn format_success(&self, message: &str) -> Result<String> {
        Ok(format!("SUCCESS: {}", message))
    }
}