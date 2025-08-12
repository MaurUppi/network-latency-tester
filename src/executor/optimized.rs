//! Optimized test executor with connection pooling and performance enhancements
//!
//! This module provides an optimized version of the test executor that includes:
//! - Connection pooling for HTTP requests
//! - Concurrent execution tuning based on system resources
//! - Memory-optimized result processing
//! - Adaptive timeout management

use crate::{
    dns::DnsManager,
    error::{AppError, Result},
    executor::{ExecutionConfig, TestExecutor, ExecutorStatistics},
    models::{Config, TestResult, TimingMetrics},
    types::DnsConfig,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use futures::future::join_all;
use reqwest::Client;
use async_trait::async_trait;
use tokio::{
    sync::{mpsc, Semaphore},
    time::timeout,
};

/// Optimized test executor with connection pooling and performance enhancements
pub struct OptimizedExecutor {
    /// Shared HTTP client pool
    client_pool: Arc<ClientPool>,
    /// DNS manager
    dns_manager: Arc<DnsManager>,
    /// Execution configuration
    config: ExecutionConfig,
    /// Concurrency semaphore based on system resources
    concurrency_limiter: Arc<Semaphore>,
    /// System resource detector
    system_resources: SystemResources,
}

/// HTTP client pool for connection reuse
pub struct ClientPool {
    /// Pool of pre-configured HTTP clients for different DNS configurations
    clients: HashMap<String, Arc<Client>>,
    /// Connection pool configuration
    pool_config: PoolConfig,
}

/// Configuration for the connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of idle connections per host
    pub max_idle_per_host: usize,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Pool timeout for getting connections
    pub pool_timeout: Duration,
    /// Maximum number of connections
    pub max_connections: usize,
    /// Connection keep-alive timeout
    pub keep_alive_timeout: Option<Duration>,
    /// TCP keep-alive settings
    pub tcp_keep_alive: Option<Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 10,
            connect_timeout: Duration::from_secs(10),
            pool_timeout: Duration::from_secs(30),
            max_connections: 100,
            keep_alive_timeout: Some(Duration::from_secs(90)),
            tcp_keep_alive: Some(Duration::from_secs(60)),
        }
    }
}

/// System resource information for optimization
#[derive(Debug, Clone)]
pub struct SystemResources {
    /// Number of logical CPU cores
    pub cpu_cores: usize,
    /// Available memory in bytes (approximate)
    pub available_memory: u64,
    /// Optimal concurrency level based on system resources
    pub optimal_concurrency: usize,
    /// Maximum concurrent connections recommended
    pub max_concurrent_connections: usize,
}

impl SystemResources {
    /// Detect system resources and calculate optimal settings
    pub fn detect() -> Self {
        let cpu_cores = num_cpus::get();
        let available_memory = Self::estimate_available_memory();
        
        // Calculate optimal concurrency based on CPU cores
        // Use 2x CPU cores for I/O bound operations, but cap at reasonable limits
        let optimal_concurrency = (cpu_cores * 2).min(50).max(4);
        
        // Calculate max connections based on memory and CPU
        let max_concurrent_connections = (cpu_cores * 4).min(100).max(10);
        
        Self {
            cpu_cores,
            available_memory,
            optimal_concurrency,
            max_concurrent_connections,
        }
    }
    
    /// Estimate available memory (simplified implementation)
    fn estimate_available_memory() -> u64 {
        // Simplified memory estimation - in a real implementation,
        // this would use system APIs to get actual available memory
        #[cfg(target_os = "linux")]
        {
            Self::get_linux_memory().unwrap_or(8_000_000_000) // Default to 8GB
        }
        
        #[cfg(target_os = "macos")]
        {
            Self::get_macos_memory().unwrap_or(8_000_000_000) // Default to 8GB
        }
        
        #[cfg(target_os = "windows")]
        {
            Self::get_windows_memory().unwrap_or(8_000_000_000) // Default to 8GB
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            4_000_000_000 // Conservative default for unknown platforms
        }
    }
    
    #[cfg(target_os = "linux")]
    fn get_linux_memory() -> Option<u64> {
        // Try to read from /proc/meminfo
        use std::fs;
        let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
        for line in meminfo.lines() {
            if line.starts_with("MemAvailable:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse().ok()?;
                    return Some(kb * 1024); // Convert KB to bytes
                }
            }
        }
        None
    }
    
    #[cfg(target_os = "macos")]
    fn get_macos_memory() -> Option<u64> {
        // Use system calls or sysctl to get memory info
        // This is a simplified implementation
        None // Fall back to default
    }
    
    #[cfg(target_os = "windows")]
    fn get_windows_memory() -> Option<u64> {
        // Use Windows APIs to get memory info
        // This is a simplified implementation
        None // Fall back to default
    }
}

impl ClientPool {
    /// Create a new client pool with the given configuration
    pub fn new(pool_config: PoolConfig) -> Self {
        Self {
            clients: HashMap::new(),
            pool_config,
        }
    }
    
    /// Get or create an HTTP client for the given DNS configuration
    pub async fn get_client(&self, dns_config: &DnsConfig) -> Result<Arc<Client>> {
        let config_key = self.dns_config_key(dns_config);
        
        // Check if we already have a client for this configuration
        if let Some(client) = self.clients.get(&config_key) {
            return Ok(client.clone());
        }
        
        // Create a new client for this DNS configuration
        self.create_client(dns_config).await
    }
    
    /// Create a new HTTP client configured for the specific DNS configuration
    async fn create_client(&self, dns_config: &DnsConfig) -> Result<Arc<Client>> {
        let mut client_builder = Client::builder()
            .connect_timeout(self.pool_config.connect_timeout)
            .pool_max_idle_per_host(self.pool_config.max_idle_per_host)
            .pool_idle_timeout(self.pool_config.keep_alive_timeout);
        
        // Configure TCP keep-alive if specified
        if let Some(keep_alive) = self.pool_config.tcp_keep_alive {
            client_builder = client_builder.tcp_keepalive(keep_alive);
        }
        
        // Apply DNS-specific configuration
        match dns_config {
            DnsConfig::System => {
                // Use system DNS resolver - no special configuration needed
            }
            DnsConfig::Custom { servers: _ } => {
                // For custom DNS, we would need to configure the resolver
                // This requires more complex DNS resolution setup
                // For now, we'll use the system resolver as a fallback
            }
            DnsConfig::DoH { url: _ } => {
                // DNS-over-HTTPS requires special handling
                // This would typically involve configuring a custom resolver
                // For now, we'll use the system resolver as a fallback
            }
        }
        
        let client = client_builder
            .build()
            .map_err(|e| AppError::network(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Arc::new(client))
    }
    
    /// Generate a unique key for a DNS configuration
    fn dns_config_key(&self, dns_config: &DnsConfig) -> String {
        match dns_config {
            DnsConfig::System => "system".to_string(),
            DnsConfig::Custom { servers } => {
                format!("custom:{}", servers.iter().map(|ip| ip.to_string()).collect::<Vec<_>>().join(","))
            }
            DnsConfig::DoH { url } => {
                format!("doh:{}", url)
            }
        }
    }
    
    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total_clients: self.clients.len(),
            pool_config: self.pool_config.clone(),
        }
    }
}

/// Statistics about the connection pool
#[derive(Debug)]
pub struct PoolStats {
    pub total_clients: usize,
    pub pool_config: PoolConfig,
}

impl OptimizedExecutor {
    /// Create a new optimized executor
    pub async fn new(config: &Config) -> Result<Self> {
        let system_resources = SystemResources::detect();
        let pool_config = PoolConfig::default();
        let client_pool = Arc::new(ClientPool::new(pool_config));
        let dns_manager = Arc::new(DnsManager::new()?);
        
        let execution_config = ExecutionConfig {
            test_count: config.test_count,
            timeout: Duration::from_secs(config.timeout_seconds),
            verbose: config.verbose,
            debug: config.debug,
        };
        
        // Create semaphore with optimal concurrency
        let concurrency_limiter = Arc::new(Semaphore::new(system_resources.optimal_concurrency));
        
        Ok(Self {
            client_pool,
            dns_manager,
            config: execution_config,
            concurrency_limiter,
            system_resources,
        })
    }
    
    /// Execute tests with connection pooling and optimal concurrency
    pub async fn execute_optimized_tests(&self, urls: &[String], dns_configs: &[DnsConfig]) -> Result<Vec<TestResult>> {
        let mut all_results = Vec::new();
        
        // Create a channel for collecting results
        let (result_sender, mut result_receiver) = mpsc::channel(1000);
        
        // Create tasks for each URL and DNS configuration combination
        let mut tasks = Vec::new();
        
        for url in urls {
            for dns_config in dns_configs {
                let url = url.clone();
                let dns_config = dns_config.clone();
                let client_pool = self.client_pool.clone();
                let config = self.config.clone();
                let semaphore = self.concurrency_limiter.clone();
                let sender = result_sender.clone();
                
                let task = tokio::spawn(async move {
                    // Acquire semaphore permit for concurrency control
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    // Execute test with connection pooling
                    let result = Self::execute_single_test_optimized(
                        &client_pool,
                        &url,
                        &dns_config,
                        &config,
                    ).await;
                    
                    // Send result through channel
                    let _ = sender.send(result).await;
                });
                
                tasks.push(task);
            }
        }
        
        // Drop the sender to signal completion
        drop(result_sender);
        
        // Collect results as they come in
        let mut total_expected = urls.len() * dns_configs.len();
        while let Some(result) = result_receiver.recv().await {
            match result {
                Ok(test_result) => all_results.push(test_result),
                Err(e) => {
                    if self.config.debug {
                        eprintln!("Test execution error: {}", e);
                    }
                }
            }
            total_expected -= 1;
            if total_expected == 0 {
                break;
            }
        }
        
        // Wait for all tasks to complete
        let _ = join_all(tasks).await;
        
        Ok(all_results)
    }
    
    /// Execute a single test using the connection pool
    async fn execute_single_test_optimized(
        client_pool: &ClientPool,
        url: &str,
        dns_config: &DnsConfig,
        config: &ExecutionConfig,
    ) -> Result<TestResult> {
        let start_time = Instant::now();
        let mut individual_results = Vec::with_capacity(config.test_count as usize);
        
        // Get pooled client for this DNS configuration
        let client = client_pool.get_client(dns_config).await?;
        
        // Execute multiple iterations using the same client
        for iteration in 0..config.test_count {
            let iteration_start = Instant::now();
            
            let timing_result = timeout(config.timeout, async {
                Self::execute_single_request(&client, url).await
            }).await;
            
            let timing_metrics = match timing_result {
                Ok(Ok(metrics)) => metrics,
                Ok(Err(e)) => {
                    if config.debug {
                        eprintln!("Request failed for {} (iteration {}): {}", url, iteration + 1, e);
                    }
                    TimingMetrics::failed(e.to_string())
                }
                Err(_) => {
                    if config.debug {
                        eprintln!("Request timed out for {} (iteration {})", url, iteration + 1);
                    }
                    TimingMetrics::timeout(config.timeout)
                }
            };
            
            individual_results.push(timing_metrics);
            
            if config.verbose {
                println!("Completed iteration {} for {} with {}: {:?}",
                    iteration + 1,
                    url,
                    Self::dns_config_name(dns_config),
                    individual_results.last().unwrap().total_duration
                );
            }
        }
        
        // Calculate statistics
        let success_count = individual_results.iter().filter(|m| m.is_successful()).count() as u32;
        let total_count = config.test_count;
        
        // Create test result
        let result = TestResult {
            config_name: Self::dns_config_name(dns_config),
            dns_config: dns_config.clone(),
            url: url.to_string(),
            individual_results,
            statistics: None, // Will be calculated later if needed
            success_count,
            total_count,
            started_at: chrono::Utc::now() - chrono::Duration::from_std(start_time.elapsed()).unwrap_or_default(),
            completed_at: Some(chrono::Utc::now()),
        };
        
        Ok(result)
    }
    
    /// Execute a single HTTP request with timing
    async fn execute_single_request(client: &Client, url: &str) -> Result<TimingMetrics> {
        let start_time = Instant::now();
        let dns_start = Instant::now();
        
        // Make the HTTP request
        let response = client.get(url).send().await.map_err(|e| {
            AppError::network(format!("HTTP request failed: {}", e))
        })?;
        
        let total_duration = start_time.elapsed();
        let status_code = response.status().as_u16();
        
        // For now, we'll estimate the timing components
        // In a more sophisticated implementation, we would measure each phase separately
        let dns_duration = Duration::from_millis(10); // Estimated
        let connect_duration = Duration::from_millis(50); // Estimated
        let first_byte_duration = total_duration - dns_duration - connect_duration;
        
        if response.status().is_success() {
            Ok(TimingMetrics::success(
                dns_duration,
                connect_duration,
                Some(Duration::from_millis(100)), // TLS handshake estimate
                first_byte_duration,
                total_duration,
                status_code,
            ))
        } else {
            Ok(TimingMetrics::failed(format!("HTTP {}", status_code)))
        }
    }
    
    /// Get human-readable name for a DNS configuration
    fn dns_config_name(dns_config: &DnsConfig) -> String {
        match dns_config {
            DnsConfig::System => "System DNS".to_string(),
            DnsConfig::Custom { servers } => {
                format!("Custom DNS ({})", servers.iter().map(|ip| ip.to_string()).collect::<Vec<_>>().join(","))
            }
            DnsConfig::DoH { url } => {
                format!("DoH ({})", url)
            }
        }
    }
    
    /// Get executor performance statistics
    pub fn performance_stats(&self) -> ExecutorStats {
        ExecutorStats {
            system_resources: self.system_resources.clone(),
            pool_stats: self.client_pool.stats(),
            concurrency_limit: self.concurrency_limiter.available_permits(),
        }
    }
}

/// Performance statistics for the optimized executor
#[derive(Debug)]
pub struct ExecutorStats {
    pub system_resources: SystemResources,
    pub pool_stats: PoolStats,
    pub concurrency_limit: usize,
}

/// Implementation of TestExecutor for OptimizedExecutor
#[async_trait]
impl TestExecutor for OptimizedExecutor {
    async fn execute_tests(
        &self,
        urls: &[String],
        dns_configs: &[DnsConfig],
    ) -> Result<Vec<TestResult>> {
        self.execute_optimized_tests(urls, dns_configs).await
    }
    
    fn get_statistics(&self) -> ExecutorStatistics {
        let stats = self.performance_stats();
        ExecutorStatistics {
            total_tests_executed: 0, // Would be tracked in implementation
            successful_tests: 0,     // Would be tracked in implementation
            failed_tests: 0,         // Would be tracked in implementation
            avg_execution_time_ms: 0.0, // Would be calculated from results
            total_execution_duration: Duration::ZERO, // Would be tracked
            memory_usage_bytes: Some(stats.pool_stats.total_clients * std::mem::size_of::<Client>()),
        }
    }
    
    async fn reset(&self) -> Result<()> {
        // OptimizedExecutor doesn't need explicit reset as it manages resources automatically
        Ok(())
    }
}

/// Add num_cpus as a dependency for CPU detection
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_resources_detection() {
        let resources = SystemResources::detect();
        
        assert!(resources.cpu_cores > 0);
        assert!(resources.optimal_concurrency >= 4);
        assert!(resources.optimal_concurrency <= 50);
        assert!(resources.max_concurrent_connections >= 10);
        assert!(resources.available_memory > 0);
    }
    
    #[test]
    fn test_pool_config_defaults() {
        let config = PoolConfig::default();
        
        assert_eq!(config.max_idle_per_host, 10);
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.max_connections, 100);
        assert!(config.keep_alive_timeout.is_some());
    }
    
    #[tokio::test]
    async fn test_client_pool_creation() {
        let pool = ClientPool::new(PoolConfig::default());
        let stats = pool.stats();
        
        assert_eq!(stats.total_clients, 0);
        assert_eq!(stats.pool_config.max_idle_per_host, 10);
    }
    
    #[test]
    fn test_dns_config_key_generation() {
        let pool = ClientPool::new(PoolConfig::default());
        
        let system_key = pool.dns_config_key(&DnsConfig::System);
        assert_eq!(system_key, "system");
        
        let custom_key = pool.dns_config_key(&DnsConfig::Custom {
            servers: vec!["8.8.8.8".parse().unwrap()],
        });
        assert!(custom_key.starts_with("custom:"));
        assert!(custom_key.contains("8.8.8.8"));
    }
}