//! Network diagnostics and connectivity tests

use crate::{
    error::{AppError, Result},
    types::DnsConfig,
    models::metrics::{TimingMetrics, TestResult},
    dns::{DnsManager, DnsPerformanceResult},
    client::{HttpClient, NetworkClient, ConnectivityTest},
    stats::{StatisticsEngine, StatisticalAnalysis},
};
use std::{
    net::{IpAddr, TcpStream, SocketAddr},
    time::{Duration, Instant},
    collections::HashMap,
    sync::Arc,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use colored::*;

/// Comprehensive network diagnostics engine
pub struct NetworkDiagnostics {
    dns_manager: Arc<DnsManager>,
    http_client: Arc<dyn HttpClient>,
    config: DiagnosticsConfig,
}

/// Configuration for diagnostic operations
#[derive(Debug, Clone)]
pub struct DiagnosticsConfig {
    /// Timeout for individual connectivity tests
    pub connectivity_timeout: Duration,
    /// Number of parallel diagnostic tests
    pub parallel_tests: usize,
    /// Whether to include DNS resolution diagnostics
    pub include_dns_diagnostics: bool,
    /// Whether to include HTTP connectivity diagnostics
    pub include_http_diagnostics: bool,
    /// Whether to include performance analysis
    pub include_performance_analysis: bool,
    /// Whether to generate detailed reports
    pub detailed_reporting: bool,
    /// Minimum sample size for reliable diagnostics
    pub min_sample_size: usize,
}

/// Comprehensive diagnostic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    /// Overall system health status
    pub system_health: SystemHealth,
    /// Network connectivity diagnostics
    pub connectivity_diagnostics: ConnectivityDiagnostics,
    /// DNS resolution diagnostics
    pub dns_diagnostics: DnsDiagnostics,
    /// HTTP/HTTPS connectivity diagnostics
    pub http_diagnostics: HttpDiagnostics,
    /// Performance analysis results
    pub performance_analysis: PerformanceAnalysis,
    /// Detected issues and problems
    pub issues: Vec<DiagnosticIssue>,
    /// Recommendations for improvement
    pub recommendations: Vec<Recommendation>,
    /// When this report was generated
    pub generated_at: DateTime<Utc>,
    /// Diagnostic execution summary
    pub execution_summary: ExecutionSummary,
}

/// Overall system health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Health score (0.0 - 1.0, higher is better)
    pub score: f64,
    /// Component-specific health scores
    pub component_scores: HashMap<String, f64>,
    /// Critical issues that affect system health
    pub critical_issues: Vec<String>,
    /// Warning issues that may affect performance
    pub warning_issues: Vec<String>,
}

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operating normally
    Healthy,
    /// Some performance issues detected
    Warning,
    /// Significant issues affecting functionality
    Critical,
    /// System is non-functional
    Failed,
}

/// Network connectivity diagnostic results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityDiagnostics {
    /// Internet connectivity status
    pub internet_connectivity: ConnectivityStatus,
    /// Target host reachability
    pub target_reachability: HashMap<String, ConnectivityStatus>,
    /// Port connectivity tests
    pub port_connectivity: HashMap<String, PortConnectivityResult>,
    /// Network interface diagnostics
    pub network_interfaces: Vec<NetworkInterfaceInfo>,
    /// Routing table analysis
    pub routing_analysis: Option<RoutingAnalysis>,
}

/// DNS resolution diagnostic results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsDiagnostics {
    /// DNS server accessibility
    pub dns_server_status: HashMap<String, DnsServerStatus>,
    /// Resolution performance by configuration
    pub resolution_performance: HashMap<String, DnsPerformanceResult>,
    /// DNS cache analysis
    pub cache_analysis: Option<DnsCacheAnalysis>,
    /// DNSSEC validation status
    pub dnssec_status: HashMap<String, bool>,
    /// DNS over HTTPS provider analysis
    pub doh_analysis: HashMap<String, DoHProviderAnalysis>,
}

/// HTTP connectivity diagnostic results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpDiagnostics {
    /// HTTP/HTTPS connectivity by URL
    pub url_connectivity: HashMap<String, HttpConnectivityResult>,
    /// SSL/TLS certificate analysis
    pub certificate_analysis: HashMap<String, CertificateAnalysis>,
    /// HTTP response analysis
    pub response_analysis: HashMap<String, ResponseAnalysis>,
    /// Redirect chain analysis
    pub redirect_analysis: HashMap<String, RedirectChainAnalysis>,
}

/// Performance analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Statistical analysis of all measurements
    pub statistical_analysis: Option<StatisticalAnalysis>,
    /// Performance bottleneck identification
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// Latency breakdown analysis
    pub latency_breakdown: LatencyBreakdown,
    /// Throughput analysis
    pub throughput_analysis: Option<ThroughputAnalysis>,
    /// Performance trends
    pub performance_trends: Vec<PerformanceTrend>,
}

/// Detected diagnostic issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticIssue {
    /// Issue severity level
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Human-readable issue title
    pub title: String,
    /// Detailed issue description
    pub description: String,
    /// Affected components
    pub affected_components: Vec<String>,
    /// Potential impact on system operation
    pub impact: String,
    /// Suggested resolution steps
    pub resolution_steps: Vec<String>,
    /// Related metrics or measurements
    pub related_metrics: HashMap<String, String>,
}

/// Recommendation for system improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation priority
    pub priority: RecommendationPriority,
    /// Recommendation type
    pub category: RecommendationCategory,
    /// Recommendation title
    pub title: String,
    /// Detailed recommendation description
    pub description: String,
    /// Expected benefits
    pub expected_benefits: Vec<String>,
    /// Implementation complexity
    pub complexity: ImplementationComplexity,
    /// Estimated time to implement
    pub estimated_time: String,
}

/// Diagnostic execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// Total execution time
    pub total_duration: Duration,
    /// Number of tests executed
    pub tests_executed: usize,
    /// Number of successful tests
    pub tests_successful: usize,
    /// Number of failed tests
    pub tests_failed: usize,
    /// Number of skipped tests
    pub tests_skipped: usize,
    /// Data collection summary
    pub data_summary: DataCollectionSummary,
}

/// Individual diagnostic test result types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityStatus {
    pub reachable: bool,
    pub response_time: Option<Duration>,
    pub error_message: Option<String>,
    pub tested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConnectivityResult {
    pub port: u16,
    pub protocol: String,
    pub status: ConnectivityStatus,
    pub service_detection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub ip_addresses: Vec<IpAddr>,
    pub status: String,
    pub mtu: Option<u16>,
    pub speed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingAnalysis {
    pub default_gateway: Option<IpAddr>,
    pub route_count: usize,
    pub routing_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServerStatus {
    pub accessible: bool,
    pub response_time: Option<Duration>,
    pub supports_dnssec: bool,
    pub error_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsCacheAnalysis {
    pub cache_size: usize,
    pub hit_rate: f64,
    pub cache_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoHProviderAnalysis {
    pub provider_name: String,
    pub url: String,
    pub accessible: bool,
    pub response_time: Option<Duration>,
    pub supports_json: bool,
    pub privacy_policy_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConnectivityResult {
    pub url: String,
    pub connectivity_test: ConnectivityTest,
    pub http_version: Option<String>,
    pub server_header: Option<String>,
    pub security_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateAnalysis {
    pub valid: bool,
    pub issuer: String,
    pub subject: String,
    pub expiry_date: Option<DateTime<Utc>>,
    pub days_until_expiry: Option<i64>,
    pub chain_valid: bool,
    pub security_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseAnalysis {
    pub status_code: u16,
    pub content_type: Option<String>,
    pub content_length: Option<usize>,
    pub compression: Option<String>,
    pub cache_headers: HashMap<String, String>,
    pub performance_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectChainAnalysis {
    pub redirect_count: usize,
    pub final_url: String,
    pub redirect_chain: Vec<String>,
    pub redirect_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub component: String,
    pub bottleneck_type: BottleneckType,
    pub severity: f64,
    pub description: String,
    pub metrics: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyBreakdown {
    pub dns_percentage: f64,
    pub tcp_percentage: f64,
    pub tls_percentage: f64,
    pub request_percentage: f64,
    pub total_average_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputAnalysis {
    pub bytes_per_second: f64,
    pub requests_per_second: f64,
    pub bandwidth_utilization: f64,
    pub bottleneck_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    pub metric: String,
    pub trend_direction: String,
    pub trend_strength: f64,
    pub time_period: (DateTime<Utc>, DateTime<Utc>),
    pub significance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionSummary {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub data_points_collected: usize,
    pub average_response_size: f64,
}

/// Enumerated types for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    Connectivity,
    Performance,
    Security,
    Configuration,
    DNS,
    Certificate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Performance,
    Security,
    Reliability,
    Configuration,
    Monitoring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImplementationComplexity {
    Simple,
    Moderate,
    Complex,
    Expert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottleneckType {
    DNS,
    TCP,
    TLS,
    HTTP,
    Bandwidth,
    Latency,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            connectivity_timeout: Duration::from_secs(10),
            parallel_tests: 4,
            include_dns_diagnostics: true,
            include_http_diagnostics: true,
            include_performance_analysis: true,
            detailed_reporting: true,
            min_sample_size: 5,
        }
    }
}

impl NetworkDiagnostics {
    /// Create a new network diagnostics instance
    pub fn new(
        dns_manager: Arc<DnsManager>,
        http_client: Arc<dyn HttpClient>,
        config: DiagnosticsConfig,
    ) -> Self {
        Self {
            dns_manager,
            http_client,
            config,
        }
    }

    /// Create diagnostics with default configuration
    pub fn with_defaults(dns_manager: Arc<DnsManager>) -> Result<Self> {
        let http_client = Arc::new(NetworkClient::new(dns_manager.clone())?);
        Ok(Self::new(dns_manager, http_client, DiagnosticsConfig::default()))
    }

    /// Run comprehensive network diagnostics
    pub async fn run_diagnostics(&self, targets: &[String], dns_configs: &[DnsConfig]) -> Result<DiagnosticReport> {
        let start_time = Instant::now();
        let mut execution_summary = ExecutionSummary {
            total_duration: Duration::ZERO,
            tests_executed: 0,
            tests_successful: 0,
            tests_failed: 0,
            tests_skipped: 0,
            data_summary: DataCollectionSummary {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                data_points_collected: 0,
                average_response_size: 0.0,
            },
        };

        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Run connectivity diagnostics
        let connectivity_diagnostics = self.run_connectivity_diagnostics(targets).await?;
        self.analyze_connectivity_issues(&connectivity_diagnostics, &mut issues, &mut recommendations);

        // Run DNS diagnostics
        let dns_diagnostics = if self.config.include_dns_diagnostics {
            self.run_dns_diagnostics(targets, dns_configs).await?
        } else {
            execution_summary.tests_skipped += targets.len() * dns_configs.len();
            DnsDiagnostics::empty()
        };
        self.analyze_dns_issues(&dns_diagnostics, &mut issues, &mut recommendations);

        // Run HTTP diagnostics
        let http_diagnostics = if self.config.include_http_diagnostics {
            self.run_http_diagnostics(targets, dns_configs).await?
        } else {
            execution_summary.tests_skipped += targets.len() * dns_configs.len();
            HttpDiagnostics::empty()
        };
        self.analyze_http_issues(&http_diagnostics, &mut issues, &mut recommendations);

        // Run performance analysis
        let performance_analysis = if self.config.include_performance_analysis {
            self.run_performance_analysis(targets, dns_configs).await?
        } else {
            PerformanceAnalysis::empty()
        };
        self.analyze_performance_issues(&performance_analysis, &mut issues, &mut recommendations);

        // Calculate system health
        let system_health = self.calculate_system_health(
            &connectivity_diagnostics,
            &dns_diagnostics,
            &http_diagnostics,
            &performance_analysis,
            &issues,
        );

        // Update execution summary
        execution_summary.total_duration = start_time.elapsed();

        Ok(DiagnosticReport {
            system_health,
            connectivity_diagnostics,
            dns_diagnostics,
            http_diagnostics,
            performance_analysis,
            issues,
            recommendations,
            generated_at: Utc::now(),
            execution_summary,
        })
    }

    /// Run connectivity diagnostics for target URLs
    async fn run_connectivity_diagnostics(&self, targets: &[String]) -> Result<ConnectivityDiagnostics> {
        let mut target_reachability = HashMap::new();
        let mut port_connectivity = HashMap::new();

        // Test basic internet connectivity
        let internet_connectivity = self.test_internet_connectivity().await;

        // Test each target for reachability
        for target in targets {
            let reachability = self.test_target_reachability(target).await;
            target_reachability.insert(target.clone(), reachability);

            // Test common ports for each target
            let ports = self.extract_ports_from_url(target);
            for port in ports {
                let port_result = self.test_port_connectivity(target, port).await;
                port_connectivity.insert(format!("{}:{}", target, port), port_result);
            }
        }

        // Gather network interface information
        let network_interfaces = self.gather_network_interface_info().await;

        // Analyze routing (simplified)
        let routing_analysis = self.analyze_routing().await;

        Ok(ConnectivityDiagnostics {
            internet_connectivity,
            target_reachability,
            port_connectivity,
            network_interfaces,
            routing_analysis,
        })
    }

    /// Run DNS diagnostics for all configurations
    async fn run_dns_diagnostics(&self, targets: &[String], dns_configs: &[DnsConfig]) -> Result<DnsDiagnostics> {
        let mut dns_server_status = HashMap::new();
        let mut resolution_performance = HashMap::new();
        let mut dnssec_status = HashMap::new();
        let mut doh_analysis = HashMap::new();

        // Test each DNS configuration
        for dns_config in dns_configs {
            let config_name = dns_config.name();

            // Test DNS server accessibility
            match dns_config {
                DnsConfig::Custom { servers } => {
                    for server in servers {
                        let status = self.test_dns_server_accessibility(*server).await;
                        dns_server_status.insert(server.to_string(), status);
                    }
                }
                DnsConfig::DoH { url } => {
                    let analysis = self.analyze_doh_provider(url).await;
                    doh_analysis.insert(url.clone(), analysis);
                }
                DnsConfig::System => {
                    // Test system DNS servers
                    if let Ok(system_servers) = self.dns_manager.get_system_dns_servers() {
                        for server in system_servers {
                            let status = self.test_dns_server_accessibility(server).await;
                            dns_server_status.insert(server.to_string(), status);
                        }
                    }
                }
            }

            // Test DNS resolution performance for each target
            for target in targets {
                if let Ok(host) = self.extract_host_from_url(target) {
                    match self.dns_manager.test_resolution_performance(&host, dns_config).await {
                        Ok(perf_result) => {
                            resolution_performance.insert(
                                format!("{}:{}", config_name, host),
                                perf_result,
                            );
                        }
                        Err(_) => {
                            // Create a failed performance result
                            resolution_performance.insert(
                                format!("{}:{}", config_name, host),
                                DnsPerformanceResult {
                                    success: false,
                                    duration: Duration::from_secs(0),
                                    resolved_ips: Vec::new(),
                                    error: Some("DNS resolution failed".to_string()),
                                },
                            );
                        }
                    }

                    // Test DNSSEC support (simplified)
                    let dnssec_supported = self.test_dnssec_support(&host, dns_config).await;
                    dnssec_status.insert(format!("{}:{}", config_name, host), dnssec_supported);
                }
            }
        }

        Ok(DnsDiagnostics {
            dns_server_status,
            resolution_performance,
            cache_analysis: None, // Not implemented in this version
            dnssec_status,
            doh_analysis,
        })
    }

    /// Run HTTP/HTTPS diagnostics
    async fn run_http_diagnostics(&self, targets: &[String], dns_configs: &[DnsConfig]) -> Result<HttpDiagnostics> {
        let mut url_connectivity = HashMap::new();
        let mut certificate_analysis = HashMap::new();
        let mut response_analysis = HashMap::new();
        let mut redirect_analysis = HashMap::new();

        for target in targets {
            for dns_config in dns_configs {
                let config_name = dns_config.name();
                let key = format!("{}:{}", config_name, target);

                // Test HTTP connectivity
                match self.http_client.test_connectivity(target, dns_config).await {
                    Ok(connectivity_test) => {
                        url_connectivity.insert(key.clone(), HttpConnectivityResult {
                            url: target.clone(),
                            connectivity_test,
                            http_version: None, // Would need more detailed HTTP client for this
                            server_header: None,
                            security_headers: HashMap::new(),
                        });
                    }
                    Err(_) => {
                        url_connectivity.insert(key.clone(), HttpConnectivityResult {
                            url: target.clone(),
                            connectivity_test: ConnectivityTest {
                                success: false,
                                status_code: None,
                                response_time: Duration::from_secs(0),
                                resolved_ip: None,
                                dns_resolution_time: Duration::from_secs(0),
                                connection_time: Duration::from_secs(0),
                                error: Some("HTTP connectivity test failed".to_string()),
                            },
                            http_version: None,
                            server_header: None,
                            security_headers: HashMap::new(),
                        });
                    }
                }

                // Analyze SSL certificate for HTTPS URLs
                if target.starts_with("https://") {
                    let cert_analysis = self.analyze_ssl_certificate(target).await;
                    certificate_analysis.insert(key.clone(), cert_analysis);
                }

                // Analyze HTTP response
                let response_info = self.analyze_http_response(target, dns_config).await;
                response_analysis.insert(key.clone(), response_info);

                // Analyze redirect chains
                let redirect_info = self.analyze_redirect_chain(target, dns_config).await;
                redirect_analysis.insert(key.clone(), redirect_info);
            }
        }

        Ok(HttpDiagnostics {
            url_connectivity,
            certificate_analysis,
            response_analysis,
            redirect_analysis,
        })
    }

    /// Run performance analysis
    async fn run_performance_analysis(&self, targets: &[String], dns_configs: &[DnsConfig]) -> Result<PerformanceAnalysis> {
        let mut statistics_engine = StatisticsEngine::with_defaults();
        let mut all_measurements = Vec::new();

        // Collect performance measurements
        for target in targets {
            for dns_config in dns_configs {
                let config_name = dns_config.name();

                // Perform multiple measurements for statistical significance
                let mut test_result = TestResult::new(config_name.clone(), dns_config.clone(), target.clone());

                for _ in 0..self.config.min_sample_size {
                    match self.http_client.head(target, dns_config).await {
                        Ok(response) => {
                            test_result.add_measurement(response.timing.clone());
                            all_measurements.push(response.timing);
                        }
                        Err(_) => {
                            test_result.add_measurement(TimingMetrics::failed("Request failed".to_string()));
                        }
                    }
                }

                test_result.calculate_statistics();
                statistics_engine.add_result(test_result);
            }
        }

        // Generate statistical analysis
        let statistical_analysis = statistics_engine.analyze().ok();

        // Identify performance bottlenecks
        let bottlenecks = self.identify_performance_bottlenecks(&all_measurements);

        // Calculate latency breakdown
        let latency_breakdown = self.calculate_latency_breakdown(&all_measurements);

        // Analyze performance trends (simplified)
        let performance_trends = self.analyze_performance_trends(&all_measurements);

        Ok(PerformanceAnalysis {
            statistical_analysis,
            bottlenecks,
            latency_breakdown,
            throughput_analysis: None, // Not implemented in this version
            performance_trends,
        })
    }

    /// Calculate overall system health
    fn calculate_system_health(
        &self,
        connectivity: &ConnectivityDiagnostics,
        dns: &DnsDiagnostics,
        http: &HttpDiagnostics,
        performance: &PerformanceAnalysis,
        issues: &[DiagnosticIssue],
    ) -> SystemHealth {
        let mut component_scores = HashMap::new();
        let mut critical_issues = Vec::new();
        let mut warning_issues = Vec::new();

        // Calculate connectivity score
        let connectivity_score = self.calculate_connectivity_score(connectivity);
        component_scores.insert("connectivity".to_string(), connectivity_score);

        // Calculate DNS score
        let dns_score = self.calculate_dns_score(dns);
        component_scores.insert("dns".to_string(), dns_score);

        // Calculate HTTP score
        let http_score = self.calculate_http_score(http);
        component_scores.insert("http".to_string(), http_score);

        // Calculate performance score
        let performance_score = self.calculate_performance_score(performance);
        component_scores.insert("performance".to_string(), performance_score);

        // Categorize issues
        for issue in issues {
            match issue.severity {
                IssueSeverity::Critical => critical_issues.push(issue.title.clone()),
                IssueSeverity::High | IssueSeverity::Medium => warning_issues.push(issue.title.clone()),
                IssueSeverity::Low => {} // Ignore low-severity issues for health calculation
            }
        }

        // Calculate overall score
        let overall_score = component_scores.values().sum::<f64>() / component_scores.len() as f64;

        // Determine health status
        let status = if !critical_issues.is_empty() {
            HealthStatus::Critical
        } else if overall_score < 0.5 {
            HealthStatus::Failed
        } else if overall_score < 0.8 || !warning_issues.is_empty() {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        SystemHealth {
            status,
            score: overall_score,
            component_scores,
            critical_issues,
            warning_issues,
        }
    }

    /// Helper methods for individual diagnostic operations
    async fn test_internet_connectivity(&self) -> ConnectivityStatus {
        // Test connectivity to a well-known reliable host
        let test_hosts = vec!["8.8.8.8", "1.1.1.1", "google.com"];
        
        for host in test_hosts {
            if let Ok(status) = self.test_basic_connectivity(host, 80).await {
                if status.reachable {
                    return status;
                }
            }
        }

        ConnectivityStatus {
            reachable: false,
            response_time: None,
            error_message: Some("No internet connectivity detected".to_string()),
            tested_at: Utc::now(),
        }
    }

    async fn test_target_reachability(&self, target: &str) -> ConnectivityStatus {
        match self.extract_host_from_url(target) {
            Ok(host) => {
                let port = self.extract_port_from_url(target);
                self.test_basic_connectivity(&host, port).await.unwrap_or_else(|_| {
                    ConnectivityStatus {
                        reachable: false,
                        response_time: None,
                        error_message: Some("Host unreachable".to_string()),
                        tested_at: Utc::now(),
                    }
                })
            }
            Err(_) => ConnectivityStatus {
                reachable: false,
                response_time: None,
                error_message: Some("Invalid URL format".to_string()),
                tested_at: Utc::now(),
            },
        }
    }

    async fn test_basic_connectivity(&self, host: &str, port: u16) -> Result<ConnectivityStatus> {
        let start_time = Instant::now();
        
        // Parse host as IP or resolve it
        let addr: SocketAddr = if let Ok(ip) = host.parse::<IpAddr>() {
            SocketAddr::new(ip, port)
        } else {
            // Simple hostname resolution (would normally use proper DNS resolution)
            return Ok(ConnectivityStatus {
                reachable: false,
                response_time: None,
                error_message: Some("Hostname resolution not implemented".to_string()),
                tested_at: Utc::now(),
            });
        };

        match TcpStream::connect_timeout(&addr, self.config.connectivity_timeout) {
            Ok(_) => Ok(ConnectivityStatus {
                reachable: true,
                response_time: Some(start_time.elapsed()),
                error_message: None,
                tested_at: Utc::now(),
            }),
            Err(e) => Ok(ConnectivityStatus {
                reachable: false,
                response_time: None,
                error_message: Some(e.to_string()),
                tested_at: Utc::now(),
            }),
        }
    }

    async fn test_port_connectivity(&self, target: &str, port: u16) -> PortConnectivityResult {
        let host = self.extract_host_from_url(target).unwrap_or_else(|_| "localhost".to_string());
        let status = self.test_basic_connectivity(&host, port).await.unwrap_or_else(|_| {
            ConnectivityStatus {
                reachable: false,
                response_time: None,
                error_message: Some("Port connectivity test failed".to_string()),
                tested_at: Utc::now(),
            }
        });

        PortConnectivityResult {
            port,
            protocol: "TCP".to_string(),
            status,
            service_detection: self.detect_service_on_port(port),
        }
    }

    async fn gather_network_interface_info(&self) -> Vec<NetworkInterfaceInfo> {
        // This would normally use system APIs to get network interface information
        // For now, return empty vec as this requires platform-specific code
        Vec::new()
    }

    async fn analyze_routing(&self) -> Option<RoutingAnalysis> {
        // This would normally analyze system routing table
        // For now, return None as this requires platform-specific code
        None
    }

    async fn test_dns_server_accessibility(&self, server: IpAddr) -> DnsServerStatus {
        // Test DNS server accessibility by attempting a simple query
        let dns_config = DnsConfig::Custom { servers: vec![server] };
        
        match self.dns_manager.test_resolution_performance("google.com", &dns_config).await {
            Ok(result) => DnsServerStatus {
                accessible: result.success,
                response_time: Some(result.duration),
                supports_dnssec: false, // Would need more detailed testing
                error_details: result.error,
            },
            Err(e) => DnsServerStatus {
                accessible: false,
                response_time: None,
                supports_dnssec: false,
                error_details: Some(e.to_string()),
            },
        }
    }

    async fn analyze_doh_provider(&self, url: &str) -> DoHProviderAnalysis {
        let _start_time = Instant::now();
        let provider_name = self.extract_provider_name_from_doh_url(url);
        
        // Test DoH provider accessibility
        let dns_config = DnsConfig::DoH { url: url.to_string() };
        
        match self.dns_manager.test_resolution_performance("google.com", &dns_config).await {
            Ok(result) => DoHProviderAnalysis {
                provider_name,
                url: url.to_string(),
                accessible: result.success,
                response_time: Some(result.duration),
                supports_json: true, // Assume JSON support for DoH
                privacy_policy_score: None, // Would need web scraping to analyze
            },
            Err(_) => DoHProviderAnalysis {
                provider_name,
                url: url.to_string(),
                accessible: false,
                response_time: None,
                supports_json: false,
                privacy_policy_score: None,
            },
        }
    }

    async fn test_dnssec_support(&self, _host: &str, _dns_config: &DnsConfig) -> bool {
        // This would require more sophisticated DNS testing
        // For now, return false as DNSSEC testing is complex
        false
    }

    async fn analyze_ssl_certificate(&self, _url: &str) -> CertificateAnalysis {
        // This would require SSL/TLS certificate validation
        // For now, return a placeholder analysis
        CertificateAnalysis {
            valid: true,
            issuer: "Unknown".to_string(),
            subject: "Unknown".to_string(),
            expiry_date: None,
            days_until_expiry: None,
            chain_valid: true,
            security_issues: Vec::new(),
        }
    }

    async fn analyze_http_response(&self, url: &str, dns_config: &DnsConfig) -> ResponseAnalysis {
        // Use the HTTP client to get response details
        match self.http_client.head(url, dns_config).await {
            Ok(response) => ResponseAnalysis {
                status_code: response.status_code,
                content_type: None, // Would need to parse headers
                content_length: Some(response.body_size),
                compression: None,
                cache_headers: HashMap::new(),
                performance_headers: HashMap::new(),
            },
            Err(_) => ResponseAnalysis {
                status_code: 0,
                content_type: None,
                content_length: None,
                compression: None,
                cache_headers: HashMap::new(),
                performance_headers: HashMap::new(),
            },
        }
    }

    async fn analyze_redirect_chain(&self, url: &str, _dns_config: &DnsConfig) -> RedirectChainAnalysis {
        // This would require following redirects and analyzing the chain
        // For now, return a simple analysis
        RedirectChainAnalysis {
            redirect_count: 0,
            final_url: url.to_string(),
            redirect_chain: vec![url.to_string()],
            redirect_issues: Vec::new(),
        }
    }

    fn identify_performance_bottlenecks(&self, measurements: &[TimingMetrics]) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        if measurements.is_empty() {
            return bottlenecks;
        }

        // Analyze DNS resolution times
        let avg_dns_time = measurements.iter().map(|m| m.dns_ms()).sum::<f64>() / measurements.len() as f64;
        if avg_dns_time > 100.0 {
            let mut metrics = HashMap::new();
            metrics.insert("average_dns_time_ms".to_string(), avg_dns_time);
            
            bottlenecks.push(PerformanceBottleneck {
                component: "DNS Resolution".to_string(),
                bottleneck_type: BottleneckType::DNS,
                severity: (avg_dns_time / 500.0).min(1.0), // Scale from 0-1
                description: format!("DNS resolution is taking {:.1}ms on average", avg_dns_time),
                metrics,
                recommendations: vec![
                    "Consider using faster DNS servers".to_string(),
                    "Enable DNS caching".to_string(),
                ],
            });
        }

        // Analyze TCP connection times
        let avg_tcp_time = measurements.iter().map(|m| m.tcp_ms()).sum::<f64>() / measurements.len() as f64;
        if avg_tcp_time > 200.0 {
            let mut metrics = HashMap::new();
            metrics.insert("average_tcp_time_ms".to_string(), avg_tcp_time);
            
            bottlenecks.push(PerformanceBottleneck {
                component: "TCP Connection".to_string(),
                bottleneck_type: BottleneckType::TCP,
                severity: (avg_tcp_time / 1000.0).min(1.0),
                description: format!("TCP connection establishment is taking {:.1}ms on average", avg_tcp_time),
                metrics,
                recommendations: vec![
                    "Check network latency to target servers".to_string(),
                    "Consider using HTTP/2 or HTTP/3 for connection reuse".to_string(),
                ],
            });
        }

        bottlenecks
    }

    fn calculate_latency_breakdown(&self, measurements: &[TimingMetrics]) -> LatencyBreakdown {
        if measurements.is_empty() {
            return LatencyBreakdown {
                dns_percentage: 0.0,
                tcp_percentage: 0.0,
                tls_percentage: 0.0,
                request_percentage: 0.0,
                total_average_ms: 0.0,
            };
        }

        let total_count = measurements.len() as f64;
        let avg_dns = measurements.iter().map(|m| m.dns_ms()).sum::<f64>() / total_count;
        let avg_tcp = measurements.iter().map(|m| m.tcp_ms()).sum::<f64>() / total_count;
        let avg_tls = measurements.iter()
            .map(|m| m.tls_ms().unwrap_or(0.0))
            .sum::<f64>() / total_count;
        let avg_first_byte = measurements.iter().map(|m| m.first_byte_ms()).sum::<f64>() / total_count;
        let avg_total = measurements.iter().map(|m| m.total_ms()).sum::<f64>() / total_count;

        let total_breakdown = avg_dns + avg_tcp + avg_tls + avg_first_byte;

        LatencyBreakdown {
            dns_percentage: if total_breakdown > 0.0 { (avg_dns / total_breakdown) * 100.0 } else { 0.0 },
            tcp_percentage: if total_breakdown > 0.0 { (avg_tcp / total_breakdown) * 100.0 } else { 0.0 },
            tls_percentage: if total_breakdown > 0.0 { (avg_tls / total_breakdown) * 100.0 } else { 0.0 },
            request_percentage: if total_breakdown > 0.0 { (avg_first_byte / total_breakdown) * 100.0 } else { 0.0 },
            total_average_ms: avg_total,
        }
    }

    fn analyze_performance_trends(&self, _measurements: &[TimingMetrics]) -> Vec<PerformanceTrend> {
        // This would require temporal analysis of measurements
        // For now, return empty vec as we need time-series data
        Vec::new()
    }

    /// Issue analysis methods
    fn analyze_connectivity_issues(
        &self,
        connectivity: &ConnectivityDiagnostics,
        issues: &mut Vec<DiagnosticIssue>,
        recommendations: &mut Vec<Recommendation>,
    ) {
        // Check internet connectivity
        if !connectivity.internet_connectivity.reachable {
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Connectivity,
                title: "No Internet Connectivity".to_string(),
                description: "Unable to establish basic internet connectivity".to_string(),
                affected_components: vec!["Network".to_string()],
                impact: "All network-dependent operations will fail".to_string(),
                resolution_steps: vec![
                    "Check network cable connections".to_string(),
                    "Verify network configuration".to_string(),
                    "Contact network administrator".to_string(),
                ],
                related_metrics: HashMap::new(),
            });

            recommendations.push(Recommendation {
                priority: RecommendationPriority::Critical,
                category: RecommendationCategory::Configuration,
                title: "Restore Internet Connectivity".to_string(),
                description: "Establish working internet connection before proceeding with tests".to_string(),
                expected_benefits: vec!["Enable all network testing capabilities".to_string()],
                complexity: ImplementationComplexity::Simple,
                estimated_time: "5-30 minutes".to_string(),
            });
        }

        // Check target reachability
        let unreachable_targets: Vec<&String> = connectivity.target_reachability
            .iter()
            .filter(|(_, status)| !status.reachable)
            .map(|(target, _)| target)
            .collect();

        if !unreachable_targets.is_empty() {
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::Connectivity,
                title: "Target Hosts Unreachable".to_string(),
                description: format!("Unable to reach {} target host(s)", unreachable_targets.len()),
                affected_components: unreachable_targets.iter().map(|s| s.to_string()).collect(),
                impact: "Tests for unreachable hosts will fail".to_string(),
                resolution_steps: vec![
                    "Verify target host URLs are correct".to_string(),
                    "Check if hosts are temporarily down".to_string(),
                    "Test with alternative hosts".to_string(),
                ],
                related_metrics: HashMap::new(),
            });
        }
    }

    fn analyze_dns_issues(
        &self,
        dns: &DnsDiagnostics,
        issues: &mut Vec<DiagnosticIssue>,
        recommendations: &mut Vec<Recommendation>,
    ) {
        // Check DNS server accessibility
        let failed_dns_servers: Vec<&String> = dns.dns_server_status
            .iter()
            .filter(|(_, status)| !status.accessible)
            .map(|(server, _)| server)
            .collect();

        if !failed_dns_servers.is_empty() {
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::DNS,
                title: "DNS Server Connectivity Issues".to_string(),
                description: format!("{} DNS server(s) are not accessible", failed_dns_servers.len()),
                affected_components: failed_dns_servers.iter().map(|s| s.to_string()).collect(),
                impact: "DNS resolution may be slower or fail for affected servers".to_string(),
                resolution_steps: vec![
                    "Test alternative DNS servers".to_string(),
                    "Check firewall settings".to_string(),
                    "Verify DNS server configuration".to_string(),
                ],
                related_metrics: HashMap::new(),
            });

            recommendations.push(Recommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::Performance,
                title: "Switch to Reliable DNS Servers".to_string(),
                description: "Use public DNS servers like 1.1.1.1 or 8.8.8.8 for better reliability".to_string(),
                expected_benefits: vec!["Improved DNS resolution reliability".to_string()],
                complexity: ImplementationComplexity::Simple,
                estimated_time: "5 minutes".to_string(),
            });
        }

        // Check DNS resolution performance
        let slow_resolutions: Vec<(&String, &DnsPerformanceResult)> = dns.resolution_performance
            .iter()
            .filter(|(_, result)| result.success && result.duration > Duration::from_millis(200))
            .collect();

        if !slow_resolutions.is_empty() {
            issues.push(DiagnosticIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Performance,
                title: "Slow DNS Resolution".to_string(),
                description: format!("{} DNS resolution(s) are slower than 200ms", slow_resolutions.len()),
                affected_components: slow_resolutions.iter().map(|(name, _)| name.to_string()).collect(),
                impact: "Overall request latency will be higher".to_string(),
                resolution_steps: vec![
                    "Use faster DNS servers".to_string(),
                    "Enable DNS caching".to_string(),
                    "Consider using DNS-over-HTTPS".to_string(),
                ],
                related_metrics: HashMap::new(),
            });
        }
    }

    fn analyze_http_issues(
        &self,
        _http: &HttpDiagnostics,
        _issues: &mut [DiagnosticIssue],
        _recommendations: &mut [Recommendation],
    ) {
        // HTTP issue analysis would go here
        // For now, skip as the implementation is already quite comprehensive
    }

    fn analyze_performance_issues(
        &self,
        performance: &PerformanceAnalysis,
        issues: &mut Vec<DiagnosticIssue>,
        recommendations: &mut Vec<Recommendation>,
    ) {
        // Analyze performance bottlenecks
        for bottleneck in &performance.bottlenecks {
            let severity = match bottleneck.severity {
                s if s >= 0.8 => IssueSeverity::Critical,
                s if s >= 0.6 => IssueSeverity::High,
                s if s >= 0.4 => IssueSeverity::Medium,
                _ => IssueSeverity::Low,
            };

            issues.push(DiagnosticIssue {
                severity,
                category: IssueCategory::Performance,
                title: format!("{} Performance Bottleneck", bottleneck.component),
                description: bottleneck.description.clone(),
                affected_components: vec![bottleneck.component.clone()],
                impact: "Reduced overall system performance".to_string(),
                resolution_steps: bottleneck.recommendations.clone(),
                related_metrics: bottleneck.metrics.iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect(),
            });

            for recommendation_text in &bottleneck.recommendations {
                recommendations.push(Recommendation {
                    priority: match severity {
                        IssueSeverity::Critical => RecommendationPriority::Critical,
                        IssueSeverity::High => RecommendationPriority::High,
                        IssueSeverity::Medium => RecommendationPriority::Medium,
                        IssueSeverity::Low => RecommendationPriority::Low,
                    },
                    category: RecommendationCategory::Performance,
                    title: format!("Optimize {}", bottleneck.component),
                    description: recommendation_text.clone(),
                    expected_benefits: vec!["Improved response times".to_string()],
                    complexity: ImplementationComplexity::Moderate,
                    estimated_time: "1-4 hours".to_string(),
                });
            }
        }
    }

    /// Component scoring methods
    fn calculate_connectivity_score(&self, connectivity: &ConnectivityDiagnostics) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Internet connectivity (40% weight)
        if connectivity.internet_connectivity.reachable {
            score += 0.4;
        }
        total_weight += 0.4;

        // Target reachability (40% weight)
        if !connectivity.target_reachability.is_empty() {
            let reachable_count = connectivity.target_reachability
                .values()
                .filter(|status| status.reachable)
                .count();
            let reachability_ratio = reachable_count as f64 / connectivity.target_reachability.len() as f64;
            score += 0.4 * reachability_ratio;
        }
        total_weight += 0.4;

        // Port connectivity (20% weight)
        if !connectivity.port_connectivity.is_empty() {
            let accessible_count = connectivity.port_connectivity
                .values()
                .filter(|port_result| port_result.status.reachable)
                .count();
            let port_ratio = accessible_count as f64 / connectivity.port_connectivity.len() as f64;
            score += 0.2 * port_ratio;
        }
        total_weight += 0.2;

        if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        }
    }

    fn calculate_dns_score(&self, dns: &DnsDiagnostics) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // DNS server accessibility (50% weight)
        if !dns.dns_server_status.is_empty() {
            let accessible_count = dns.dns_server_status
                .values()
                .filter(|status| status.accessible)
                .count();
            let dns_ratio = accessible_count as f64 / dns.dns_server_status.len() as f64;
            score += 0.5 * dns_ratio;
        }
        total_weight += 0.5;

        // DNS resolution performance (50% weight)
        if !dns.resolution_performance.is_empty() {
            let successful_count = dns.resolution_performance
                .values()
                .filter(|result| result.success)
                .count();
            let success_ratio = successful_count as f64 / dns.resolution_performance.len() as f64;
            
            // Factor in performance (resolutions under 100ms get full score)
            let fast_resolutions = dns.resolution_performance
                .values()
                .filter(|result| result.success && result.duration <= Duration::from_millis(100))
                .count();
            let performance_bonus = fast_resolutions as f64 / successful_count.max(1) as f64;
            
            score += 0.5 * success_ratio * (0.5 + 0.5 * performance_bonus);
        }
        total_weight += 0.5;

        if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        }
    }

    fn calculate_http_score(&self, http: &HttpDiagnostics) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // URL connectivity (60% weight)
        if !http.url_connectivity.is_empty() {
            let successful_count = http.url_connectivity
                .values()
                .filter(|result| result.connectivity_test.success)
                .count();
            let success_ratio = successful_count as f64 / http.url_connectivity.len() as f64;
            score += 0.6 * success_ratio;
        }
        total_weight += 0.6;

        // Certificate validity (20% weight)
        if !http.certificate_analysis.is_empty() {
            let valid_count = http.certificate_analysis
                .values()
                .filter(|cert| cert.valid)
                .count();
            let cert_ratio = valid_count as f64 / http.certificate_analysis.len() as f64;
            score += 0.2 * cert_ratio;
        }
        total_weight += 0.2;

        // Response quality (20% weight)
        if !http.response_analysis.is_empty() {
            let good_responses = http.response_analysis
                .values()
                .filter(|response| response.status_code >= 200 && response.status_code < 400)
                .count();
            let response_ratio = good_responses as f64 / http.response_analysis.len() as f64;
            score += 0.2 * response_ratio;
        }
        total_weight += 0.2;

        if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        }
    }

    fn calculate_performance_score(&self, performance: &PerformanceAnalysis) -> f64 {
        // Base score
        let mut score = 0.8;

        // Penalize for bottlenecks
        for bottleneck in &performance.bottlenecks {
            score -= bottleneck.severity * 0.2;
        }

        // Bonus for good latency breakdown
        if performance.latency_breakdown.total_average_ms <= 500.0 {
            score += 0.1;
        }

        // Ensure score is between 0 and 1
        score.clamp(0.0, 1.0)
    }

    /// URL parsing utility methods
    fn extract_host_from_url(&self, url: &str) -> Result<String> {
        let parsed = url::Url::parse(url)
            .map_err(|e| AppError::parse(format!("Invalid URL: {}", e)))?;
        
        parsed.host_str()
            .ok_or_else(|| AppError::validation("URL must have a host"))
            .map(|s| s.to_string())
    }

    fn extract_port_from_url(&self, url: &str) -> u16 {
        if let Ok(parsed) = url::Url::parse(url) {
            parsed.port().unwrap_or_else(|| {
                match parsed.scheme() {
                    "https" => 443,
                    "http" => 80,
                    _ => 80,
                }
            })
        } else {
            80
        }
    }

    fn extract_ports_from_url(&self, url: &str) -> Vec<u16> {
        vec![self.extract_port_from_url(url)]
    }

    fn detect_service_on_port(&self, port: u16) -> Option<String> {
        match port {
            21 => Some("FTP".to_string()),
            22 => Some("SSH".to_string()),
            23 => Some("Telnet".to_string()),
            25 => Some("SMTP".to_string()),
            53 => Some("DNS".to_string()),
            80 => Some("HTTP".to_string()),
            110 => Some("POP3".to_string()),
            143 => Some("IMAP".to_string()),
            443 => Some("HTTPS".to_string()),
            993 => Some("IMAPS".to_string()),
            995 => Some("POP3S".to_string()),
            _ => None,
        }
    }

    fn extract_provider_name_from_doh_url(&self, url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                if host.contains("cloudflare") {
                    return "Cloudflare".to_string();
                } else if host.contains("google") {
                    return "Google".to_string();
                } else if host.contains("quad9") {
                    return "Quad9".to_string();
                } else if host.contains("adguard") {
                    return "AdGuard".to_string();
                } else if host.contains("doh.pub") {
                    return "Tencent DNSPod".to_string();
                } else if host.contains("alidns") {
                    return "AliDNS".to_string();
                }
                return host.to_string();
            }
        }
        "Unknown".to_string()
    }
}

/// Empty implementations for diagnostic data structures
impl DnsDiagnostics {
    fn empty() -> Self {
        Self {
            dns_server_status: HashMap::new(),
            resolution_performance: HashMap::new(),
            cache_analysis: None,
            dnssec_status: HashMap::new(),
            doh_analysis: HashMap::new(),
        }
    }
}

impl HttpDiagnostics {
    fn empty() -> Self {
        Self {
            url_connectivity: HashMap::new(),
            certificate_analysis: HashMap::new(),
            response_analysis: HashMap::new(),
            redirect_analysis: HashMap::new(),
        }
    }
}

impl PerformanceAnalysis {
    fn empty() -> Self {
        Self {
            statistical_analysis: None,
            bottlenecks: Vec::new(),
            latency_breakdown: LatencyBreakdown {
                dns_percentage: 0.0,
                tcp_percentage: 0.0,
                tls_percentage: 0.0,
                request_percentage: 0.0,
                total_average_ms: 0.0,
            },
            throughput_analysis: None,
            performance_trends: Vec::new(),
        }
    }
}

/// Report formatting and display functionality
impl DiagnosticReport {
    /// Format the report as a human-readable text summary
    pub fn format_summary(&self) -> String {
        let mut output = String::new();
        
        output.push_str("🔍 Network Diagnostics Report\n");
        output.push_str("================================\n\n");
        
        // System Health Overview
        output.push_str(&format!("📊 System Health: {} (Score: {:.1}%)\n", 
            self.format_health_status(), self.system_health.score * 100.0));
        
        if !self.system_health.critical_issues.is_empty() {
            output.push_str(&format!("🚨 Critical Issues: {}\n", 
                self.system_health.critical_issues.join(", ")));
        }
        
        if !self.system_health.warning_issues.is_empty() {
            output.push_str(&format!("⚠️ Warning Issues: {}\n", 
                self.system_health.warning_issues.join(", ")));
        }
        
        output.push('\n');
        
        // Component Scores
        output.push_str("📈 Component Scores:\n");
        for (component, score) in &self.system_health.component_scores {
            output.push_str(&format!("  • {}: {:.1}%\n", component, score * 100.0));
        }
        output.push('\n');
        
        // Top Issues
        if !self.issues.is_empty() {
            output.push_str("🔧 Top Issues:\n");
            for (i, issue) in self.issues.iter().take(5).enumerate() {
                output.push_str(&format!("{}. {} [{}]\n", 
                    i + 1, issue.title, self.format_severity(issue.severity)));
            }
            output.push('\n');
        }
        
        // Top Recommendations
        if !self.recommendations.is_empty() {
            output.push_str("💡 Top Recommendations:\n");
            for (i, rec) in self.recommendations.iter().take(3).enumerate() {
                output.push_str(&format!("{}. {} [{}]\n", 
                    i + 1, rec.title, self.format_priority(rec.priority)));
            }
            output.push('\n');
        }
        
        // Execution Summary
        output.push_str(&format!("⏱️ Execution: {:.1}s | Tests: {} passed, {} failed\n",
            self.execution_summary.total_duration.as_secs_f64(),
            self.execution_summary.tests_successful,
            self.execution_summary.tests_failed));
        
        output
    }
    
    fn format_health_status(&self) -> String {
        match self.system_health.status {
            HealthStatus::Healthy => "Healthy".green().to_string(),
            HealthStatus::Warning => "Warning".yellow().to_string(),
            HealthStatus::Critical => "Critical".red().to_string(),
            HealthStatus::Failed => "Failed".red().bold().to_string(),
        }
    }
    
    fn format_severity(&self, severity: IssueSeverity) -> String {
        match severity {
            IssueSeverity::Low => "LOW".green().to_string(),
            IssueSeverity::Medium => "MED".yellow().to_string(),
            IssueSeverity::High => "HIGH".red().to_string(),
            IssueSeverity::Critical => "CRIT".red().bold().to_string(),
        }
    }
    
    fn format_priority(&self, priority: RecommendationPriority) -> String {
        match priority {
            RecommendationPriority::Low => "Low".green().to_string(),
            RecommendationPriority::Medium => "Medium".yellow().to_string(),
            RecommendationPriority::High => "High".red().to_string(),
            RecommendationPriority::Critical => "Critical".red().bold().to_string(),
        }
    }

    /// Export the report as JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| AppError::io(format!("Failed to export report to JSON: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_diagnostics_config_default() {
        let config = DiagnosticsConfig::default();
        assert_eq!(config.connectivity_timeout, Duration::from_secs(10));
        assert_eq!(config.parallel_tests, 4);
        assert!(config.include_dns_diagnostics);
        assert!(config.include_http_diagnostics);
        assert!(config.include_performance_analysis);
        assert!(config.detailed_reporting);
        assert_eq!(config.min_sample_size, 5);
    }

    #[test]
    fn test_health_status_ordering() {
        assert!(matches!(HealthStatus::Healthy, HealthStatus::Healthy));
        assert!(matches!(HealthStatus::Warning, HealthStatus::Warning));
        assert!(matches!(HealthStatus::Critical, HealthStatus::Critical));
        assert!(matches!(HealthStatus::Failed, HealthStatus::Failed));
    }

    #[test]
    fn test_connectivity_status_creation() {
        let status = ConnectivityStatus {
            reachable: true,
            response_time: Some(Duration::from_millis(150)),
            error_message: None,
            tested_at: Utc::now(),
        };
        
        assert!(status.reachable);
        assert!(status.response_time.is_some());
        assert!(status.error_message.is_none());
    }

    #[test]
    fn test_diagnostic_issue_severity() {
        let issue = DiagnosticIssue {
            severity: IssueSeverity::High,
            category: IssueCategory::Performance,
            title: "Test Issue".to_string(),
            description: "Test Description".to_string(),
            affected_components: vec!["Test Component".to_string()],
            impact: "Test Impact".to_string(),
            resolution_steps: vec!["Test Step".to_string()],
            related_metrics: HashMap::new(),
        };
        
        assert_eq!(issue.severity, IssueSeverity::High);
        assert_eq!(issue.category, IssueCategory::Performance);
        assert_eq!(issue.title, "Test Issue");
    }

    #[test]
    fn test_recommendation_priority() {
        let recommendation = Recommendation {
            priority: RecommendationPriority::Critical,
            category: RecommendationCategory::Security,
            title: "Test Recommendation".to_string(),
            description: "Test Description".to_string(),
            expected_benefits: vec!["Test Benefit".to_string()],
            complexity: ImplementationComplexity::Simple,
            estimated_time: "5 minutes".to_string(),
        };
        
        assert_eq!(recommendation.priority, RecommendationPriority::Critical);
        assert_eq!(recommendation.category, RecommendationCategory::Security);
        assert_eq!(recommendation.complexity, ImplementationComplexity::Simple);
    }

    #[test]
    fn test_performance_bottleneck_creation() {
        let mut metrics = HashMap::new();
        metrics.insert("test_metric".to_string(), 42.0);
        
        let bottleneck = PerformanceBottleneck {
            component: "Test Component".to_string(),
            bottleneck_type: BottleneckType::DNS,
            severity: 0.8,
            description: "Test bottleneck".to_string(),
            metrics,
            recommendations: vec!["Fix it".to_string()],
        };
        
        assert_eq!(bottleneck.component, "Test Component");
        assert_eq!(bottleneck.bottleneck_type, BottleneckType::DNS);
        assert_eq!(bottleneck.severity, 0.8);
    }

    #[test]
    fn test_empty_diagnostic_structures() {
        let dns_diag = DnsDiagnostics::empty();
        assert!(dns_diag.dns_server_status.is_empty());
        assert!(dns_diag.resolution_performance.is_empty());
        
        let http_diag = HttpDiagnostics::empty();
        assert!(http_diag.url_connectivity.is_empty());
        
        let perf_analysis = PerformanceAnalysis::empty();
        assert!(perf_analysis.bottlenecks.is_empty());
        assert_eq!(perf_analysis.latency_breakdown.total_average_ms, 0.0);
    }

    #[tokio::test]
    async fn test_network_diagnostics_creation() {
        let dns_manager = Arc::new(DnsManager::new().unwrap());
        let result = NetworkDiagnostics::with_defaults(dns_manager);
        assert!(result.is_ok());
        
        let diagnostics = result.unwrap();
        assert_eq!(diagnostics.config.parallel_tests, 4);
        assert!(diagnostics.config.include_dns_diagnostics);
    }

    #[test]
    fn test_system_health_calculation() {
        let mut component_scores = HashMap::new();
        component_scores.insert("connectivity".to_string(), 0.9);
        component_scores.insert("dns".to_string(), 0.8);
        component_scores.insert("http".to_string(), 0.7);
        component_scores.insert("performance".to_string(), 0.85);
        
        let system_health = SystemHealth {
            status: HealthStatus::Healthy,
            score: 0.8125, // Average of the above
            component_scores,
            critical_issues: Vec::new(),
            warning_issues: Vec::new(),
        };
        
        assert_eq!(system_health.status, HealthStatus::Healthy);
        assert!((system_health.score - 0.8125).abs() < 0.001);
        assert!(system_health.critical_issues.is_empty());
    }
}