//! Configuration management module

pub mod parser;
pub mod validation;
pub mod env;

// Re-export main functionality
pub use parser::{ConfigParser, load_config, display_config_summary};
pub use validation::{ConfigValidator, validate_config};
pub use env::EnvManager;

// Re-export from models for convenience
pub use crate::models::Config;

// Additional comprehensive tests in separate module
#[cfg(test)]
mod comprehensive_tests;