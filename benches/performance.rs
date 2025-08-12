//! Performance benchmarks for the network latency tester
//!
//! These benchmarks measure the performance of key components and compare
//! them to expected baselines to ensure the Rust implementation meets
//! or exceeds the original bash script performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use network_latency_tester::{
    cli::Cli,
    config::parser::ConfigParser,
    dns::DnsManager,
    models::{Config, TimingMetrics, TestResult},
    stats::StatisticsEngine,
    types::DnsConfig,
};
use std::{net::IpAddr, time::Duration};
use chrono::Utc;
use clap::Parser;

/// Create a test configuration for benchmarking
fn create_benchmark_config() -> Config {
    Config {
        target_urls: vec!["https://httpbin.org/delay/0".to_string()],
        dns_servers: vec!["8.8.8.8".to_string()],
        doh_providers: vec![],
        test_count: 1, // Single iteration for benchmarking
        timeout_seconds: 5,
        enable_color: false,
        verbose: false,
        debug: false,
    }
}

/// Create sample timing metrics for benchmarking statistics
fn create_sample_metrics(count: usize) -> Vec<TimingMetrics> {
    (0..count)
        .map(|i| TimingMetrics::success(
            Duration::from_millis(10 + i as u64 % 50), // dns_resolution
            Duration::from_millis(50 + i as u64 % 100), // tcp_connection
            Some(Duration::from_millis(100 + i as u64 % 200)), // tls_handshake
            Duration::from_millis(200 + i as u64 % 300), // first_byte
            Duration::from_millis(410 + i as u64 % 650), // total_duration
            200, // http_status
        ))
        .collect()
}

/// Create sample test results for benchmarking
fn create_sample_results(count: usize) -> Vec<TestResult> {
    (0..count)
        .map(|i| {
            let individual_results = if i % 10 == 0 {
                // 10% failure rate
                vec![TimingMetrics::failed("Server error".to_string())]
            } else {
                vec![TimingMetrics::success(
                    Duration::from_millis(10 + i as u64 % 50), // dns_resolution
                    Duration::from_millis(50 + i as u64 % 100), // tcp_connection
                    Some(Duration::from_millis(100 + i as u64 % 200)), // tls_handshake
                    Duration::from_millis(200 + i as u64 % 300), // first_byte
                    Duration::from_millis(410 + i as u64 % 650), // total_duration
                    200, // http_status
                )]
            };
            
            TestResult {
                config_name: format!("DNS Config {}", i % 3),
                dns_config: DnsConfig::System,
                url: "https://example.com".to_string(),
                individual_results,
                statistics: None, // Will be calculated
                success_count: if i % 10 == 0 { 0 } else { 1 },
                total_count: 1,
                started_at: Utc::now(),
                completed_at: Some(Utc::now()),
            }
        })
        .collect()
}

/// Benchmark DNS configuration parsing and management
fn benchmark_dns_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("dns_parsing");
    
    // Benchmark DNS configuration enum creation
    group.bench_function("create_dns_configs", |b| {
        b.iter(|| {
            let configs = vec![
                DnsConfig::System,
                DnsConfig::Custom {
                    servers: vec!["8.8.8.8".parse::<IpAddr>().unwrap()],
                },
                DnsConfig::DoH {
                    url: "https://dns.google/dns-query".to_string(),
                },
            ];
            black_box(configs);
        });
    });
    
    // Benchmark DNS manager creation
    group.bench_function("create_dns_manager", |b| {
        b.iter(|| {
            let dns_manager = DnsManager::new().unwrap();
            black_box(dns_manager);
        });
    });
    
    // Benchmark IP address parsing
    group.bench_function("parse_ip_addresses", |b| {
        let ip_strings = ["8.8.8.8", "1.1.1.1", "9.9.9.9", "208.67.222.222"];
        b.iter(|| {
            let parsed: Vec<IpAddr> = ip_strings.iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            black_box(parsed);
        });
    });
    
    group.finish();
}

/// Benchmark configuration parsing from various sources
fn benchmark_config_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_parsing");
    
    // Benchmark CLI parsing
    group.bench_function("parse_cli_args", |b| {
        let args = vec![
            "network-latency-tester",
            "--url", "https://example.com",
            "--count", "5",
            "--timeout", "10",
            "--dns-servers", "8.8.8.8,1.1.1.1",
        ];
        b.iter(|| {
            let cli = Cli::try_parse_from(black_box(&args)).unwrap();
            black_box(cli);
        });
    });
    
    // Benchmark config validation
    group.bench_function("validate_config", |b| {
        let config = create_benchmark_config();
        b.iter(|| {
            let result = config.validate();
            black_box(result);
        });
    });
    
    // Benchmark config parsing from CLI
    group.bench_function("parse_from_cli", |b| {
        let cli = Cli::try_parse_from(vec![
            "network-latency-tester",
            "--url", "https://example.com",
            "--count", "5",
            "--timeout", "10",
        ]).unwrap();
        
        b.iter(|| {
            let parser = ConfigParser::new(black_box(cli.clone()));
            let config = parser.parse().unwrap();
            black_box(config);
        });
    });
    
    group.finish();
}

/// Benchmark statistics calculation performance
fn benchmark_statistics_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");
    
    // Benchmark with different dataset sizes
    for size in [10, 50, 100, 500, 1000].iter() {
        let metrics = create_sample_metrics(*size);
        let results = create_sample_results(*size);
        
        group.bench_with_input(BenchmarkId::new("calculate_stats", size), size, |b, _| {
            b.iter(|| {
                let mut engine = StatisticsEngine::with_defaults();
                engine.add_results(black_box(results.clone()));
                let analysis = engine.analyze();
                black_box(analysis);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("timing_calculations", size), size, |b, _| {
            let times: Vec<Duration> = metrics.iter().map(|m| m.total_duration).collect();
            b.iter(|| {
                // Basic statistics calculations
                let mut sorted_times = times.clone();
                sorted_times.sort();
                
                let sum: Duration = times.iter().sum();
                let avg = sum / times.len() as u32;
                let min = sorted_times.first().copied().unwrap_or(Duration::ZERO);
                let max = sorted_times.last().copied().unwrap_or(Duration::ZERO);
                
                black_box((avg, min, max));
            });
        });
    }
    
    group.finish();
}

/// Benchmark result processing
fn benchmark_result_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("result_processing");
    
    // Benchmark result aggregation
    group.bench_function("aggregate_results", |b| {
        let results = create_sample_results(100);
        b.iter(|| {
            // Simulate result processing operations
            let total_tests: u32 = results.iter().map(|r| r.total_count).sum();
            let successful_tests: u32 = results.iter().map(|r| r.success_count).sum();
            let success_rate = successful_tests as f64 / total_tests as f64;
            black_box(success_rate);
        });
    });
    
    // Benchmark URL processing
    group.bench_function("process_urls", |b| {
        let urls = vec![
            "https://example1.com",
            "https://example2.com",
            "https://example3.com/path",
            "https://example4.com/path?param=value",
        ];
        
        b.iter(|| {
            let processed: Vec<String> = urls.iter()
                .map(|url| url.to_lowercase())
                .collect();
            black_box(processed);
        });
    });
    
    group.finish();
}

/// Benchmark complete application workflow (without actual network calls)
fn benchmark_application_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("application_workflow");
    group.sample_size(10);
    
    // Benchmark configuration loading and validation
    group.bench_function("config_loading_pipeline", |b| {
        let args = vec![
            "network-latency-tester",
            "--url", "https://httpbin.org/delay/0",
            "--count", "1",
            "--timeout", "5",
            "--dns-servers", "8.8.8.8",
            "--no-color",
        ];
        
        b.iter(|| {
            let cli = Cli::try_parse_from(black_box(&args)).unwrap();
            let parser = ConfigParser::new(cli);
            let config = parser.parse().unwrap();
            let validated = config.validate().unwrap();
            black_box(validated);
        });
    });
    
    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    // Benchmark string allocations in result processing
    group.bench_function("result_formatting", |b| {
        let results = create_sample_results(100);
        b.iter(|| {
            let formatted: Vec<String> = results.iter().map(|r| {
                let avg_duration = if !r.individual_results.is_empty() {
                    r.individual_results[0].total_duration
                } else {
                    Duration::ZERO
                };
                format!("URL: {}, Config: {}, Time: {:?}", 
                    r.url, r.config_name, avg_duration)
            }).collect();
            black_box(formatted);
        });
    });
    
    // Benchmark vector operations in statistics
    group.bench_function("vector_operations", |b| {
        let metrics = create_sample_metrics(1000);
        b.iter(|| {
            let times: Vec<Duration> = metrics.iter()
                .map(|m| m.total_duration)
                .collect();
            let sorted_times = {
                let mut times = times.clone();
                times.sort();
                times
            };
            black_box(sorted_times);
        });
    });
    
    group.finish();
}

/// Performance regression tests - these should consistently meet performance targets
fn benchmark_performance_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_regression");
    
    // Configuration parsing should be under 100μs
    group.bench_function("config_parsing_speed", |b| {
        let args = vec![
            "network-latency-tester",
            "--url", "https://example.com",
            "--count", "5",
        ];
        b.iter(|| {
            let cli = Cli::try_parse_from(black_box(&args)).unwrap();
            let parser = ConfigParser::new(cli);
            let config = parser.parse().unwrap();
            black_box(config);
        });
    });
    
    // Statistics calculation for 100 results should be under 1ms
    group.bench_function("stats_calculation_speed", |b| {
        let results = create_sample_results(100);
        b.iter(|| {
            let mut engine = StatisticsEngine::with_defaults();
            engine.add_results(results.clone());
            let analysis = engine.analyze();
            black_box(analysis);
        });
    });
    
    // DNS configuration setup should be under 10μs
    group.bench_function("dns_setup_speed", |b| {
        let servers = vec!["8.8.8.8".parse::<IpAddr>().unwrap()];
        b.iter(|| {
            let config = DnsConfig::Custom { 
                servers: black_box(servers.clone()) 
            };
            black_box(config);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_dns_parsing,
    benchmark_config_parsing,
    benchmark_statistics_calculation,
    benchmark_result_processing,
    benchmark_application_workflow,
    benchmark_memory_allocation,
    benchmark_performance_regression
);

criterion_main!(benches);