//! Update feature module for version management
//!
//! This module provides comprehensive version management functionality including
//! checking for updates, upgrading/downgrading versions, and geographic-aware
//! download acceleration.
//!
//! ## Architecture
//!
//! The updater module follows a layered architecture:
//! - **UpdateCoordinator**: Main orchestration component managing the update workflow
//! - **Data Sources**: Multiple data sources (GitHub Atom feeds, REST API, cache)
//! - **Version Management**: Version comparison, validation, and upgrade/downgrade logic
//! - **Geographic Detection**: Location-aware download acceleration
//! - **Interactive UI**: User interaction for version selection and progress display
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::updater::{UpdateCoordinator, UpdateArgs};
//!
//! let args = UpdateArgs::new(true, Some("1.2.3".to_string()), false);
//! let coordinator = UpdateCoordinator::new();
//! coordinator.execute_update_flow(&args).await?;
//! ```

use crate::{AppError, Result};
use std::time::{Duration, Instant};

// Core types module
pub mod types;

// Sub-modules for specific functionality (to be implemented in subsequent tasks)
pub mod version;     // Task 6: Version comparison and validation logic  
pub mod cache;       // Task 7: Cache management system
pub mod feeds;       // Task 8: GitHub Atom feeds client
// pub mod github;      // Task 9: GitHub REST API client
// pub mod data;        // Task 10: Data source management layer
// pub mod geo;         // Task 11: Geographic detection
// pub mod interactive; // Task 12: Interactive user interface

// Re-export commonly used types for convenience
pub use types::{
    Release, ReleaseAsset, Version, UpdateArgs, GeographicRegion,
    VersionRelation, VersionChoice,
};
pub use version::VersionManager;
pub use cache::{CacheManager, CacheStats};
pub use feeds::{FeedsClient, FeedStats};

/// Update operation results
#[derive(Debug, Clone)]
pub enum UpdateResult {
    /// Update check completed successfully
    UpdateAvailable {
        current: Version,
        latest: Release,
        download_url: String,
    },
    /// Already up to date
    AlreadyUpToDate {
        current: Version,
    },
    /// Downgrade operation
    DowngradeAvailable {
        current: Version,
        target: Release,
        download_url: String,
    },
    /// Interactive mode - user needs to select version
    InteractiveSelection {
        current: Version,
        available_releases: Vec<Release>,
    },
}

/// Update operation modes
#[derive(Debug, Clone)]
pub enum UpdateMode {
    /// Check for latest version and upgrade
    CheckLatest,
    /// Target specific version
    TargetVersion(String),
    /// Interactive version selection
    Interactive,
    /// Force version change (including downgrades)
    ForceVersion(String),
}

/// Main coordinator for update operations
pub struct UpdateCoordinator {
    /// Start time for operation tracking
    start_time: Instant,
    /// Whether to use colored output
    use_colors: bool,
    /// Verbose output mode
    verbose: bool,
    /// Version manager for semantic version operations
    version_manager: version::VersionManager,
}

impl UpdateCoordinator {
    /// Create a new UpdateCoordinator
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            use_colors: true,
            verbose: false,
            version_manager: version::VersionManager::new(),
        }
    }

    /// Create a new UpdateCoordinator with configuration
    pub fn with_config(use_colors: bool, verbose: bool) -> Self {
        Self {
            start_time: Instant::now(),
            use_colors,
            verbose,
            version_manager: version::VersionManager::with_config(false, verbose),
        }
    }

    /// Main entry point for update operations
    pub async fn execute_update_flow(&self, args: &UpdateArgs) -> Result<UpdateResult> {
        if self.verbose {
            self.log_info("Starting update operation...");
        }

        // Validate update arguments
        args.validate().map_err(AppError::validation)?;

        // Determine update mode based on arguments
        let mode = self.determine_update_mode(args);

        if self.verbose {
            self.log_info(&format!("Update mode: {:?}", mode));
        }

        // Execute the appropriate update flow
        match mode {
            UpdateMode::CheckLatest => self.check_latest_version().await,
            UpdateMode::TargetVersion(version) => self.target_specific_version(&version).await,
            UpdateMode::Interactive => self.interactive_version_selection().await,
            UpdateMode::ForceVersion(version) => self.force_version_change(&version).await,
        }
    }

    /// Check for updates against CLI arguments
    pub async fn check_for_updates(&self, args: &UpdateArgs) -> Result<UpdateResult> {
        if self.verbose {
            self.log_info("Checking for updates...");
        }

        self.execute_update_flow(args).await
    }

    /// Determine update mode from arguments
    fn determine_update_mode(&self, args: &UpdateArgs) -> UpdateMode {
        match (args.target_version.as_ref(), args.force_downgrade, args.interactive) {
            (Some(version), true, _) => UpdateMode::ForceVersion(version.clone()),
            (Some(version), false, _) => UpdateMode::TargetVersion(version.clone()),
            (None, _, true) => UpdateMode::Interactive,
            (None, _, false) => UpdateMode::CheckLatest,
        }
    }

    /// Check for latest version (placeholder implementation)
    async fn check_latest_version(&self) -> Result<UpdateResult> {
        if self.verbose {
            self.log_info("Checking for latest version...");
        }

        // TODO: Implement in subsequent tasks
        // This is a placeholder that will be replaced when data source modules are implemented
        let current_version = self.get_current_version()?;
        
        // For now, return already up to date
        Ok(UpdateResult::AlreadyUpToDate {
            current: current_version,
        })
    }

    /// Target specific version with enhanced validation
    async fn target_specific_version(&self, version: &str) -> Result<UpdateResult> {
        if self.verbose {
            self.log_info(&format!("Targeting specific version: {}", version));
        }

        // Parse and validate target version using VersionManager
        let target_version = self.version_manager.parse_version(version)?;

        let current_version = self.get_current_version()?;
        
        // Check version relationship and downgrade safety
        self.version_manager.check_downgrade_safety(&current_version, &target_version, false)?;

        let relation = self.version_manager.compare_versions(&current_version, &target_version)?;
        
        match relation {
            types::VersionRelation::Same => {
                if self.verbose {
                    self.log_info("Target version is the same as current version");
                }
                Ok(UpdateResult::AlreadyUpToDate {
                    current: current_version,
                })
            }
            types::VersionRelation::Upgrade => {
                if self.verbose {
                    self.log_info(&format!("Upgrade available: {} -> {}", current_version.original, target_version.original));
                }
                // TODO: In subsequent tasks, this will fetch the actual release data
                Ok(UpdateResult::AlreadyUpToDate { current: current_version })
            }
            types::VersionRelation::Downgrade => {
                // This should not happen since check_downgrade_safety would catch it
                Err(AppError::version(format!(
                    "Downgrade detected from {} to {}. Use --force to proceed.",
                    current_version.original, target_version.original
                )))
            }
        }
    }

    /// Interactive version selection (placeholder implementation)
    async fn interactive_version_selection(&self) -> Result<UpdateResult> {
        if self.verbose {
            self.log_info("Starting interactive version selection...");
        }

        // TODO: Implement in subsequent tasks
        // This will involve data retrieval, UI display, and user input handling
        let current_version = self.get_current_version()?;
        
        // For now, return already up to date
        Ok(UpdateResult::AlreadyUpToDate {
            current: current_version,
        })
    }

    /// Force version change with downgrade protection bypass
    async fn force_version_change(&self, version: &str) -> Result<UpdateResult> {
        if self.verbose {
            self.log_info(&format!("Forcing version change to: {}", version));
        }

        // Parse and validate target version using VersionManager
        let target_version = self.version_manager.parse_version(version)?;

        let current_version = self.get_current_version()?;
        
        // Check version relationship and allow downgrade with force flag
        self.version_manager.check_downgrade_safety(&current_version, &target_version, true)?;

        let relation = self.version_manager.compare_versions(&current_version, &target_version)?;
        
        match relation {
            types::VersionRelation::Same => {
                if self.verbose {
                    self.log_info("Target version is the same as current version");
                }
                Ok(UpdateResult::AlreadyUpToDate {
                    current: current_version,
                })
            }
            types::VersionRelation::Upgrade => {
                if self.verbose {
                    self.log_info(&format!("Forced upgrade: {} -> {}", current_version.original, target_version.original));
                }
                // TODO: In subsequent tasks, this will fetch the actual release data
                Ok(UpdateResult::AlreadyUpToDate { current: current_version })
            }
            types::VersionRelation::Downgrade => {
                if self.verbose {
                    self.log_warning(&format!("Forced downgrade: {} -> {}", current_version.original, target_version.original));
                    self.log_warning("WARNING: Downgrades may introduce security vulnerabilities or remove features");
                }
                // TODO: In subsequent tasks, this will fetch the actual release data
                Ok(UpdateResult::AlreadyUpToDate { current: current_version })
            }
        }
    }

    /// Get current application version using VersionManager
    fn get_current_version(&self) -> Result<Version> {
        // Get version from the crate environment variable
        let current_version_str = crate::VERSION;
        self.version_manager.parse_version(current_version_str)
            .map_err(|e| AppError::version(format!("Failed to parse current version '{}': {}", current_version_str, e)))
    }

    /// Get operation duration
    pub fn get_duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Log info message with optional color
    fn log_info(&self, message: &str) {
        if self.use_colors {
            use colored::Colorize;
            eprintln!("{} {}", "[UPDATE]".blue().bold(), message);
        } else {
            eprintln!("[UPDATE] {}", message);
        }
    }

    /// Log success message with optional color
    fn log_success(&self, message: &str) {
        if self.use_colors {
            use colored::Colorize;
            eprintln!("{} {}", "[SUCCESS]".green().bold(), message);
        } else {
            eprintln!("[SUCCESS] {}", message);
        }
    }

    /// Log warning message with optional color
    fn log_warning(&self, message: &str) {
        if self.use_colors {
            use colored::Colorize;
            eprintln!("{} {}", "[WARNING]".yellow().bold(), message);
        } else {
            eprintln!("[WARNING] {}", message);
        }
    }

    /// Log error message with optional color
    fn log_error(&self, message: &str) {
        if self.use_colors {
            use colored::Colorize;
            eprintln!("{} {}", "[ERROR]".red().bold(), message);
        } else {
            eprintln!("[ERROR] {}", message);
        }
    }
}

impl Default for UpdateCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for UpdateCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateCoordinator")
            .field("elapsed", &self.start_time.elapsed())
            .field("use_colors", &self.use_colors)
            .field("verbose", &self.verbose)
            .field("version_manager_config", &format!("VersionManager(verbose: {})", self.verbose))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_coordinator_creation() {
        let coordinator = UpdateCoordinator::new();
        assert!(coordinator.use_colors);
        assert!(!coordinator.verbose);
        assert!(coordinator.get_duration().as_millis() < 100); // Should be very recent
    }

    #[test]
    fn test_update_coordinator_with_config() {
        let coordinator = UpdateCoordinator::with_config(false, true);
        assert!(!coordinator.use_colors);
        assert!(coordinator.verbose);
    }

    #[test]
    fn test_update_coordinator_default() {
        let coordinator = UpdateCoordinator::default();
        assert!(coordinator.use_colors);
        assert!(!coordinator.verbose);
    }

    #[test]
    fn test_determine_update_mode() {
        let coordinator = UpdateCoordinator::new();

        // Force version mode
        let args = UpdateArgs::new(true, Some("1.2.3".to_string()), true);
        let mode = coordinator.determine_update_mode(&args);
        assert!(matches!(mode, UpdateMode::ForceVersion(_)));

        // Target version mode
        let args = UpdateArgs::new(true, Some("1.2.3".to_string()), false);
        let mode = coordinator.determine_update_mode(&args);
        assert!(matches!(mode, UpdateMode::TargetVersion(_)));

        // Interactive mode (interactive=true)
        let interactive_args = UpdateArgs { 
            update: true, 
            target_version: None, 
            force_downgrade: false, 
            interactive: true 
        };
        let mode = coordinator.determine_update_mode(&interactive_args);
        assert!(matches!(mode, UpdateMode::Interactive));

        // Check latest mode (interactive=false, no target version)
        let check_latest_args = UpdateArgs { 
            update: true, 
            target_version: None, 
            force_downgrade: false, 
            interactive: false 
        };
        let mode = coordinator.determine_update_mode(&check_latest_args);
        assert!(matches!(mode, UpdateMode::CheckLatest));
    }

    #[test]
    fn test_get_current_version() {
        let coordinator = UpdateCoordinator::new();
        let version = coordinator.get_current_version();
        assert!(version.is_ok());
        
        let version = version.unwrap();
        assert_eq!(version.original, crate::VERSION);
    }

    #[tokio::test]
    async fn test_execute_update_flow_validation() {
        let coordinator = UpdateCoordinator::new();
        
        // Test invalid arguments (version without update)
        let invalid_args = UpdateArgs::new(false, Some("1.2.3".to_string()), false);
        let result = coordinator.execute_update_flow(&invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_for_updates_basic() {
        let coordinator = UpdateCoordinator::new();
        let args = UpdateArgs::new(true, None, false);
        
        let result = coordinator.check_for_updates(&args).await;
        assert!(result.is_ok());
        
        // Should return AlreadyUpToDate for placeholder implementation
        match result.unwrap() {
            UpdateResult::AlreadyUpToDate { .. } => {}, // Expected
            _ => panic!("Expected AlreadyUpToDate result"),
        }
    }

    #[tokio::test]
    async fn test_target_specific_version() {
        let coordinator = UpdateCoordinator::new();
        
        // Test valid version format
        let result = coordinator.target_specific_version("1.2.3").await;
        assert!(result.is_ok());
        
        // Test invalid version format
        let result = coordinator.target_specific_version("invalid").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_force_version_change() {
        let coordinator = UpdateCoordinator::new();
        
        // Test valid version format
        let result = coordinator.force_version_change("0.1.0").await;
        assert!(result.is_ok());
        
        // Test invalid version format
        let result = coordinator.force_version_change("not-a-version").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_update_result_variants() {
        let current_version = Version::parse("1.0.0").unwrap();
        
        // Test UpdateAvailable
        let release = Release::new(
            "v1.1.0".to_string(),
            "Version 1.1.0".to_string(),
            "2024-01-01T00:00:00Z".to_string(),
            "https://example.com".to_string(),
            vec![],
            false,
        );
        
        let result = UpdateResult::UpdateAvailable {
            current: current_version.clone(),
            latest: release.clone(),
            download_url: "https://download.com".to_string(),
        };
        
        match result {
            UpdateResult::UpdateAvailable { current, latest, download_url } => {
                assert_eq!(current.major, 1);
                assert_eq!(latest.tag_name, "v1.1.0");
                assert_eq!(download_url, "https://download.com");
            }
            _ => panic!("Expected UpdateAvailable"),
        }
        
        // Test AlreadyUpToDate
        let result = UpdateResult::AlreadyUpToDate {
            current: current_version.clone(),
        };
        
        match result {
            UpdateResult::AlreadyUpToDate { current } => {
                assert_eq!(current.major, 1);
            }
            _ => panic!("Expected AlreadyUpToDate"),
        }
    }

    #[test]
    fn test_update_mode_debug() {
        let mode = UpdateMode::CheckLatest;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("CheckLatest"));
        
        let mode = UpdateMode::TargetVersion("1.2.3".to_string());
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("TargetVersion"));
        assert!(debug_str.contains("1.2.3"));
    }

    #[test]
    fn test_coordinator_debug() {
        let coordinator = UpdateCoordinator::with_config(false, true);
        let debug_str = format!("{:?}", coordinator);
        assert!(debug_str.contains("UpdateCoordinator"));
        assert!(debug_str.contains("use_colors"));
        assert!(debug_str.contains("verbose"));
        assert!(debug_str.contains("elapsed"));
    }
}