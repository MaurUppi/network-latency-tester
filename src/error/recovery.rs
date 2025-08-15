//! Error recovery mechanisms for graceful handling of recoverable failures
//!
//! This module provides intelligent error recovery strategies including:
//! - Automatic retry with exponential backoff
//! - Fallback DNS server selection
//! - Alternative URL testing
//! - Adaptive timeout adjustment
//! - Circuit breaker pattern for repeated failures

#![allow(dead_code)]

use super::AppError;
use crate::logging::{ErrorEventLogger, Logger};
use crate::models::Config;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use std::collections::HashMap;
use uuid::Uuid;

/// Result type for recovery operations
pub type RecoveryResult<T> = std::result::Result<T, RecoveryError>;

/// Recovery-specific error types
#[derive(Debug, Clone)]
pub enum RecoveryError {
    /// All recovery attempts exhausted
    AllAttemptsFailed {
        attempts: usize,
        last_error: AppError,
        recovery_history: Vec<RecoveryAttempt>,
    },
    /// Recovery not applicable for this error type
    NotRecoverable(AppError),
    /// Recovery configuration invalid
    InvalidConfiguration(String),
    /// Timeout during recovery process
    RecoveryTimeout {
        duration: Duration,
        last_error: AppError,
    },
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllAttemptsFailed { attempts, last_error, .. } => {
                write!(f, "All {} recovery attempts failed. Last error: {}", attempts, last_error)
            }
            Self::NotRecoverable(error) => {
                write!(f, "Error is not recoverable: {}", error)
            }
            Self::InvalidConfiguration(msg) => {
                write!(f, "Invalid recovery configuration: {}", msg)
            }
            Self::RecoveryTimeout { duration, last_error } => {
                write!(f, "Recovery timed out after {}ms. Last error: {}", duration.as_millis(), last_error)
            }
        }
    }
}

impl std::error::Error for RecoveryError {}

/// Configuration for error recovery behavior
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum total time to spend on recovery
    pub max_recovery_time: Duration,
    /// Enable circuit breaker pattern
    pub enable_circuit_breaker: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: usize,
    /// Circuit breaker reset timeout
    pub circuit_breaker_reset_time: Duration,
    /// Enable adaptive timeout adjustment
    pub enable_adaptive_timeout: bool,
    /// Alternative DNS servers for fallback
    pub fallback_dns_servers: Vec<String>,
    /// Enable DNS fallback recovery
    pub enable_dns_fallback: bool,
    /// Enable URL validation recovery
    pub enable_url_validation: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            max_recovery_time: Duration::from_secs(30),
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_reset_time: Duration::from_secs(30),
            enable_adaptive_timeout: true,
            fallback_dns_servers: vec![
                "8.8.8.8".to_string(),
                "1.1.1.1".to_string(),
                "208.67.222.222".to_string(),
                "9.9.9.9".to_string(),
            ],
            enable_dns_fallback: true,
            enable_url_validation: true,
        }
    }
}

/// Individual recovery attempt record
#[derive(Debug, Clone)]
pub struct RecoveryAttempt {
    /// Attempt number (1-based)
    pub attempt_number: usize,
    /// Recovery strategy used
    pub strategy: RecoveryStrategy,
    /// Time when attempt was made
    pub timestamp: Instant,
    /// Duration of the attempt
    pub duration: Duration,
    /// Result of the attempt
    pub result: Result<(), AppError>,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Available recovery strategies
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Simple retry with delay
    Retry { delay: Duration },
    /// Retry with exponential backoff
    ExponentialBackoff { base_delay: Duration, multiplier: f64 },
    /// Switch to fallback DNS servers
    DnsFallback { fallback_server: String },
    /// Adjust timeout and retry
    TimeoutAdjustment { new_timeout: Duration },
    /// Validate and correct URL format
    UrlValidation { corrected_url: Option<String> },
    /// Circuit breaker reset
    CircuitBreakerReset,
    /// Combined strategy
    Combined { strategies: Vec<RecoveryStrategy> },
}

impl RecoveryStrategy {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Retry { .. } => "retry",
            Self::ExponentialBackoff { .. } => "exponential_backoff",
            Self::DnsFallback { .. } => "dns_fallback",
            Self::TimeoutAdjustment { .. } => "timeout_adjustment",
            Self::UrlValidation { .. } => "url_validation",
            Self::CircuitBreakerReset => "circuit_breaker_reset",
            Self::Combined { .. } => "combined",
        }
    }
}

/// Circuit breaker state for preventing cascade failures
#[derive(Debug, Clone, PartialEq)]
enum CircuitBreakerState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failing fast)
    Open { opened_at: Instant },
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Circuit breaker for a specific operation/resource
#[derive(Debug)]
struct CircuitBreaker {
    /// Current state
    state: CircuitBreakerState,
    /// Number of consecutive failures
    failure_count: usize,
    /// Failure threshold to open circuit
    failure_threshold: usize,
    /// Time to wait before attempting recovery
    reset_timeout: Duration,
}

impl CircuitBreaker {
    fn new(failure_threshold: usize, reset_timeout: Duration) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            failure_threshold,
            reset_timeout,
        }
    }
    
    /// Check if operation should be allowed
    fn can_execute(&mut self) -> bool {
        match &self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open { opened_at } => {
                if opened_at.elapsed() >= self.reset_timeout {
                    self.state = CircuitBreakerState::HalfOpen;
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    /// Record operation success
    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
    }
    
    /// Record operation failure
    fn record_failure(&mut self) {
        self.failure_count += 1;
        
        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitBreakerState::Open { opened_at: Instant::now() };
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open { opened_at: Instant::now() };
            }
            CircuitBreakerState::Open { .. } => {
                // Already open, update timestamp
                self.state = CircuitBreakerState::Open { opened_at: Instant::now() };
            }
        }
    }
}

/// Main error recovery manager
pub struct ErrorRecoveryManager {
    /// Recovery configuration
    config: RecoveryConfig,
    /// Circuit breakers by resource/operation key
    circuit_breakers: HashMap<String, CircuitBreaker>,
    /// Recovery history for analysis
    recovery_history: Vec<RecoveryAttempt>,
    /// Error event logger
    error_logger: ErrorEventLogger,
    /// General logger
    logger: Logger,
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new(config: RecoveryConfig, app_config: &Config) -> Self {
        Self {
            config,
            circuit_breakers: HashMap::new(),
            recovery_history: Vec::new(),
            error_logger: ErrorEventLogger::new(app_config),
            logger: Logger::with_config("RECOVERY".to_string(), app_config),
        }
    }
    
    /// Create with default configuration
    pub fn with_defaults(app_config: &Config) -> Self {
        Self::new(RecoveryConfig::default(), app_config)
    }
    
    /// Attempt to recover from an error using appropriate strategies
    pub async fn recover_from_error<T, F>(&mut self, 
        error: &AppError, 
        operation_key: &str,
        recovery_operation: F
    ) -> RecoveryResult<T> 
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, AppError>> + Send>> + Send + Sync,
    {
        let correlation_id = Uuid::new_v4().to_string();
        let recovery_start = Instant::now();
        
        // Check if error is recoverable
        if !error.is_recoverable() {
            self.error_logger.log_recovery_attempt(
                error, 
                "Error marked as non-recoverable", 
                Some(&correlation_id)
            ).await;
            return Err(RecoveryError::NotRecoverable(error.clone()));
        }
        
        // Check circuit breaker
        if self.config.enable_circuit_breaker {
            let circuit_breaker = self.circuit_breakers
                .entry(operation_key.to_string())
                .or_insert_with(|| CircuitBreaker::new(
                    self.config.circuit_breaker_threshold,
                    self.config.circuit_breaker_reset_time
                ));
            
            if !circuit_breaker.can_execute() {
                let error_msg = format!("Circuit breaker is open for operation: {}", operation_key);
                self.logger.warn(&error_msg).correlation_id(&correlation_id).log().await;
                return Err(RecoveryError::AllAttemptsFailed {
                    attempts: 0,
                    last_error: error.clone(),
                    recovery_history: vec![],
                });
            }
        }
        
        // Generate recovery strategies
        let strategies = self.generate_recovery_strategies(error);
        let mut recovery_attempts = Vec::new();
        let mut current_delay = self.config.initial_delay;
        
        self.logger.info(&format!("Starting recovery for error: {} using {} strategies", 
            error.category(), strategies.len()))
            .correlation_id(&correlation_id)
            .field("operation_key", operation_key)
            .field("error_category", error.category())
            .field("strategies_count", strategies.len())
            .log().await;
        
        // Attempt recovery with each strategy
        for (attempt_num, strategy) in strategies.iter().enumerate() {
            let attempt_start = Instant::now();
            
            // Check if we've exceeded maximum recovery time
            if recovery_start.elapsed() >= self.config.max_recovery_time {
                return Err(RecoveryError::RecoveryTimeout {
                    duration: recovery_start.elapsed(),
                    last_error: error.clone(),
                });
            }
            
            // Check if we've exceeded maximum attempts
            if attempt_num >= self.config.max_attempts {
                break;
            }
            
            self.logger.debug(&format!("Recovery attempt {} using strategy: {}", 
                attempt_num + 1, strategy.name()))
                .correlation_id(&correlation_id)
                .field("attempt_number", attempt_num + 1)
                .field("strategy", strategy.name())
                .log().await;
            
            // Apply recovery strategy
            self.apply_recovery_strategy(strategy, &correlation_id).await;
            
            // Apply delay before retry (except for first attempt)
            if attempt_num > 0 {
                sleep(current_delay).await;
                current_delay = Duration::from_millis(
                    ((current_delay.as_millis() as f64) * self.config.backoff_multiplier) as u64
                ).min(self.config.max_delay);
            }
            
            // Attempt the operation
            let operation_result = recovery_operation().await;
            let attempt_duration = attempt_start.elapsed();
            
            let mut context = HashMap::new();
            context.insert("strategy".to_string(), strategy.name().to_string());
            context.insert("delay_ms".to_string(), current_delay.as_millis().to_string());
            
            let attempt = RecoveryAttempt {
                attempt_number: attempt_num + 1,
                strategy: strategy.clone(),
                timestamp: attempt_start,
                duration: attempt_duration,
                result: operation_result.as_ref().map(|_| ()).map_err(|e| e.clone()),
                context,
            };
            
            recovery_attempts.push(attempt);
            
            match operation_result {
                Ok(result) => {
                    // Success! Update circuit breaker and log
                    if self.config.enable_circuit_breaker {
                        if let Some(circuit_breaker) = self.circuit_breakers.get_mut(operation_key) {
                            circuit_breaker.record_success();
                        }
                    }
                    
                    self.logger.info(&format!("Recovery successful after {} attempts using strategy: {}", 
                        attempt_num + 1, strategy.name()))
                        .correlation_id(&correlation_id)
                        .field("total_attempts", attempt_num + 1)
                        .field("total_duration_ms", recovery_start.elapsed().as_millis())
                        .field("successful_strategy", strategy.name())
                        .log().await;
                    
                    self.error_logger.log_recovery_success(
                        error.category(),
                        strategy.name(),
                        Some(&correlation_id)
                    ).await;
                    
                    // Store recovery history
                    self.recovery_history.extend(recovery_attempts);
                    
                    return Ok(result);
                }
                Err(attempt_error) => {
                    // Failure, continue with next strategy
                    self.logger.debug(&format!("Recovery attempt {} failed with: {}", 
                        attempt_num + 1, attempt_error))
                        .correlation_id(&correlation_id)
                        .error_info(&attempt_error)
                        .log().await;
                }
            }
        }
        
        // All recovery attempts failed
        if self.config.enable_circuit_breaker {
            if let Some(circuit_breaker) = self.circuit_breakers.get_mut(operation_key) {
                circuit_breaker.record_failure();
            }
        }
        
        let final_error = RecoveryError::AllAttemptsFailed {
            attempts: recovery_attempts.len(),
            last_error: error.clone(),
            recovery_history: recovery_attempts.clone(),
        };
        
        self.logger.error(&format!("All recovery attempts failed after {} tries", recovery_attempts.len()))
            .correlation_id(&correlation_id)
            .field("total_attempts", recovery_attempts.len())
            .field("total_duration_ms", recovery_start.elapsed().as_millis())
            .log().await;
        
        self.error_logger.log_recovery_failure(
            error.category(),
            "All strategies exhausted",
            Some(error),
            Some(&correlation_id)
        ).await;
        
        // Store recovery history
        self.recovery_history.extend(recovery_attempts);
        
        Err(final_error)
    }
    
    /// Generate appropriate recovery strategies for an error
    fn generate_recovery_strategies(&self, error: &AppError) -> Vec<RecoveryStrategy> {
        let mut strategies = Vec::new();
        
        match error {
            AppError::Network(_) => {
                // Network errors: retry with backoff, then try DNS fallback
                strategies.push(RecoveryStrategy::Retry { delay: self.config.initial_delay });
                strategies.push(RecoveryStrategy::ExponentialBackoff { 
                    base_delay: self.config.initial_delay, 
                    multiplier: self.config.backoff_multiplier 
                });
                
                if self.config.enable_dns_fallback && !self.config.fallback_dns_servers.is_empty() {
                    for dns_server in &self.config.fallback_dns_servers {
                        strategies.push(RecoveryStrategy::DnsFallback { 
                            fallback_server: dns_server.clone() 
                        });
                    }
                }
            }
            
            AppError::DnsResolution(_) => {
                // DNS errors: try fallback DNS servers
                if self.config.enable_dns_fallback {
                    for dns_server in &self.config.fallback_dns_servers {
                        strategies.push(RecoveryStrategy::DnsFallback { 
                            fallback_server: dns_server.clone() 
                        });
                    }
                }
                // Also try with increased timeout
                if self.config.enable_adaptive_timeout {
                    strategies.push(RecoveryStrategy::TimeoutAdjustment { 
                        new_timeout: Duration::from_secs(30) 
                    });
                }
            }
            
            AppError::HttpRequest(_) => {
                // HTTP errors: retry with backoff, then timeout adjustment
                strategies.push(RecoveryStrategy::ExponentialBackoff { 
                    base_delay: self.config.initial_delay, 
                    multiplier: self.config.backoff_multiplier 
                });
                
                if self.config.enable_adaptive_timeout {
                    strategies.push(RecoveryStrategy::TimeoutAdjustment { 
                        new_timeout: Duration::from_secs(20) 
                    });
                }
            }
            
            AppError::Timeout(_) => {
                // Timeout errors: increase timeout and retry
                if self.config.enable_adaptive_timeout {
                    strategies.push(RecoveryStrategy::TimeoutAdjustment { 
                        new_timeout: Duration::from_secs(30) 
                    });
                    strategies.push(RecoveryStrategy::TimeoutAdjustment { 
                        new_timeout: Duration::from_secs(60) 
                    });
                }
                // Also try with reduced load
                strategies.push(RecoveryStrategy::Retry { delay: Duration::from_secs(2) });
            }
            
            AppError::Validation(_) => {
                // Validation errors: try URL correction
                if self.config.enable_url_validation {
                    strategies.push(RecoveryStrategy::UrlValidation { corrected_url: None });
                }
            }
            
            _ => {
                // For other errors, just try simple retry
                strategies.push(RecoveryStrategy::Retry { delay: self.config.initial_delay });
            }
        }
        
        // Limit strategies to max attempts
        strategies.truncate(self.config.max_attempts);
        
        strategies
    }
    
    /// Apply a specific recovery strategy
    async fn apply_recovery_strategy(&self, strategy: &RecoveryStrategy, correlation_id: &str) {
        match strategy {
            RecoveryStrategy::Retry { delay } => {
                self.logger.debug(&format!("Applying retry strategy with {}ms delay", delay.as_millis()))
                    .correlation_id(correlation_id)
                    .field("delay_ms", delay.as_millis())
                    .log().await;
                // Delay is applied in the main recovery loop
            }
            
            RecoveryStrategy::ExponentialBackoff { base_delay, multiplier } => {
                self.logger.debug(&format!("Applying exponential backoff: base={}ms, multiplier={}", 
                    base_delay.as_millis(), multiplier))
                    .correlation_id(correlation_id)
                    .field("base_delay_ms", base_delay.as_millis())
                    .field("multiplier", *multiplier)
                    .log().await;
            }
            
            RecoveryStrategy::DnsFallback { fallback_server } => {
                self.logger.info(&format!("Switching to fallback DNS server: {}", fallback_server))
                    .correlation_id(correlation_id)
                    .field("fallback_dns", fallback_server)
                    .log().await;
                // DNS switching would be implemented by the caller
            }
            
            RecoveryStrategy::TimeoutAdjustment { new_timeout } => {
                self.logger.info(&format!("Adjusting timeout to {}s", new_timeout.as_secs()))
                    .correlation_id(correlation_id)
                    .field("new_timeout_seconds", new_timeout.as_secs())
                    .log().await;
                // Timeout adjustment would be implemented by the caller
            }
            
            RecoveryStrategy::UrlValidation { corrected_url } => {
                if let Some(url) = corrected_url {
                    self.logger.info(&format!("Using corrected URL: {}", url))
                        .correlation_id(correlation_id)
                        .field("corrected_url", url)
                        .log().await;
                } else {
                    self.logger.debug("Attempting URL format correction")
                        .correlation_id(correlation_id)
                        .log().await;
                }
            }
            
            RecoveryStrategy::CircuitBreakerReset => {
                self.logger.info("Attempting circuit breaker reset")
                    .correlation_id(correlation_id)
                    .log().await;
            }
            
            RecoveryStrategy::Combined { strategies } => {
                self.logger.info(&format!("Applying combined strategy with {} sub-strategies", strategies.len()))
                    .correlation_id(correlation_id)
                    .field("sub_strategies_count", strategies.len())
                    .log().await;
                
                for sub_strategy in strategies {
                    Box::pin(self.apply_recovery_strategy(sub_strategy, correlation_id)).await;
                }
            }
        }
    }
    
    /// Get recovery statistics
    pub fn get_recovery_stats(&self) -> RecoveryStats {
        let total_attempts = self.recovery_history.len();
        let successful_recoveries = self.recovery_history.iter()
            .filter(|attempt| attempt.result.is_ok())
            .count();
        
        let mut strategy_success_rates = HashMap::new();
        for attempt in &self.recovery_history {
            let strategy_name = attempt.strategy.name().to_string();
            let entry = strategy_success_rates.entry(strategy_name).or_insert((0, 0));
            entry.1 += 1; // Total attempts
            if attempt.result.is_ok() {
                entry.0 += 1; // Successful attempts
            }
        }
        
        let average_recovery_time = if !self.recovery_history.is_empty() {
            let total_time: Duration = self.recovery_history.iter()
                .map(|attempt| attempt.duration)
                .sum();
            total_time / self.recovery_history.len() as u32
        } else {
            Duration::ZERO
        };
        
        RecoveryStats {
            total_recovery_attempts: total_attempts,
            successful_recoveries,
            recovery_success_rate: if total_attempts > 0 {
                (successful_recoveries as f64 / total_attempts as f64) * 100.0
            } else {
                0.0
            },
            strategy_success_rates,
            average_recovery_time,
            circuit_breaker_states: self.circuit_breakers.iter()
                .map(|(key, breaker)| (key.clone(), breaker.state.clone()))
                .collect(),
        }
    }
    
    /// Clear recovery history (for testing or memory management)
    pub fn clear_history(&mut self) {
        self.recovery_history.clear();
    }
    
    /// Reset circuit breakers
    pub fn reset_circuit_breakers(&mut self) {
        self.circuit_breakers.clear();
    }
}

/// Recovery statistics for monitoring and analysis
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    /// Total number of recovery attempts made
    pub total_recovery_attempts: usize,
    /// Number of successful recoveries
    pub successful_recoveries: usize,
    /// Overall recovery success rate (0-100%)
    pub recovery_success_rate: f64,
    /// Success rate by strategy (strategy_name -> (successful, total))
    pub strategy_success_rates: HashMap<String, (usize, usize)>,
    /// Average time per recovery attempt
    pub average_recovery_time: Duration,
    /// Current circuit breaker states
    pub circuit_breaker_states: HashMap<String, CircuitBreakerState>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Config;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    #[test]
    fn test_recovery_config_default() {
        let config = RecoveryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.enable_circuit_breaker);
        assert!(config.enable_dns_fallback);
        assert!(!config.fallback_dns_servers.is_empty());
    }
    
    #[test]
    fn test_circuit_breaker_states() {
        let mut breaker = CircuitBreaker::new(3, Duration::from_secs(10));
        
        // Initially closed and can execute
        assert!(breaker.can_execute());
        assert_eq!(breaker.state, CircuitBreakerState::Closed);
        
        // Record failures to trip breaker
        breaker.record_failure();
        breaker.record_failure();
        assert!(breaker.can_execute()); // Still closed
        
        breaker.record_failure(); // Should open circuit
        assert!(!breaker.can_execute()); // Now open
        
        // Success should close circuit
        breaker.record_success();
        assert!(breaker.can_execute());
        assert_eq!(breaker.state, CircuitBreakerState::Closed);
    }
    
    #[test]
    fn test_recovery_strategy_naming() {
        let strategies = vec![
            RecoveryStrategy::Retry { delay: Duration::from_millis(100) },
            RecoveryStrategy::ExponentialBackoff { base_delay: Duration::from_millis(100), multiplier: 2.0 },
            RecoveryStrategy::DnsFallback { fallback_server: "8.8.8.8".to_string() },
            RecoveryStrategy::TimeoutAdjustment { new_timeout: Duration::from_secs(30) },
            RecoveryStrategy::UrlValidation { corrected_url: None },
            RecoveryStrategy::CircuitBreakerReset,
        ];
        
        let expected_names = ["retry", "exponential_backoff", "dns_fallback", "timeout_adjustment", "url_validation", "circuit_breaker_reset"];
        
        for (strategy, expected_name) in strategies.iter().zip(expected_names.iter()) {
            assert_eq!(strategy.name(), *expected_name);
        }
    }
    
    #[test]
    fn test_recovery_error_display() {
        let app_error = AppError::network("Connection failed");
        let recovery_error = RecoveryError::NotRecoverable(app_error.clone());
        
        let display = recovery_error.to_string();
        assert!(display.contains("not recoverable"));
        assert!(display.contains("Connection failed"));
        
        let all_failed = RecoveryError::AllAttemptsFailed {
            attempts: 3,
            last_error: app_error,
            recovery_history: vec![],
        };
        
        let display = all_failed.to_string();
        assert!(display.contains("All 3 recovery attempts failed"));
    }
    
    #[tokio::test]
    async fn test_error_recovery_manager_creation() {
        let config = Config::default();
        let recovery_config = RecoveryConfig::default();
        let manager = ErrorRecoveryManager::new(recovery_config, &config);
        
        assert_eq!(manager.config.max_attempts, 3);
        assert!(manager.circuit_breakers.is_empty());
        assert!(manager.recovery_history.is_empty());
    }
    
    #[test]
    fn test_generate_recovery_strategies() {
        let config = Config::default();
        let manager = ErrorRecoveryManager::with_defaults(&config);
        
        // Test network error strategies
        let network_error = AppError::network("Connection failed");
        let strategies = manager.generate_recovery_strategies(&network_error);
        assert!(!strategies.is_empty());
        assert!(strategies.iter().any(|s| matches!(s, RecoveryStrategy::Retry { .. })));
        
        // Test DNS error strategies
        let dns_error = AppError::dns_resolution("DNS failed");
        let strategies = manager.generate_recovery_strategies(&dns_error);
        assert!(!strategies.is_empty());
        assert!(strategies.iter().any(|s| matches!(s, RecoveryStrategy::DnsFallback { .. })));
        
        // Test timeout error strategies
        let timeout_error = AppError::timeout("Request timed out");
        let strategies = manager.generate_recovery_strategies(&timeout_error);
        assert!(!strategies.is_empty());
        assert!(strategies.iter().any(|s| matches!(s, RecoveryStrategy::TimeoutAdjustment { .. })));
    }
    
    #[tokio::test]
    async fn test_non_recoverable_error() {
        let config = Config::default();
        let mut manager = ErrorRecoveryManager::with_defaults(&config);
        
        let non_recoverable_error = AppError::config("Invalid configuration");
        assert!(!non_recoverable_error.is_recoverable());
        
        let result = manager.recover_from_error(
            &non_recoverable_error,
            "test_operation",
            || Box::pin(async { Ok(42) })
        ).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RecoveryError::NotRecoverable(_)));
    }
    
    #[tokio::test]
    async fn test_successful_recovery() {
        let config = Config::default();
        let mut manager = ErrorRecoveryManager::with_defaults(&config);
        
        let recoverable_error = AppError::network("Temporary network issue");
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();
        
        let result = manager.recover_from_error(
            &recoverable_error,
            "test_operation",
            move || {
                let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if count < 2 {
                        // Fail first two attempts
                        Err(AppError::network("Still failing"))
                    } else {
                        // Succeed on third attempt
                        Ok(42)
                    }
                })
            }
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        // Should have made 3 calls (2 failures + 1 success)
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
        
        let stats = manager.get_recovery_stats();
        assert!(stats.total_recovery_attempts > 0);
        assert!(stats.successful_recoveries > 0);
        assert!(stats.recovery_success_rate > 0.0);
    }
    
    #[tokio::test]
    async fn test_recovery_timeout() {
        let config = Config::default();
        let recovery_config = RecoveryConfig {
            max_recovery_time: Duration::from_millis(10), // Very short timeout
            ..Default::default()
        };
        let mut manager = ErrorRecoveryManager::new(recovery_config, &config);
        
        let recoverable_error = AppError::network("Network issue");
        
        let result: Result<(), RecoveryError> = manager.recover_from_error(
            &recoverable_error,
            "test_operation",
            || Box::pin(async {
                // Always fail
                tokio::time::sleep(Duration::from_millis(20)).await;
                Err(AppError::network("Still failing"))
            })
        ).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RecoveryError::RecoveryTimeout { .. }));
    }
    
    #[tokio::test]
    async fn test_recovery_stats() {
        let config = Config::default();
        let mut manager = ErrorRecoveryManager::with_defaults(&config);
        
        // Perform some recovery attempts
        let error = AppError::network("Test error");
        let _ = manager.recover_from_error(
            &error,
            "test_op",
            || Box::pin(async { Ok(()) })
        ).await;
        
        let stats = manager.get_recovery_stats();
        assert!(stats.total_recovery_attempts > 0);
    }
    
    #[test]
    fn test_recovery_attempt_structure() {
        let attempt = RecoveryAttempt {
            attempt_number: 1,
            strategy: RecoveryStrategy::Retry { delay: Duration::from_millis(100) },
            timestamp: Instant::now(),
            duration: Duration::from_millis(50),
            result: Ok(()),
            context: HashMap::new(),
        };
        
        assert_eq!(attempt.attempt_number, 1);
        assert_eq!(attempt.strategy.name(), "retry");
        assert!(attempt.result.is_ok());
        assert!(attempt.duration.as_millis() > 0);
    }
}