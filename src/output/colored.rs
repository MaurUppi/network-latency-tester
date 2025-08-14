//! Colored formatter implementation with terminal color support
//!
//! This module provides a rich colored output formatter that uses
//! ANSI colors and Unicode symbols for enhanced visual presentation.

use crate::{
    error::{AppError, Result},
    executor::{ExecutionResults, ExecutionSummary},
    models::metrics::TestResult,
    stats::StatisticalAnalysis,
    diagnostics::DiagnosticReport,
};
use super::formatter::{OutputFormatter, FormattingOptions, PlainFormatter};
use std::collections::HashMap;
use std::fmt::Write as _;
use colored::*;

/// Performance level classification for color coding
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceLevel {
    Excellent,  // < 50ms
    Good,       // 50-100ms
    Fair,       // 100-300ms
    Poor,       // 300-1000ms
    VeryPoor,   // > 1000ms
}

impl PerformanceLevel {
    /// Determine performance level from response time in milliseconds
    pub fn from_response_time(time_ms: f64) -> Self {
        if time_ms < 50.0 {
            Self::Excellent
        } else if time_ms < 100.0 {
            Self::Good
        } else if time_ms < 300.0 {
            Self::Fair
        } else if time_ms < 1000.0 {
            Self::Poor
        } else {
            Self::VeryPoor
        }
    }

    /// Get color for this performance level
    pub fn color(&self) -> Color {
        match self {
            Self::Excellent => Color::Green,
            Self::Good => Color::Cyan,
            Self::Fair => Color::Yellow,
            Self::Poor => Color::Magenta,
            Self::VeryPoor => Color::Red,
        }
    }

    /// Get Unicode symbol for this performance level
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Excellent => "🚀",
            Self::Good => "⚡",
            Self::Fair => "🔶",
            Self::Poor => "⚠️",
            Self::VeryPoor => "🔴",
        }
    }

    /// Get descriptive text
    pub fn description(&self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Poor => "Poor",
            Self::VeryPoor => "Very Poor",
        }
    }
}

/// Color scheme configuration
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub header: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub highlight: Color,
    pub muted: Color,
    pub border: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            header: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,
            highlight: Color::Magenta,
            muted: Color::BrightBlack,
            border: Color::BrightBlack,
        }
    }
}

/// Colored formatter implementation
pub struct ColoredFormatter {
    #[allow(dead_code)]
    plain_formatter: PlainFormatter,
    options: FormattingOptions,
    color_scheme: ColorScheme,
}

impl ColoredFormatter {
    /// Create a new colored formatter with options
    pub fn new(options: FormattingOptions) -> Self {
        let plain_formatter = PlainFormatter::new(options.clone());
        Self {
            plain_formatter,
            options,
            color_scheme: ColorScheme::default(),
        }
    }

    /// Create a colored formatter with custom color scheme
    pub fn with_color_scheme(options: FormattingOptions, color_scheme: ColorScheme) -> Self {
        let plain_formatter = PlainFormatter::new(options.clone());
        Self {
            plain_formatter,
            options,
            color_scheme,
        }
    }

    /// Apply color to text if colors are enabled
    fn colorize(&self, text: &str, color: Color) -> ColoredString {
        if self.options.enable_color {
            text.color(color)
        } else {
            text.normal()
        }
    }

    /// Apply bold formatting if colors are enabled
    fn bold(&self, text: &str) -> ColoredString {
        if self.options.enable_color {
            text.bold()
        } else {
            text.normal()
        }
    }

    /// Apply dimmed formatting if colors are enabled
    fn dimmed(&self, text: &str) -> ColoredString {
        if self.options.enable_color {
            text.dimmed()
        } else {
            text.normal()
        }
    }

    /// Format duration with appropriate color coding
    fn format_duration_colored(&self, duration_ms: f64) -> ColoredString {
        let formatted = self.format_duration(duration_ms);
        let performance = PerformanceLevel::from_response_time(duration_ms);
        self.colorize(&formatted, performance.color())
    }

    /// Format percentage with color coding based on value
    fn format_percentage_colored(&self, percentage: f64) -> ColoredString {
        let formatted = self.format_percentage(percentage);
        let color = if percentage >= 95.0 {
            self.color_scheme.success
        } else if percentage >= 80.0 {
            self.color_scheme.warning
        } else {
            self.color_scheme.error
        };
        self.colorize(&formatted, color)
    }

    /// Format duration in human-readable format
    fn format_duration(&self, duration_ms: f64) -> String {
        if duration_ms < 1.0 {
            format!("{:.0}μs", duration_ms * 1000.0)
        } else if duration_ms < 1000.0 {
            format!("{:.0}ms", duration_ms)
        } else if duration_ms < 60000.0 {
            format!("{:.1}s", duration_ms / 1000.0)
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

    /// Create a colored performance bar
    fn create_performance_bar(&self, percentage: f64, width: usize) -> String {
        if !self.options.enable_color {
            return format!("[{}{}]", 
                "=".repeat((percentage * width as f64 / 100.0) as usize),
                " ".repeat(width - (percentage * width as f64 / 100.0) as usize));
        }

        let filled = (percentage * width as f64 / 100.0) as usize;
        let empty = width - filled;
        
        let bar_color = if percentage >= 95.0 {
            Color::Green
        } else if percentage >= 80.0 {
            Color::Yellow
        } else {
            Color::Red
        };

        format!("[{}{}]",
            "█".repeat(filled).color(bar_color),
            "░".repeat(empty).color(Color::BrightBlack))
    }

    /// Create a colored section header
    fn create_section_header(&self, title: &str, icon: &str) -> String {
        if self.options.enable_color {
            format!("{} {}", icon, title.bold().color(self.color_scheme.header))
        } else {
            format!("{} {}", icon, title)
        }
    }

    /// Create a colored table with enhanced formatting grouped by target URL
    fn create_colored_table(&self, results: &HashMap<String, TestResult>) -> Result<String> {
        if results.is_empty() {
            return Ok(self.colorize("No test results available.", self.color_scheme.muted).to_string());
        }

        let mut output = String::new();

        // Group results by URL
        let mut results_by_url: std::collections::HashMap<String, Vec<&TestResult>> = std::collections::HashMap::new();
        for result in results.values() {
            results_by_url.entry(result.url.clone()).or_default().push(result);
        }

        // Sort URLs for consistent output
        let mut sorted_urls: Vec<String> = results_by_url.keys().cloned().collect();
        sorted_urls.sort();

        let mut overall_rank = 0;

        for (url_index, url) in sorted_urls.iter().enumerate() {
            // Always add URL section header (for both single and multiple URLs)
            if url_index > 0 {
                writeln!(output)
                    .map_err(|e| AppError::io(format!("Failed to format table: {}", e)))?;
            }
            let url_display = if url.len() > 80 {
                format!("{}...", &url[..77])
            } else {
                url.clone()
            };
            writeln!(output, "🎯 Target: {}", self.bold(&url_display).color(self.color_scheme.info))
                .map_err(|e| AppError::io(format!("Failed to format table: {}", e)))?;
            writeln!(output, "{}", "─".repeat(95).color(self.color_scheme.border))
                .map_err(|e| AppError::io(format!("Failed to format table: {}", e)))?;

            // Header for each section
            let header = format!("{:<40} {:>12} {:>12} {:>15} {:>12}",
                "Configuration", "Success", "Avg Response", "Min/Max", "Level");
            
            writeln!(output, "{}", self.bold(&header))
                .map_err(|e| AppError::io(format!("Failed to format table: {}", e)))?;
                
            writeln!(output, "{}", "─".repeat(95).color(self.color_scheme.border))
                .map_err(|e| AppError::io(format!("Failed to format table: {}", e)))?;

            // Sort this URL's results by DNS type first, then by performance within each type
            let mut url_results = results_by_url[url].clone();
            url_results.sort_by(|a, b| {
                // Helper function to determine DNS type order
                let get_dns_type_order = |config_name: &str| -> u8 {
                    if config_name.contains("System DNS") {
                        0 // System DNS first
                    } else if config_name.contains("Custom DNS") {
                        1 // Custom DNS second  
                    } else if config_name.contains("DoH") {
                        2 // DoH third
                    } else {
                        3 // Unknown last
                    }
                };
                
                let a_type = get_dns_type_order(&a.config_name);
                let b_type = get_dns_type_order(&b.config_name);
                
                // First sort by DNS type
                match a_type.cmp(&b_type) {
                    std::cmp::Ordering::Equal => {
                        // If same DNS type, sort by performance ( the best first)
                        let a_time = a.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
                        let b_time = b.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(f64::MAX);
                        a_time.partial_cmp(&b_time).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    other => other
                }
            });

            // Data rows with colors
            for (index, result) in url_results.iter().enumerate() {
                overall_rank += 1;
                let rank_icon = match overall_rank {
                    1 => "🥇",
                    2 => "🥈", 
                    3 => "🥉",
                    _ => if results_by_url.len() > 1 {
                        // For multi-URL, show local ranking within the URL
                        match index {
                            0 => "🏆", // Best for this URL
                            _ => "  ",
                        }
                    } else {
                        "  "
                    }
                };

                let config_name = format!("{} {}", rank_icon, result.config_name);
                let config_display = if config_name.len() > 38 {
                    format!("{}...", &config_name[..35])
                } else {
                    config_name
                };

                let success_rate = result.success_rate();
                let success_display = format!("{} {}", 
                    self.format_percentage_colored(success_rate),
                    self.create_performance_bar(success_rate, 8));

                let (avg_response, min_max, performance_level) = if let Some(ref stats) = result.statistics {
                    let avg_colored = self.format_duration_colored(stats.total_avg_ms);
                    let min_max = format!("{}/{}", 
                        self.format_duration(stats.total_min_ms), 
                        self.format_duration(stats.total_max_ms));
                    let perf_level = PerformanceLevel::from_response_time(stats.total_avg_ms);
                    let perf_display = format!("{} {}", perf_level.symbol(), perf_level.description());
                    
                    (avg_colored.to_string(), min_max, 
                     self.colorize(&perf_display, perf_level.color()).to_string())
                } else {
                    (self.dimmed("N/A").to_string(), self.dimmed("N/A").to_string(), self.dimmed("Unknown").to_string())
                };

                writeln!(output, "{:<40} {:>20} {:>12} {:>15} {:>20}",
                    config_display, success_display, avg_response, min_max, performance_level)
                    .map_err(|e| AppError::io(format!("Failed to format table: {}", e)))?;
            }
        }

        Ok(output)
    }
}

impl OutputFormatter for ColoredFormatter {
    fn format_header(&self, title: &str) -> Result<String> {
        let mut output = String::new();
        
        let decorated_title = format!("🎯 {}", title);
        let border = "═".repeat(decorated_title.len() + 4);
        
        writeln!(output, "{}", self.colorize(&border, self.color_scheme.border))
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        writeln!(output, "  {}  ", self.bold(&decorated_title).color(self.color_scheme.header))
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        write!(output, "{}", self.colorize(&border, self.color_scheme.border))
            .map_err(|e| AppError::io(format!("Failed to format header: {}", e)))?;
        
        Ok(output)
    }

    fn format_execution_summary(&self, summary: &ExecutionSummary) -> Result<String> {
        let mut output = String::new();
        
        writeln!(output, "{}", self.create_section_header("Execution Summary", "📊"))
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        
        let duration_colored = self.format_duration_colored(summary.total_duration.as_secs_f64() * 1000.0);
        let success_colored = self.format_percentage_colored(summary.success_rate);
        
        writeln!(output, "⏱️  Duration:     {}", duration_colored)
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        writeln!(output, "🧪 Total Tests:  {}", self.colorize(&summary.total_tests.to_string(), self.color_scheme.info))
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        writeln!(output, "✅ Successful:   {} ({})", 
            self.colorize(&summary.successful_tests.to_string(), self.color_scheme.success), 
            success_colored)
            .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        
        if summary.failed_tests > 0 {
            writeln!(output, "❌ Failed:       {}", 
                self.colorize(&summary.failed_tests.to_string(), self.color_scheme.error))
                .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        }
        
        if summary.timeout_tests > 0 {
            writeln!(output, "⏰ Timeout:      {}", 
                self.colorize(&summary.timeout_tests.to_string(), self.color_scheme.warning))
                .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        }
        
        if summary.skipped_tests > 0 {
            write!(output, "⏭️  Skipped:      {}", 
                self.dimmed(&summary.skipped_tests.to_string()))
                .map_err(|e| AppError::io(format!("Failed to format summary: {}", e)))?;
        }
        
        Ok(output)
    }

    fn format_performance_table(&self, results: &HashMap<String, TestResult>) -> Result<String> {
        let mut output = String::new();
        
        writeln!(output, "{}", self.create_section_header("Performance Results", "🚀"))
            .map_err(|e| AppError::io(format!("Failed to format table: {}", e)))?;
        writeln!(output)
            .map_err(|e| AppError::io(format!("Failed to format table: {}", e)))?;
        
        output.push_str(&self.create_colored_table(results)?);
        
        Ok(output)
    }

    fn format_statistical_analysis(&self, analysis: &StatisticalAnalysis) -> Result<String> {
        let mut output = String::new();
        
        writeln!(output, "{}", self.create_section_header("Statistical Analysis", "📈"))
            .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
        
        if let Some(ref recommended) = analysis.summary.recommended_config {
            writeln!(output, "🏆 Recommended:   {}", 
                self.bold(recommended).color(self.color_scheme.highlight))
                .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
        }
        
        // Calculate overall metrics from basic stats
        let total_success_rate = if !analysis.basic_stats.is_empty() {
            analysis.basic_stats.values()
                .map(|stats| stats.reliability.success_rate)
                .sum::<f64>() / analysis.basic_stats.len() as f64
        } else {
            0.0
        };
        let avg_response_time = if !analysis.basic_stats.is_empty() {
            analysis.basic_stats.values()
                .map(|stats| stats.basic.total_avg_ms)
                .sum::<f64>() / analysis.basic_stats.len() as f64
        } else {
            0.0
        };
        
        writeln!(output, "📊 Success Rate:  {}", 
            self.format_percentage_colored(total_success_rate))
            .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
        writeln!(output, "⚡ Avg Response:  {}", 
            self.format_duration_colored(avg_response_time))
            .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
        
        if self.options.verbose_mode && !analysis.basic_stats.is_empty() {
            writeln!(output, "\n{}", self.dimmed("Detailed Analysis:"))
                .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
            for (config_name, config_stats) in &analysis.basic_stats {
                let truncated_name = if config_name.len() > 25 {
                    format!("{}...", &config_name[..22])
                } else {
                    config_name.clone()
                };
                writeln!(output, "  📋 {}:", self.colorize(&truncated_name, self.color_scheme.info))
                    .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
                writeln!(output, "     Success: {} | Response: {} ± {}", 
                    self.format_percentage_colored(config_stats.reliability.success_rate),
                    self.format_duration_colored(config_stats.basic.total_avg_ms),
                    self.format_duration(config_stats.basic.total_std_dev_ms))
                    .map_err(|e| AppError::io(format!("Failed to format analysis: {}", e)))?;
            }
        }
        
        Ok(output)
    }

    fn format_diagnostics_report(&self, report: &DiagnosticReport) -> Result<String> {
        let mut output = String::new();
        
        writeln!(output, "{}", self.create_section_header("Network Diagnostics", "🔧"))
            .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
        
        match report.system_health.status {
            crate::diagnostics::HealthStatus::Healthy => {
                writeln!(output, "🟢 Network Health: {}", 
                    self.colorize("EXCELLENT", self.color_scheme.success))
                    .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
            }
            _ => {
                writeln!(output, "🔴 Network Health: {}", 
                    self.colorize("ISSUES DETECTED", self.color_scheme.error))
                    .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
                
                for issue in &report.system_health.critical_issues {
                    writeln!(output, "   🚨 {}", self.colorize(&format!("CRITICAL: {}", issue), self.color_scheme.error))
                        .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
                }
                
                for issue in &report.system_health.warning_issues {
                    writeln!(output, "   ⚠️  {}", self.colorize(&format!("WARNING: {}", issue), self.color_scheme.warning))
                        .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
                }
            }
        }
        
        if self.options.verbose_mode && !report.connectivity_diagnostics.target_reachability.is_empty() {
            writeln!(output, "\n{}", self.dimmed("Target Reachability:"))
                .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
            for (target, status) in &report.connectivity_diagnostics.target_reachability {
                let (icon, status_text) = if status.reachable {
                    if let Some(time) = status.response_time {
                        ("✅", format!("Connected ({})", self.format_duration(time.as_secs_f64() * 1000.0)))
                    } else {
                        ("✅", "Connected".to_string())
                    }
                } else if let Some(ref error) = status.error_message {
                    ("❌", format!("Failed: {}", error))
                } else {
                    ("❌", "Disconnected".to_string())
                };
                let truncated_target = if target.len() > 40 {
                    format!("{}...", &target[..37])
                } else {
                    target.clone()
                };
                writeln!(output, "  {} {} - {}", icon, truncated_target, status_text)
                    .map_err(|e| AppError::io(format!("Failed to format diagnostics: {}", e)))?;
            }
        }
        
        Ok(output)
    }

    fn format_recommendations(&self, results: &ExecutionResults) -> Result<String> {
        let mut output = String::new();
        
        writeln!(output, "{}", self.create_section_header("Recommendations", "💡"))
            .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        
        if let Some(best) = results.best_config() {
            writeln!(output, "🎯 Use {} for optimal performance", 
                self.bold(best).color(self.color_scheme.highlight))
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        }
        
        if results.execution_summary.success_rate < 95.0 {
            writeln!(output, "⚠️  Success rate below 95% - {}", 
                self.colorize("investigate network issues", self.color_scheme.warning))
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        }
        
        // Performance analysis based on actual test results
        let mut fast_configs = 0;
        let mut slow_configs = 0;
        let mut total_configs = 0;
        
        for result in results.test_results.values() {
            if let Some(ref stats) = result.statistics {
                total_configs += 1;
                if stats.total_avg_ms < 100.0 {
                    fast_configs += 1;
                } else if stats.total_avg_ms > 500.0 {
                    slow_configs += 1;
                }
            }
        }
        
        if total_configs == 0 {
            // No valid statistics available
        } else if fast_configs == 0 && slow_configs > total_configs / 2 {
            writeln!(output, "🐌 Most configurations are slow - {}", 
                self.colorize("check network conditions", self.color_scheme.warning))
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        } else if slow_configs > fast_configs {
            writeln!(output, "⚡ Consider faster DNS providers for better performance")
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        } else {
            writeln!(output, "✨ Network performance looks good!")
                .map_err(|e| AppError::io(format!("Failed to format recommendations: {}", e)))?;
        }
        
        Ok(output)
    }

    fn format_quick_summary(&self, results: &ExecutionResults) -> Result<String> {
        let success_colored = self.format_percentage_colored(results.execution_summary.success_rate);
        let duration_colored = self.format_duration_colored(results.execution_summary.total_duration.as_secs_f64() * 1000.0);
        let best_config = results.best_config().unwrap_or("Unknown");

        Ok(format!(
            "📊 {}/{} {} | 🏆 {} | ⏱️ {}",
            self.colorize(&results.execution_summary.successful_tests.to_string(), self.color_scheme.success),
            results.execution_summary.total_tests,
            success_colored,
            self.colorize(best_config, self.color_scheme.highlight),
            duration_colored
        ))
    }

    fn format_test_result(&self, result: &TestResult) -> Result<String> {
        let success_colored = self.format_percentage_colored(result.success_rate());
        let avg_response = if let Some(ref stats) = result.statistics {
            self.format_duration_colored(stats.total_avg_ms).to_string()
        } else {
            self.dimmed("N/A").to_string()
        };

        Ok(format!(
            "🧪 {}: {} success, {} avg",
            self.colorize(&result.config_name, self.color_scheme.info),
            success_colored,
            avg_response
        ))
    }

    fn format_error(&self, error: &str) -> Result<String> {
        Ok(format!("❌ {}", self.colorize(error, self.color_scheme.error)))
    }

    fn format_warning(&self, warning: &str) -> Result<String> {
        Ok(format!("⚠️  {}", self.colorize(warning, self.color_scheme.warning)))
    }

    fn format_success(&self, message: &str) -> Result<String> {
        Ok(format!("✅ {}", self.colorize(message, self.color_scheme.success)))
    }
}

/// Helper functions for color management
impl ColoredFormatter {
    /// Check if terminal supports colors
    pub fn supports_color() -> bool {
        std::env::var("NO_COLOR").is_err() && 
        std::env::var("TERM").map(|term| term != "dumb").unwrap_or(true)
    }

    /// Enable or disable colors at runtime
    pub fn set_colors_enabled(&mut self, enabled: bool) {
        self.options.enable_color = enabled && Self::supports_color();
    }
}