//! Structured logging system for network latency tester
//!
//! This module provides comprehensive logging functionality including:
//! - Structured logging with multiple levels and contexts
//! - Debug mode detailed tracing
//! - Performance timing logging
//! - Error event logging with correlation IDs
//! - JSON structured output for integration with log aggregators

use crate::error::{AppError, Result};
use crate::models::{Config, TimingMetrics, TestResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level - most detailed
    Trace = 0,
    /// Debug level - detailed information for debugging
    Debug = 1,
    /// Info level - general application information
    Info = 2,
    /// Warning level - potentially harmful situations
    Warn = 3,
    /// Error level - error events but application can continue
    Error = 4,
    /// Fatal level - severe error events that cause application termination
    Fatal = 5,
}

impl LogLevel {
    /// Get log level name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
    
    /// Get ANSI color code for console output
    pub fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Trace => "\x1b[37m",    // White
            LogLevel::Debug => "\x1b[36m",    // Cyan
            LogLevel::Info => "\x1b[32m",     // Green
            LogLevel::Warn => "\x1b[33m",     // Yellow
            LogLevel::Error => "\x1b[31m",    // Red
            LogLevel::Fatal => "\x1b[35m",    // Magenta
        }
    }
    
    /// Reset ANSI color code
    pub fn reset_code() -> &'static str {
        "\x1b[0m"
    }
}

impl std::str::FromStr for LogLevel {
    type Err = AppError;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Ok(LogLevel::Trace),
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARN" | "WARNING" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            "FATAL" => Ok(LogLevel::Fatal),
            _ => Err(AppError::parse(format!("Invalid log level: {}", s))),
        }
    }
}

/// Log entry structure for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp when log entry was created
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Logger name/component
    pub logger: String,
    /// Correlation ID for tracking related events
    pub correlation_id: Option<String>,
    /// Additional structured fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Thread ID if available
    pub thread_id: Option<String>,
    /// File and line information
    pub location: Option<LogLocation>,
}

/// Source code location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLocation {
    /// Source file name
    pub file: String,
    /// Line number
    pub line: u32,
    /// Module path
    pub module: Option<String>,
}

/// Logger implementation with multiple output formats
pub struct Logger {
    /// Minimum log level to output
    min_level: LogLevel,
    /// Whether to use colored output
    use_color: bool,
    /// Whether to include location information
    include_location: bool,
    /// Output format
    format: LogFormat,
    /// Logger name
    name: String,
    /// Shared context storage
    context: Arc<RwLock<LogContext>>,
}

/// Log output format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogFormat {
    /// Human-readable console format
    Console,
    /// JSON format for structured logging
    Json,
    /// Compact single-line format
    Compact,
}

/// Shared logging context for correlation and session tracking
#[derive(Debug, Default)]
struct LogContext {
    /// Global correlation ID for the session
    session_id: Option<String>,
    /// Current operation correlation ID
    current_correlation_id: Option<String>,
    /// Additional context fields
    context_fields: HashMap<String, serde_json::Value>,
}

/// Performance timing logger for detailed execution tracking
pub struct PerformanceLogger {
    logger: Logger,
    start_times: HashMap<String, DateTime<Utc>>,
    operation_stack: Vec<String>,
}

/// Specialized logger for network operations
pub struct NetworkLogger {
    logger: Logger,
}

/// Error event logger with enhanced context
pub struct ErrorEventLogger {
    logger: Logger,
}

impl Logger {
    /// Create a new logger
    pub fn new(name: String) -> Self {
        Self {
            min_level: LogLevel::Info,
            use_color: true,
            include_location: false,
            format: LogFormat::Console,
            name,
            context: Arc::new(RwLock::new(LogContext::default())),
        }
    }
    
    /// Create a logger with specific configuration
    pub fn with_config(name: String, config: &Config) -> Self {
        let min_level = if config.debug {
            LogLevel::Debug
        } else if config.verbose {
            LogLevel::Info
        } else {
            LogLevel::Warn
        };
        
        Self {
            min_level,
            use_color: config.enable_color,
            include_location: config.debug,
            format: if config.debug { LogFormat::Json } else { LogFormat::Console },
            name,
            context: Arc::new(RwLock::new(LogContext::default())),
        }
    }
    
    /// Set minimum log level
    pub fn set_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }
    
    /// Set output format
    pub fn set_format(&mut self, format: LogFormat) {
        self.format = format;
    }
    
    /// Enable or disable colored output
    pub fn set_color(&mut self, use_color: bool) {
        self.use_color = use_color;
    }
    
    /// Set session correlation ID
    pub async fn set_session_id(&self, session_id: String) {
        let mut context = self.context.write().await;
        context.session_id = Some(session_id);
    }
    
    /// Add context field for all subsequent log entries
    pub async fn add_context_field<T: Serialize>(&self, key: String, value: T) {
        if let Ok(json_value) = serde_json::to_value(value) {
            let mut context = self.context.write().await;
            context.context_fields.insert(key, json_value);
        }
    }
    
    /// Start a correlated operation
    pub async fn start_operation(&self, operation_name: &str) -> String {
        let correlation_id = Uuid::new_v4().to_string();
        {
            let mut context = self.context.write().await;
            context.current_correlation_id = Some(correlation_id.clone());
        }
        
        self.info(&format!("Started operation: {}", operation_name))
            .correlation_id(&correlation_id)
            .field("operation", operation_name)
            .field("operation_type", "start")
            .log()
            .await;
        
        correlation_id
    }
    
    /// End a correlated operation
    pub async fn end_operation(&self, correlation_id: &str, operation_name: &str, success: bool) {
        self.info(&format!("Completed operation: {} (success: {})", operation_name, success))
            .correlation_id(correlation_id)
            .field("operation", operation_name)
            .field("operation_type", "end")
            .field("success", success)
            .log()
            .await;
        
        // Clear current correlation ID if it matches
        let mut context = self.context.write().await;
        if context.current_correlation_id.as_ref() == Some(&correlation_id.to_string()) {
            context.current_correlation_id = None;
        }
    }
    
    /// Create a log entry builder
    pub fn log(&self, level: LogLevel, message: &str) -> LogEntryBuilder {
        LogEntryBuilder::new(self, level, message.to_string())
    }
    
    /// Convenience methods for different log levels
    pub fn trace(&self, message: &str) -> LogEntryBuilder {
        self.log(LogLevel::Trace, message)
    }
    
    pub fn debug(&self, message: &str) -> LogEntryBuilder {
        self.log(LogLevel::Debug, message)
    }
    
    pub fn info(&self, message: &str) -> LogEntryBuilder {
        self.log(LogLevel::Info, message)
    }
    
    pub fn warn(&self, message: &str) -> LogEntryBuilder {
        self.log(LogLevel::Warn, message)
    }
    
    pub fn error(&self, message: &str) -> LogEntryBuilder {
        self.log(LogLevel::Error, message)
    }
    
    pub fn fatal(&self, message: &str) -> LogEntryBuilder {
        self.log(LogLevel::Fatal, message)
    }
    
    /// Check if a log level would be output
    pub fn would_log(&self, level: LogLevel) -> bool {
        level >= self.min_level
    }
    
    /// Write log entry to output
    async fn write_entry(&self, mut entry: LogEntry) {
        // Don't output if below minimum level
        if entry.level < self.min_level {
            return;
        }
        
        // Add context fields
        let context = self.context.read().await;
        if let Some(session_id) = &context.session_id {
            entry.fields.insert("session_id".to_string(), serde_json::Value::String(session_id.clone()));
        }
        
        for (key, value) in &context.context_fields {
            entry.fields.insert(key.clone(), value.clone());
        }
        drop(context);
        
        // Format and write the entry
        let output = match self.format {
            LogFormat::Console => self.format_console(&entry),
            LogFormat::Json => self.format_json(&entry),
            LogFormat::Compact => self.format_compact(&entry),
        };
        
        // Write to stderr for errors/warnings, stdout for others
        if entry.level >= LogLevel::Warn {
            let _ = writeln!(io::stderr(), "{}", output);
        } else {
            let _ = writeln!(io::stdout(), "{}", output);
        }
    }
    
    /// Format log entry for console output
    fn format_console(&self, entry: &LogEntry) -> String {
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let level_str = entry.level.as_str();
        
        let formatted_level = if self.use_color {
            format!("{}{:>5}{}", entry.level.color_code(), level_str, LogLevel::reset_code())
        } else {
            format!("{:>5}", level_str)
        };
        
        let mut output = format!("{} {} [{}] {}", 
            timestamp, 
            formatted_level, 
            entry.logger, 
            entry.message
        );
        
        // Add correlation ID if present
        if let Some(correlation_id) = &entry.correlation_id {
            output.push_str(&format!(" [{}]", &correlation_id[..8])); // Show first 8 chars
        }
        
        // Add fields if any
        if !entry.fields.is_empty() {
            let fields_str: Vec<String> = entry.fields.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            output.push_str(&format!(" {{{}}}", fields_str.join(", ")));
        }
        
        // Add location if available and enabled
        if self.include_location {
            if let Some(location) = &entry.location {
                output.push_str(&format!(" @ {}:{}", location.file, location.line));
            }
        }
        
        output
    }
    
    /// Format log entry as JSON
    fn format_json(&self, entry: &LogEntry) -> String {
        match serde_json::to_string(entry) {
            Ok(json) => json,
            Err(_) => format!("{{\"error\": \"Failed to serialize log entry\", \"message\": \"{}\"}}", entry.message),
        }
    }
    
    /// Format log entry in compact format
    fn format_compact(&self, entry: &LogEntry) -> String {
        let timestamp = entry.timestamp.format("%H:%M:%S");
        format!("{} {} {}: {}", 
            timestamp, 
            entry.level.as_str().chars().next().unwrap_or('?'), 
            entry.logger, 
            entry.message
        )
    }
}

/// Builder pattern for creating log entries
pub struct LogEntryBuilder<'a> {
    logger: &'a Logger,
    entry: LogEntry,
}

impl<'a> LogEntryBuilder<'a> {
    fn new(logger: &'a Logger, level: LogLevel, message: String) -> Self {
        Self {
            logger,
            entry: LogEntry {
                timestamp: Utc::now(),
                level,
                message,
                logger: logger.name.clone(),
                correlation_id: None,
                fields: HashMap::new(),
                thread_id: std::thread::current().name().map(String::from),
                location: None,
            },
        }
    }
    
    /// Add a correlation ID
    pub fn correlation_id(mut self, id: &str) -> Self {
        self.entry.correlation_id = Some(id.to_string());
        self
    }
    
    /// Add a structured field
    pub fn field<T: Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.entry.fields.insert(key.to_string(), json_value);
        }
        self
    }
    
    /// Add location information
    pub fn location(mut self, file: &str, line: u32, module: Option<&str>) -> Self {
        self.entry.location = Some(LogLocation {
            file: file.to_string(),
            line,
            module: module.map(String::from),
        });
        self
    }
    
    /// Add timing information
    pub fn timing(self, metrics: &TimingMetrics) -> Self {
        self.field("dns_ms", metrics.dns_ms())
            .field("tcp_ms", metrics.tcp_ms())
            .field("tls_ms", metrics.tls_ms())
            .field("first_byte_ms", metrics.first_byte_ms())
            .field("total_ms", metrics.total_ms())
            .field("http_status", metrics.http_status)
            .field("success", metrics.is_successful())
    }
    
    /// Add error information
    pub fn error_info(self, error: &AppError) -> Self {
        self.field("error_category", error.category())
            .field("error_recoverable", error.is_recoverable())
            .field("error_exit_code", error.exit_code())
    }
    
    /// Finalize and write the log entry
    pub async fn log(self) {
        self.logger.write_entry(self.entry).await;
    }
}

impl PerformanceLogger {
    /// Create a new performance logger
    pub fn new(config: &Config) -> Self {
        Self {
            logger: Logger::with_config("PERF".to_string(), config),
            start_times: HashMap::new(),
            operation_stack: Vec::new(),
        }
    }
    
    /// Start timing an operation
    pub async fn start_timing(&mut self, operation: &str) {
        let start_time = Utc::now();
        self.start_times.insert(operation.to_string(), start_time);
        self.operation_stack.push(operation.to_string());
        
        self.logger.debug(&format!("Started timing: {}", operation))
            .field("operation", operation)
            .field("start_time", start_time)
            .log()
            .await;
    }
    
    /// End timing an operation and log the duration
    pub async fn end_timing(&mut self, operation: &str) -> Option<chrono::Duration> {
        if let Some(start_time) = self.start_times.remove(operation) {
            let end_time = Utc::now();
            let duration = end_time - start_time;
            
            // Remove from operation stack
            if let Some(pos) = self.operation_stack.iter().position(|x| x == operation) {
                self.operation_stack.remove(pos);
            }
            
            self.logger.info(&format!("Completed timing: {} in {}ms", operation, duration.num_milliseconds()))
                .field("operation", operation)
                .field("start_time", start_time)
                .field("end_time", end_time)
                .field("duration_ms", duration.num_milliseconds())
                .log()
                .await;
            
            Some(duration)
        } else {
            self.logger.warn(&format!("Attempted to end timing for unknown operation: {}", operation))
                .field("operation", operation)
                .log()
                .await;
            None
        }
    }
    
    /// Log test result with detailed performance metrics
    pub async fn log_test_result(&self, result: &TestResult) {
        for (i, timing) in result.individual_results.iter().enumerate() {
            self.logger.debug(&format!("Test iteration {} for {}", i + 1, result.url))
                .field("url", &result.url)
                .field("dns_config", &result.config_name)
                .field("iteration", i + 1)
                .timing(timing)
                .log()
                .await;
        }
        
        if let Some(stats) = &result.statistics {
            self.logger.info(&format!("Test completed for {} with {}: avg={}ms, success_rate={:.1}%", 
                result.url, result.config_name, stats.format_avg_total(), result.success_rate()))
                .field("url", &result.url)
                .field("dns_config", &result.config_name)
                .field("total_count", result.total_count)
                .field("success_count", result.success_count)
                .field("success_rate", result.success_rate())
                .field("avg_total_ms", stats.total_avg_ms)
                .field("min_ms", stats.total_min_ms)
                .field("max_ms", stats.total_max_ms)
                .field("std_dev_ms", stats.total_std_dev_ms)
                .log()
                .await;
        }
    }
    
    /// Get currently active operations
    pub fn active_operations(&self) -> &[String] {
        &self.operation_stack
    }

    /// Log completion of a configuration operation with performance summary
    pub async fn log_operation_complete(
        &self,
        operation_name: &str,
        duration: std::time::Duration,
        test_count: usize,
        success_rate: f64,
        additional_info: Option<&str>,
    ) {
        let message = format!(
            "Operation '{}' completed: {} tests, {:.1}% success rate, {:.3}ms avg",
            operation_name,
            test_count,
            success_rate,
            duration.as_secs_f64() * 1000.0
        );

        let mut builder = self.logger.info(&message)
            .field("operation", operation_name)
            .field("duration_ms", duration.as_secs_f64() * 1000.0)
            .field("test_count", test_count)
            .field("success_rate", success_rate)
            .field("operation_type", "completed");

        if let Some(info) = additional_info {
            builder = builder.field("additional_info", info);
        }

        builder.log().await;
    }

    /// Log batch operation summary with aggregate timing information
    pub async fn log_batch_summary(
        &self,
        batch_size: usize,
        total_duration: std::time::Duration,
        additional_info: Option<&str>,
    ) {
        let avg_per_item = if batch_size > 0 {
            total_duration.as_secs_f64() / batch_size as f64
        } else {
            0.0
        };

        let message = format!(
            "Batch summary: {} items processed in {:.3}s (avg {:.3}s per item)",
            batch_size,
            total_duration.as_secs_f64(),
            avg_per_item
        );

        let mut builder = self.logger.info(&message)
            .field("batch_size", batch_size)
            .field("total_duration_seconds", total_duration.as_secs_f64())
            .field("avg_per_item_seconds", avg_per_item)
            .field("operation_type", "batch_summary");

        if let Some(info) = additional_info {
            builder = builder.field("additional_info", info);
        }

        builder.log().await;
    }
}

impl NetworkLogger {
    /// Create a new network logger
    pub fn new(config: &Config) -> Self {
        Self {
            logger: Logger::with_config("NET".to_string(), config),
        }
    }
    
    /// Log DNS resolution attempt
    pub async fn log_dns_resolution(&self, domain: &str, dns_config: &str, success: bool, duration_ms: f64) {
        let level = if success { LogLevel::Debug } else { LogLevel::Warn };
        let message = format!("DNS resolution for {} using {}: {}", 
            domain, dns_config, if success { "success" } else { "failed" });
        
        self.logger.log(level, &message)
            .field("domain", domain)
            .field("dns_config", dns_config)
            .field("success", success)
            .field("duration_ms", duration_ms)
            .log()
            .await;
    }
    
    /// Log HTTP request
    pub async fn log_http_request(&self, url: &str, method: &str, status_code: Option<u16>, duration_ms: f64) {
        let success = status_code.map_or(false, |code| code >= 200 && code < 400);
        let level = if success { LogLevel::Debug } else { LogLevel::Warn };
        
        let message = format!("{} {} -> {} in {:.1}ms", 
            method, url, 
            status_code.map_or("FAILED".to_string(), |c| c.to_string()),
            duration_ms);
        
        self.logger.log(level, &message)
            .field("url", url)
            .field("method", method)
            .field("status_code", status_code)
            .field("success", success)
            .field("duration_ms", duration_ms)
            .log()
            .await;
    }
    
    /// Log connection attempt
    pub async fn log_connection(&self, target: &str, success: bool, error: Option<&str>) {
        let level = if success { LogLevel::Debug } else { LogLevel::Warn };
        let message = if success {
            format!("Connected to {}", target)
        } else {
            format!("Failed to connect to {}: {}", target, error.unwrap_or("unknown error"))
        };
        
        let mut builder = self.logger.log(level, &message)
            .field("target", target)
            .field("success", success);
        
        if let Some(err) = error {
            builder = builder.field("error", err);
        }
        
        builder.log().await;
    }
}

impl ErrorEventLogger {
    /// Create a new error event logger
    pub fn new(config: &Config) -> Self {
        Self {
            logger: Logger::with_config("ERR".to_string(), config),
        }
    }
    
    /// Log an application error with full context
    pub async fn log_error(&self, error: &AppError, context: Option<&str>, correlation_id: Option<&str>) {
        let message = if let Some(ctx) = context {
            format!("{}: {}", ctx, error)
        } else {
            error.to_string()
        };
        
        let mut builder = self.logger.error(&message)
            .error_info(error);
        
        if let Some(id) = correlation_id {
            builder = builder.correlation_id(id);
        }
        
        if let Some(ctx) = context {
            builder = builder.field("context", ctx);
        }
        
        builder.log().await;
    }
    
    /// Log error recovery attempt
    pub async fn log_recovery_attempt(&self, error: &AppError, recovery_action: &str, correlation_id: Option<&str>) {
        let message = format!("Attempting recovery from {}: {}", error.category(), recovery_action);
        
        let mut builder = self.logger.warn(&message)
            .field("error_category", error.category())
            .field("recovery_action", recovery_action)
            .field("error_recoverable", error.is_recoverable());
        
        if let Some(id) = correlation_id {
            builder = builder.correlation_id(id);
        }
        
        builder.log().await;
    }
    
    /// Log successful error recovery
    pub async fn log_recovery_success(&self, error_category: &str, recovery_action: &str, correlation_id: Option<&str>) {
        let message = format!("Successfully recovered from {} using: {}", error_category, recovery_action);
        
        let mut builder = self.logger.info(&message)
            .field("error_category", error_category)
            .field("recovery_action", recovery_action)
            .field("recovery_success", true);
        
        if let Some(id) = correlation_id {
            builder = builder.correlation_id(id);
        }
        
        builder.log().await;
    }
    
    /// Log failed error recovery
    pub async fn log_recovery_failure(&self, error_category: &str, recovery_action: &str, new_error: Option<&AppError>, correlation_id: Option<&str>) {
        let message = format!("Failed to recover from {} using: {}", error_category, recovery_action);
        
        let mut builder = self.logger.error(&message)
            .field("error_category", error_category)
            .field("recovery_action", recovery_action)
            .field("recovery_success", false);
        
        if let Some(new_err) = new_error {
            builder = builder.field("new_error", new_err.to_string())
                .field("new_error_category", new_err.category());
        }
        
        if let Some(id) = correlation_id {
            builder = builder.correlation_id(id);
        }
        
        builder.log().await;
    }
}

/// Global logger factory and management
pub struct LoggerFactory {
    config: Config,
    session_id: String,
}

impl LoggerFactory {
    /// Create a new logger factory
    pub fn new(config: Config) -> Self {
        Self {
            config,
            session_id: Uuid::new_v4().to_string(),
        }
    }
    
    /// Create a logger with a specific name
    pub async fn create_logger(&self, name: &str) -> Logger {
        let logger = Logger::with_config(name.to_string(), &self.config);
        logger.set_session_id(self.session_id.clone()).await;
        logger
    }
    
    /// Create a performance logger
    pub fn create_performance_logger(&self) -> PerformanceLogger {
        PerformanceLogger::new(&self.config)
    }
    
    /// Create a network logger
    pub fn create_network_logger(&self) -> NetworkLogger {
        NetworkLogger::new(&self.config)
    }
    
    /// Create an error event logger
    pub fn create_error_logger(&self) -> ErrorEventLogger {
        ErrorEventLogger::new(&self.config)
    }
    
    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

/// Convenience macros for logging with location information
#[macro_export]
macro_rules! log_trace {
    ($logger:expr, $($arg:tt)*) => {
        $logger.trace(&format!($($arg)*))
            .location(file!(), line!(), Some(module_path!()))
            .log()
            .await
    };
}

#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $($arg:tt)*) => {
        $logger.debug(&format!($($arg)*))
            .location(file!(), line!(), Some(module_path!()))
            .log()
            .await
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.info(&format!($($arg)*))
            .location(file!(), line!(), Some(module_path!()))
            .log()
            .await
    };
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $($arg:tt)*) => {
        $logger.warn(&format!($($arg)*))
            .location(file!(), line!(), Some(module_path!()))
            .log()
            .await
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.error(&format!($($arg)*))
            .location(file!(), line!(), Some(module_path!()))
            .log()
            .await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    
    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("DEBUG").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("WARN").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("warning").unwrap(), LogLevel::Warn);
        assert!(LogLevel::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Fatal);
    }
    
    #[test]
    fn test_log_level_strings() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Fatal.as_str(), "FATAL");
    }
    
    #[tokio::test]
    async fn test_logger_creation() {
        let logger = Logger::new("TEST".to_string());
        assert_eq!(logger.name, "TEST");
        assert_eq!(logger.min_level, LogLevel::Info);
        assert!(logger.use_color);
    }
    
    #[tokio::test]
    async fn test_logger_with_config() {
        let config = Config {
            debug: true,
            verbose: true,
            enable_color: false,
            ..Default::default()
        };
        
        let logger = Logger::with_config("TEST".to_string(), &config);
        assert_eq!(logger.min_level, LogLevel::Debug);
        assert!(!logger.use_color);
        assert!(logger.include_location);
    }
    
    #[tokio::test]
    async fn test_session_id_management() {
        let logger = Logger::new("TEST".to_string());
        logger.set_session_id("test-session".to_string()).await;
        
        let context = logger.context.read().await;
        assert_eq!(context.session_id.as_ref().unwrap(), "test-session");
    }
    
    #[tokio::test]
    async fn test_context_fields() {
        let logger = Logger::new("TEST".to_string());
        logger.add_context_field("test_key".to_string(), "test_value").await;
        
        let context = logger.context.read().await;
        assert!(context.context_fields.contains_key("test_key"));
    }
    
    #[tokio::test]
    async fn test_operation_correlation() {
        let logger = Logger::new("TEST".to_string());
        let correlation_id = logger.start_operation("test_operation").await;
        
        assert!(!correlation_id.is_empty());
        
        logger.end_operation(&correlation_id, "test_operation", true).await;
    }
    
    #[tokio::test]
    async fn test_would_log() {
        let mut logger = Logger::new("TEST".to_string());
        logger.set_level(LogLevel::Warn);
        
        assert!(!logger.would_log(LogLevel::Debug));
        assert!(!logger.would_log(LogLevel::Info));
        assert!(logger.would_log(LogLevel::Warn));
        assert!(logger.would_log(LogLevel::Error));
        assert!(logger.would_log(LogLevel::Fatal));
    }
    
    #[tokio::test]
    async fn test_log_entry_builder() {
        let logger = Logger::new("TEST".to_string());
        
        // Test that the builder pattern works without panicking
        logger.info("test message")
            .correlation_id("test-id")
            .field("test_field", "test_value")
            .location("test.rs", 123, Some("test::module"))
            .log()
            .await;
    }
    
    #[test]
    fn test_performance_logger_creation() {
        let config = Config::default();
        let perf_logger = PerformanceLogger::new(&config);
        assert_eq!(perf_logger.logger.name, "PERF");
    }
    
    #[test]
    fn test_network_logger_creation() {
        let config = Config::default();
        let net_logger = NetworkLogger::new(&config);
        assert_eq!(net_logger.logger.name, "NET");
    }
    
    #[test]
    fn test_error_logger_creation() {
        let config = Config::default();
        let err_logger = ErrorEventLogger::new(&config);
        assert_eq!(err_logger.logger.name, "ERR");
    }
    
    #[tokio::test]
    async fn test_logger_factory() {
        let config = Config::default();
        let factory = LoggerFactory::new(config);
        
        let logger = factory.create_logger("TEST").await;
        assert_eq!(logger.name, "TEST");
        
        let session_id = factory.session_id();
        assert!(!session_id.is_empty());
    }
    
    #[tokio::test]
    async fn test_performance_timing() {
        let config = Config::default();
        let mut perf_logger = PerformanceLogger::new(&config);
        
        perf_logger.start_timing("test_operation").await;
        
        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        
        let duration = perf_logger.end_timing("test_operation").await;
        assert!(duration.is_some());
        assert!(duration.unwrap().num_milliseconds() >= 0);
        
        // Test ending unknown operation
        let unknown_duration = perf_logger.end_timing("unknown_operation").await;
        assert!(unknown_duration.is_none());
    }
    
    #[test]
    fn test_log_formats() {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Test message".to_string(),
            logger: "TEST".to_string(),
            correlation_id: Some("test-id".to_string()),
            fields: {
                let mut map = HashMap::new();
                map.insert("key".to_string(), serde_json::Value::String("value".to_string()));
                map
            },
            thread_id: None,
            location: None,
        };
        
        let logger = Logger::new("TEST".to_string());
        
        // Test console format
        let console_output = logger.format_console(&entry);
        assert!(console_output.contains("INFO"));
        assert!(console_output.contains("Test message"));
        assert!(console_output.contains("test-id"));
        
        // Test JSON format
        let json_output = logger.format_json(&entry);
        assert!(json_output.starts_with('{'));
        assert!(json_output.ends_with('}'));
        
        // Test compact format
        let compact_output = logger.format_compact(&entry);
        assert!(compact_output.contains('I')); // First character of INFO
        assert!(compact_output.contains("Test message"));
    }
    
    #[tokio::test]
    async fn test_timing_metrics_logging() {
        use std::time::Duration;
        use crate::models::TimingMetrics;
        
        let config = Config::default();
        let perf_logger = PerformanceLogger::new(&config);
        
        let timing = TimingMetrics::success(
            Duration::from_millis(10),
            Duration::from_millis(20),
            Some(Duration::from_millis(30)),
            Duration::from_millis(50),
            Duration::from_millis(100),
            200,
        );
        
        // Test that timing information can be logged
        perf_logger.logger.info("Test timing")
            .timing(&timing)
            .log()
            .await;
    }
    
    #[tokio::test]
    async fn test_error_logging() {
        let config = Config::default();
        let err_logger = ErrorEventLogger::new(&config);
        let error = AppError::network("Test network error");
        
        // Test error logging
        err_logger.log_error(&error, Some("During test execution"), Some("test-correlation")).await;
        
        // Test recovery logging
        err_logger.log_recovery_attempt(&error, "Retry with different DNS", Some("test-correlation")).await;
        err_logger.log_recovery_success("NETWORK", "Retry with different DNS", Some("test-correlation")).await;
        err_logger.log_recovery_failure("NETWORK", "Retry failed", Some(&error), Some("test-correlation")).await;
    }
    
    #[tokio::test]
    async fn test_network_logging() {
        let config = Config::default();
        let net_logger = NetworkLogger::new(&config);
        
        // Test DNS resolution logging
        net_logger.log_dns_resolution("example.com", "System DNS", true, 25.5).await;
        net_logger.log_dns_resolution("invalid.domain", "Custom DNS", false, 5000.0).await;
        
        // Test HTTP request logging
        net_logger.log_http_request("https://example.com", "GET", Some(200), 150.0).await;
        net_logger.log_http_request("https://invalid.com", "GET", None, 10000.0).await;
        
        // Test connection logging
        net_logger.log_connection("example.com:443", true, None).await;
        net_logger.log_connection("invalid.com:443", false, Some("Connection refused")).await;
    }
    
    #[test]
    fn test_log_location() {
        let location = LogLocation {
            file: "test.rs".to_string(),
            line: 42,
            module: Some("test::module".to_string()),
        };
        
        assert_eq!(location.file, "test.rs");
        assert_eq!(location.line, 42);
        assert_eq!(location.module.as_ref().unwrap(), "test::module");
    }
    
    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Test".to_string(),
            logger: "TEST".to_string(),
            correlation_id: None,
            fields: HashMap::new(),
            thread_id: None,
            location: None,
        };
        
        // Test that log entry can be serialized/deserialized
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: LogEntry = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.level, LogLevel::Info);
        assert_eq!(deserialized.message, "Test");
        assert_eq!(deserialized.logger, "TEST");
    }
}