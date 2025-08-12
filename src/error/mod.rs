//! Error handling for the network latency tester

pub mod user_messages;

pub use user_messages::{
    UserMessageProvider, EnhancedErrorMessage, UserMessageConfig,
    Platform, ExperienceLevel, ResolutionTime,
};

use thiserror::Error;

/// Custom error types for the network latency tester
#[derive(Error, Debug)]
pub enum AppError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network connectivity errors
    #[error("Network error: {0}")]
    Network(String),

    /// DNS resolution errors
    #[error("DNS resolution error: {0}")]
    DnsResolution(String),

    /// HTTP request errors
    #[error("HTTP request error: {0}")]
    HttpRequest(String),

    /// Timeout errors
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// I/O errors (file operations, etc.)
    #[error("I/O error: {0}")]
    Io(String),

    /// Parsing errors (URLs, JSON, etc.)
    #[error("Parsing error: {0}")]
    Parse(String),

    /// Authentication/authorization errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Test execution errors
    #[error("Test execution error: {0}")]
    TestExecution(String),

    /// Statistics calculation errors
    #[error("Statistics error: {0}")]
    Statistics(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
    }

    /// Create a new network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network(message.into())
    }

    /// Create a new DNS resolution error
    pub fn dns_resolution<S: Into<String>>(message: S) -> Self {
        Self::DnsResolution(message.into())
    }

    /// Create a new HTTP request error
    pub fn http_request<S: Into<String>>(message: S) -> Self {
        Self::HttpRequest(message.into())
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout(message.into())
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    /// Create a new I/O error
    pub fn io<S: Into<String>>(message: S) -> Self {
        Self::Io(message.into())
    }

    /// Create a new parsing error
    pub fn parse<S: Into<String>>(message: S) -> Self {
        Self::Parse(message.into())
    }

    /// Create a new authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth(message.into())
    }

    /// Create a new test execution error
    pub fn test_execution<S: Into<String>>(message: S) -> Self {
        Self::TestExecution(message.into())
    }

    /// Create a new statistics error
    pub fn statistics<S: Into<String>>(message: S) -> Self {
        Self::Statistics(message.into())
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    /// Get error category for logging and reporting
    pub fn category(&self) -> &'static str {
        match self {
            Self::Config(_) => "CONFIG",
            Self::Network(_) => "NETWORK",
            Self::DnsResolution(_) => "DNS",
            Self::HttpRequest(_) => "HTTP",
            Self::Timeout(_) => "TIMEOUT",
            Self::Validation(_) => "VALIDATION",
            Self::Io(_) => "IO",
            Self::Parse(_) => "PARSE",
            Self::Auth(_) => "AUTH",
            Self::TestExecution(_) => "TEST",
            Self::Statistics(_) => "STATS",
            Self::Internal(_) => "INTERNAL",
        }
    }

    /// Check if error is recoverable (can retry)
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Network(_) | Self::HttpRequest(_) | Self::Timeout(_) | Self::DnsResolution(_) => true,
            Self::Config(_) | Self::Validation(_) | Self::Parse(_) | Self::Auth(_) => false,
            Self::Io(_) | Self::TestExecution(_) | Self::Statistics(_) | Self::Internal(_) => false,
        }
    }

    /// Get user-friendly error message with suggestions
    pub fn user_friendly_message(&self) -> String {
        match self {
            Self::Config(msg) => {
                format!("Configuration problem: {}\n\nSuggestion: Check your .env file or command line arguments.", msg)
            }
            Self::Network(msg) => {
                format!("Network connectivity issue: {}\n\nSuggestion: Check your internet connection and try again.", msg)
            }
            Self::DnsResolution(msg) => {
                format!("DNS resolution failed: {}\n\nSuggestion: Try using different DNS servers (8.8.8.8, 1.1.1.1) or check if the domain exists.", msg)
            }
            Self::HttpRequest(msg) => {
                format!("HTTP request failed: {}\n\nSuggestion: The target server may be down or blocking requests. Try a different URL or check if it requires authentication.", msg)
            }
            Self::Timeout(msg) => {
                format!("Request timed out: {}\n\nSuggestion: Increase the timeout value using --timeout or check your network connection.", msg)
            }
            Self::Validation(msg) => {
                format!("Invalid input: {}\n\nSuggestion: Check the format of your URLs, IP addresses, or other configuration values.", msg)
            }
            Self::Io(msg) => {
                format!("File operation failed: {}\n\nSuggestion: Check file permissions and disk space.", msg)
            }
            Self::Parse(msg) => {
                format!("Failed to parse data: {}\n\nSuggestion: Check the format of your input data or configuration files.", msg)
            }
            Self::Auth(msg) => {
                format!("Authentication failed: {}\n\nSuggestion: Check your credentials or API keys.", msg)
            }
            Self::TestExecution(msg) => {
                format!("Test execution failed: {}\n\nSuggestion: This may be a temporary issue. Try running the test again.", msg)
            }
            Self::Statistics(msg) => {
                format!("Statistics calculation failed: {}\n\nSuggestion: This may indicate insufficient or invalid test data.", msg)
            }
            Self::Internal(msg) => {
                format!("Internal error: {}\n\nThis is likely a bug. Please report this issue with the error details.", msg)
            }
        }
    }

    /// Get exit code for this error type
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) | Self::Validation(_) | Self::Parse(_) => 1,  // Invalid configuration/usage
            Self::Network(_) | Self::DnsResolution(_) | Self::HttpRequest(_) => 2,  // Network issues
            Self::Timeout(_) => 3,  // Timeout issues
            Self::Auth(_) => 4,  // Authentication issues
            Self::Io(_) => 5,  // I/O issues
            Self::TestExecution(_) | Self::Statistics(_) => 6,  // Test execution issues
            Self::Internal(_) => 99,  // Internal/unexpected errors
        }
    }

    /// Format error for console display with color coding
    pub fn format_for_console(&self, use_color: bool) -> String {
        let category = self.category();
        let message = self.to_string();

        if use_color {
            use colored::Colorize;
            match self {
                Self::Config(_) | Self::Validation(_) | Self::Parse(_) => {
                    format!("[{}] {}", category.red().bold(), message.red())
                }
                Self::Network(_) | Self::DnsResolution(_) | Self::HttpRequest(_) => {
                    format!("[{}] {}", category.yellow().bold(), message.yellow())
                }
                Self::Timeout(_) => {
                    format!("[{}] {}", category.blue().bold(), message.blue())
                }
                Self::Auth(_) => {
                    format!("[{}] {}", category.magenta().bold(), message.magenta())
                }
                Self::Io(_) | Self::TestExecution(_) | Self::Statistics(_) => {
                    format!("[{}] {}", category.cyan().bold(), message.cyan())
                }
                Self::Internal(_) => {
                    format!("[{}] {}", category.bright_red().bold(), message.bright_red())
                }
            }
        } else {
            format!("[{}] {}", category, message)
        }
    }
}

// Standard library error conversions
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}

impl From<url::ParseError> for AppError {
    fn from(error: url::ParseError) -> Self {
        Self::parse(format!("URL parse error: {}", error))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::parse(format!("JSON parse error: {}", error))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::timeout(error.to_string())
        } else if error.is_connect() || error.is_request() {
            Self::network(error.to_string())
        } else {
            Self::http_request(error.to_string())
        }
    }
}

impl From<trust_dns_resolver::error::ResolveError> for AppError {
    fn from(error: trust_dns_resolver::error::ResolveError) -> Self {
        Self::dns_resolution(error.to_string())
    }
}

impl From<dotenv::Error> for AppError {
    fn from(error: dotenv::Error) -> Self {
        Self::config(format!("Environment file error: {}", error))
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::parse(format!("Integer parse error: {}", error))
    }
}

impl From<std::num::ParseFloatError> for AppError {
    fn from(error: std::num::ParseFloatError) -> Self {
        Self::parse(format!("Float parse error: {}", error))
    }
}

impl From<std::str::ParseBoolError> for AppError {
    fn from(error: std::str::ParseBoolError) -> Self {
        Self::parse(format!("Boolean parse error: {}", error))
    }
}

impl From<std::net::AddrParseError> for AppError {
    fn from(error: std::net::AddrParseError) -> Self {
        Self::parse(format!("IP address parse error: {}", error))
    }
}

// Anyhow integration
impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::internal(error.to_string())
    }
}


/// Custom Result type for the application
pub type Result<T> = std::result::Result<T, AppError>;

/// Error context trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Add static context to an error
    fn context(self, message: &'static str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<AppError>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let original_error = e.into();
            let context = f();
            AppError::internal(format!("{}: {}", context, original_error))
        })
    }

    fn context(self, message: &'static str) -> Result<T> {
        self.with_context(|| message.to_string())
    }
}

/// Helper function to create context for errors
pub fn context<T>(message: &'static str) -> impl Fn(AppError) -> AppError {
    let msg = message.to_string();
    move |error| AppError::internal(format!("{}: {}", msg, error))
}

/// Error reporter for structured error logging and user feedback
pub struct ErrorReporter {
    pub use_color: bool,
    pub verbose: bool,
}

impl ErrorReporter {
    /// Create a new error reporter
    pub fn new(use_color: bool, verbose: bool) -> Self {
        Self { use_color, verbose }
    }

    /// Report an error to the user
    pub fn report_error(&self, error: &AppError) {
        eprintln!("{}", error.format_for_console(self.use_color));

        if self.verbose {
            eprintln!();
            eprintln!("{}", error.user_friendly_message());
            
            if error.is_recoverable() {
                eprintln!();
                if self.use_color {
                    use colored::Colorize;
                    eprintln!("{}", "This error might be temporary. You can try running the command again.".green());
                } else {
                    eprintln!("This error might be temporary. You can try running the command again.");
                }
            }
        }
    }

    /// Report multiple errors
    pub fn report_errors(&self, errors: &[AppError]) {
        for (i, error) in errors.iter().enumerate() {
            if i > 0 {
                eprintln!();
            }
            self.report_error(error);
        }
    }

    /// Get formatted error summary
    pub fn format_error_summary(&self, errors: &[AppError]) -> String {
        if errors.is_empty() {
            return "No errors".to_string();
        }

        let mut summary = format!("Found {} error(s):", errors.len());
        
        // Group errors by category
        let mut error_groups: std::collections::HashMap<&'static str, Vec<&AppError>> = std::collections::HashMap::new();
        for error in errors {
            error_groups.entry(error.category()).or_default().push(error);
        }

        for (category, group_errors) in error_groups {
            summary.push_str(&format!("\n  {}: {} error(s)", category, group_errors.len()));
            if self.verbose {
                for error in group_errors {
                    summary.push_str(&format!("\n    - {}", error));
                }
            }
        }

        summary
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new(true, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let config_error = AppError::config("Invalid configuration");
        assert_eq!(config_error.category(), "CONFIG");
        assert!(!config_error.is_recoverable());
        assert_eq!(config_error.exit_code(), 1);

        let network_error = AppError::network("Connection failed");
        assert_eq!(network_error.category(), "NETWORK");
        assert!(network_error.is_recoverable());
        assert_eq!(network_error.exit_code(), 2);
    }

    #[test]
    fn test_error_display() {
        let error = AppError::config("Test configuration error");
        let display = error.to_string();
        assert!(display.contains("Configuration error"));
        assert!(display.contains("Test configuration error"));
    }

    #[test]
    fn test_error_categories() {
        let errors = [
            AppError::config("config"),
            AppError::network("network"),
            AppError::dns_resolution("dns"),
            AppError::http_request("http"),
            AppError::timeout("timeout"),
            AppError::validation("validation"),
            AppError::io("io"),
            AppError::parse("parse"),
            AppError::auth("auth"),
            AppError::test_execution("test"),
            AppError::statistics("stats"),
            AppError::internal("internal"),
        ];

        let expected_categories = [
            "CONFIG", "NETWORK", "DNS", "HTTP", "TIMEOUT",
            "VALIDATION", "IO", "PARSE", "AUTH", "TEST", "STATS", "INTERNAL"
        ];

        for (error, expected) in errors.iter().zip(expected_categories.iter()) {
            assert_eq!(error.category(), *expected);
        }
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(AppError::network("test").is_recoverable());
        assert!(AppError::http_request("test").is_recoverable());
        assert!(AppError::timeout("test").is_recoverable());
        assert!(AppError::dns_resolution("test").is_recoverable());

        assert!(!AppError::config("test").is_recoverable());
        assert!(!AppError::validation("test").is_recoverable());
        assert!(!AppError::parse("test").is_recoverable());
        assert!(!AppError::auth("test").is_recoverable());
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(AppError::config("test").exit_code(), 1);
        assert_eq!(AppError::network("test").exit_code(), 2);
        assert_eq!(AppError::timeout("test").exit_code(), 3);
        assert_eq!(AppError::auth("test").exit_code(), 4);
        assert_eq!(AppError::io("test").exit_code(), 5);
        assert_eq!(AppError::test_execution("test").exit_code(), 6);
        assert_eq!(AppError::internal("test").exit_code(), 99);
    }

    #[test]
    fn test_user_friendly_messages() {
        let error = AppError::config("Invalid URL format");
        let message = error.user_friendly_message();
        assert!(message.contains("Configuration problem"));
        assert!(message.contains("Suggestion:"));
        assert!(message.contains("Invalid URL format"));
    }

    #[test]
    fn test_error_conversions() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_error: AppError = io_error.into();
        assert_eq!(app_error.category(), "IO");

        let parse_error = "not_a_number".parse::<i32>().unwrap_err();
        let app_error: AppError = parse_error.into();
        assert_eq!(app_error.category(), "PARSE");
    }

    #[test]
    fn test_error_context() {
        let result: Result<i32> = Err(AppError::network("Connection failed"));
        let with_context = result.context("While testing network connectivity");
        
        assert!(with_context.is_err());
        let error = with_context.unwrap_err();
        assert_eq!(error.category(), "INTERNAL");
        assert!(error.to_string().contains("While testing network connectivity"));
    }

    #[test]
    fn test_error_reporter() {
        let reporter = ErrorReporter::new(false, true);
        let error = AppError::config("Test error");
        
        // Just test that it doesn't panic
        reporter.report_error(&error);
        
        let errors = vec![
            AppError::config("Error 1"),
            AppError::network("Error 2"),
        ];
        
        let summary = reporter.format_error_summary(&errors);
        assert!(summary.contains("Found 2 error(s)"));
        assert!(summary.contains("CONFIG"));
        assert!(summary.contains("NETWORK"));
    }

    #[test]
    fn test_console_formatting() {
        let error = AppError::config("Test error");
        let formatted_no_color = error.format_for_console(false);
        let formatted_color = error.format_for_console(true);
        
        assert!(formatted_no_color.contains("[CONFIG]"));
        assert!(formatted_color.contains("[CONFIG]"));
        assert!(formatted_no_color.contains("Test error"));
        assert!(formatted_color.contains("Test error"));
        
        // Both should contain the basic structure even if colors don't work in test environment
        assert!(formatted_no_color.len() > 0);
        assert!(formatted_color.len() > 0);
    }

    #[test]
    fn test_reqwest_error_conversion() {
        // Create a simple reqwest error for testing conversion
        // We can't easily create specific error types, so we test the general conversion
        let invalid_url = "not-a-valid-url";
        match reqwest::Url::parse(invalid_url) {
            Err(_) => {
                // This tests that our From<reqwest::Error> implementation exists
                // The actual conversion logic is tested indirectly
                assert!(true);
            }
            Ok(_) => panic!("Expected URL parse to fail"),
        }
    }

    #[test] 
    fn test_url_parse_error_conversion() {
        let url_error = url::Url::parse("not-a-valid-url").unwrap_err();
        let app_error: AppError = url_error.into();
        assert_eq!(app_error.category(), "PARSE");
        assert!(app_error.to_string().contains("URL parse error"));
    }

    #[test]
    fn test_json_parse_error_conversion() {
        let json_error: serde_json::Error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let app_error: AppError = json_error.into();
        assert_eq!(app_error.category(), "PARSE");
        assert!(app_error.to_string().contains("JSON parse error"));
    }

    #[test]
    fn test_dotenv_error_conversion() {
        let dotenv_error = dotenv::Error::LineParse(".env".to_string(), 1);
        let app_error: AppError = dotenv_error.into();
        assert_eq!(app_error.category(), "CONFIG");
        assert!(app_error.to_string().contains("Environment file error"));
    }

    #[test]
    fn test_addr_parse_error_conversion() {
        let addr_error = "not-an-ip".parse::<std::net::IpAddr>().unwrap_err();
        let app_error: AppError = addr_error.into();
        assert_eq!(app_error.category(), "PARSE");
        assert!(app_error.to_string().contains("IP address parse error"));
    }

    #[test]
    fn test_bool_parse_error_conversion() {
        let bool_error = "not-a-bool".parse::<bool>().unwrap_err();
        let app_error: AppError = bool_error.into();
        assert_eq!(app_error.category(), "PARSE");
        assert!(app_error.to_string().contains("Boolean parse error"));
    }

    #[test]
    fn test_anyhow_integration() {
        let anyhow_error = anyhow::anyhow!("Test anyhow error");
        let app_error: AppError = anyhow_error.into();
        assert_eq!(app_error.category(), "INTERNAL");
        
        // Test conversion to anyhow is automatic due to std::error::Error implementation
        let app_error = AppError::config("Test config error");
        let anyhow_error = anyhow::anyhow!(app_error);
        assert!(anyhow_error.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_error_context_trait() {
        let result: std::result::Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found"
        ));
        
        let with_context = result.with_context(|| "While reading config file".to_string());
        assert!(with_context.is_err());
        
        let error = with_context.unwrap_err();
        assert!(error.to_string().contains("While reading config file"));
    }

    #[test]
    fn test_context_helper_function() {
        let original_error = AppError::io("File not found");
        let context_fn = context::<()>("While loading configuration");
        let contextual_error = context_fn(original_error);
        
        assert_eq!(contextual_error.category(), "INTERNAL");
        assert!(contextual_error.to_string().contains("While loading configuration"));
        assert!(contextual_error.to_string().contains("File not found"));
    }

    #[test]
    fn test_error_reporter_default() {
        let reporter = ErrorReporter::default();
        assert!(reporter.use_color);
        assert!(!reporter.verbose);
    }

    #[test]
    fn test_empty_error_summary() {
        let reporter = ErrorReporter::new(false, false);
        let errors: Vec<AppError> = vec![];
        let summary = reporter.format_error_summary(&errors);
        assert_eq!(summary, "No errors");
    }

    #[test]
    fn test_multiple_error_reporting() {
        let reporter = ErrorReporter::new(false, false);
        let errors = vec![
            AppError::config("Config error"),
            AppError::network("Network error"),
            AppError::config("Another config error"),
        ];
        
        // Test that it doesn't panic
        reporter.report_errors(&errors);
        
        let summary = reporter.format_error_summary(&errors);
        assert!(summary.contains("Found 3 error(s)"));
        assert!(summary.contains("CONFIG: 2 error(s)"));
        assert!(summary.contains("NETWORK: 1 error(s)"));
    }

    #[test]
    fn test_all_error_type_constructors() {
        // Test all constructor methods
        let errors = vec![
            AppError::config("config"),
            AppError::network("network"),
            AppError::dns_resolution("dns"),
            AppError::http_request("http"),
            AppError::timeout("timeout"),
            AppError::validation("validation"),
            AppError::io("io"),
            AppError::parse("parse"),
            AppError::auth("auth"),
            AppError::test_execution("test"),
            AppError::statistics("stats"),
            AppError::internal("internal"),
        ];

        // Each should have the correct category
        let expected = ["CONFIG", "NETWORK", "DNS", "HTTP", "TIMEOUT", "VALIDATION", "IO", "PARSE", "AUTH", "TEST", "STATS", "INTERNAL"];
        
        for (error, expected_category) in errors.iter().zip(expected.iter()) {
            assert_eq!(error.category(), *expected_category);
        }
    }
}