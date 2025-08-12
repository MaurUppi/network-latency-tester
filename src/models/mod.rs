//! Data models and structures for the network latency tester

pub mod config;
pub mod metrics;

// Re-export main model types
pub use config::Config;
pub use metrics::{TimingMetrics, TestResult, Statistics};