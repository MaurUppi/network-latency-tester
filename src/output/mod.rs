//! Output formatting and display system
//!
//! This module provides a flexible output formatting system for test results,
//! supporting both colored and plain text output with table formatting.

mod formatter;
mod colored;
mod verbose;

pub use formatter::{
    OutputFormatter, 
    PlainFormatter,
    TableFormat,
    FormattingOptions,
    Column,
    RowData,
};
pub use colored::{
    ColoredFormatter,
    ColorScheme,
    PerformanceLevel,
};
pub use verbose::VerboseTimingFormatter;

use crate::{
    error::Result,
    executor::ExecutionResults,
    models::metrics::TestResult,
};

/// Output formatting factory for creating appropriate formatters
pub struct OutputFormatterFactory;

impl OutputFormatterFactory {
    /// Create a formatter based on color support and preferences
    pub fn create_formatter(enable_color: bool, verbose: bool) -> Box<dyn OutputFormatter> {
        let options = FormattingOptions {
            enable_color,
            verbose_mode: verbose,
            show_individual_results: verbose,
            table_borders: true,
            max_width: 120,
            compact_mode: !verbose,
        };

        if enable_color {
            Box::new(ColoredFormatter::new(options))
        } else {
            Box::new(PlainFormatter::new(options))
        }
    }

    /// Create a console-optimized formatter
    pub fn create_console_formatter() -> Box<dyn OutputFormatter> {
        Self::create_formatter(true, false)
    }

    /// Create a plain text formatter for scripts/logs
    pub fn create_plain_formatter() -> Box<dyn OutputFormatter> {
        Self::create_formatter(false, true)
    }
    
    /// Create a verbose timing formatter for detailed timing analysis
    pub fn create_verbose_timing_formatter(config: &crate::models::Config) -> VerboseTimingFormatter {
        VerboseTimingFormatter::new(config)
    }
}

/// Main output coordinator that handles all result display
pub struct OutputCoordinator {
    formatter: Box<dyn OutputFormatter>,
    verbose_formatter: Option<VerboseTimingFormatter>,
    config: Option<crate::models::Config>,
}

impl OutputCoordinator {
    /// Create a new output coordinator with the specified formatter
    pub fn new(formatter: Box<dyn OutputFormatter>) -> Self {
        Self { 
            formatter,
            verbose_formatter: None,
            config: None,
        }
    }
    
    /// Create a new output coordinator with verbose timing support
    pub fn with_verbose_timing(formatter: Box<dyn OutputFormatter>, config: &crate::models::Config) -> Self {
        let verbose_formatter = if config.verbose {
            Some(VerboseTimingFormatter::new(config))
        } else {
            None
        };
        
        Self {
            formatter,
            verbose_formatter,
            config: Some(config.clone()),
        }
    }

    /// Display complete execution results
    pub async fn display_results(&self, results: &ExecutionResults) -> Result<String> {
        let mut output = String::new();

        // Check if we should use verbose timing output
        if let (Some(verbose_formatter), Some(config)) = (&self.verbose_formatter, &self.config) {
            if config.verbose {
                // Use comprehensive verbose timing output
                return verbose_formatter.format_verbose_results(results).await;
            }
        }

        // Standard output formatting
        // Header
        output.push_str(&self.formatter.format_header("Network Latency Test Results")?);
        output.push_str("\n\n");

        // Execution summary
        output.push_str(&self.formatter.format_execution_summary(&results.execution_summary)?);
        output.push_str("\n\n");

        // Performance table
        output.push_str(&self.formatter.format_performance_table(&results.test_results)?);
        output.push_str("\n\n");

        // Statistical analysis
        if let Some(ref analysis) = results.statistical_analysis {
            output.push_str(&self.formatter.format_statistical_analysis(analysis)?);
            output.push_str("\n\n");
        }

        // Diagnostics report
        if let Some(ref diagnostics) = results.diagnostics_report {
            output.push_str(&self.formatter.format_diagnostics_report(diagnostics)?);
            output.push_str("\n\n");
        }

        // Recommendations
        output.push_str(&self.formatter.format_recommendations(results)?);

        Ok(output)
    }

    /// Display a quick summary for progress updates
    pub async fn display_quick_summary(&self, results: &ExecutionResults) -> Result<String> {
        // Use verbose timing summary if available
        if let Some(verbose_formatter) = &self.verbose_formatter {
            verbose_formatter.format_console_timing_summary(results).await
        } else {
            self.formatter.format_quick_summary(results)
        }
    }

    /// Display individual test result during execution
    pub fn display_test_result(&self, result: &TestResult) -> Result<String> {
        self.formatter.format_test_result(result)
    }
    
    /// Display verbose timing information for a specific configuration
    pub async fn display_verbose_config_timing(&self, config_name: &str, result: &TestResult) -> Result<String> {
        if let Some(_verbose_formatter) = &self.verbose_formatter {
            // This would be a new method we'd add to VerboseTimingFormatter
            Ok(format!("Verbose timing for {}: {} tests completed", config_name, result.total_count))
        } else {
            self.formatter.format_test_result(result)
        }
    }
}