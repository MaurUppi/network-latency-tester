//! Version comparison and validation logic
//!
//! This module provides comprehensive version management functionality using semantic versioning
//! rules from the semver crate. It handles version parsing, comparison, upgrade/downgrade 
//! validation, and string normalization.

use crate::{AppError, Result};
use crate::updater::types::{Version, VersionRelation};
use semver::Version as SemVer;
use std::str::FromStr;

/// Version manager for semantic version operations and upgrade/downgrade logic
pub struct VersionManager {
    /// Whether to allow pre-release versions in comparisons
    allow_prerelease: bool,
    /// Whether verbose logging is enabled
    verbose: bool,
}

impl VersionManager {
    /// Create a new VersionManager with default settings
    pub fn new() -> Self {
        Self {
            allow_prerelease: false,
            verbose: false,
        }
    }

    /// Create a new VersionManager with configuration
    pub fn with_config(allow_prerelease: bool, verbose: bool) -> Self {
        Self {
            allow_prerelease,
            verbose,
        }
    }

    /// Parse a version string into a Version struct with semver validation
    pub fn parse_version(&self, version_str: &str) -> Result<Version> {
        if self.verbose {
            eprintln!("[VERSION] Parsing version string: '{}'", version_str);
        }

        // Normalize the version string by stripping 'v' prefix
        let normalized = self.normalize_version_string(version_str);
        
        // Validate using semver first for strict semantic version rules
        let semver = SemVer::from_str(&normalized)
            .map_err(|e| AppError::version(format!("Invalid semantic version '{}': {}", version_str, e)))?;

        // Check pre-release policy
        if !self.allow_prerelease && !semver.pre.is_empty() {
            return Err(AppError::version(format!(
                "Pre-release versions not allowed: '{}'. Use --allow-prerelease to enable.",
                version_str
            )));
        }

        // Convert to our Version struct
        let pre_release = if semver.pre.is_empty() {
            None
        } else {
            Some(semver.pre.to_string())
        };

        Ok(Version::new(
            semver.major as u32,
            semver.minor as u32,
            semver.patch as u32,
            pre_release,
            version_str.to_string(),
        ))
    }

    /// Compare two versions and determine their relationship
    pub fn compare_versions(&self, current: &Version, target: &Version) -> Result<VersionRelation> {
        if self.verbose {
            eprintln!("[VERSION] Comparing current '{}' with target '{}'", current.original, target.original);
        }

        // Convert to semver for accurate comparison
        let current_semver = self.to_semver(current)?;
        let target_semver = self.to_semver(target)?;

        let relation = match current_semver.cmp(&target_semver) {
            std::cmp::Ordering::Less => VersionRelation::Upgrade,
            std::cmp::Ordering::Equal => VersionRelation::Same,
            std::cmp::Ordering::Greater => VersionRelation::Downgrade,
        };

        if self.verbose {
            eprintln!("[VERSION] Comparison result: {}", relation);
        }

        Ok(relation)
    }

    /// Check if a downgrade operation is safe and allowed
    pub fn check_downgrade_safety(&self, current: &Version, target: &Version, force: bool) -> Result<()> {
        let relation = self.compare_versions(current, target)?;

        match relation {
            VersionRelation::Upgrade => {
                if self.verbose {
                    eprintln!("[VERSION] Upgrade detected: {} -> {}", current.original, target.original);
                }
                Ok(())
            }
            VersionRelation::Same => {
                if self.verbose {
                    eprintln!("[VERSION] Same version detected: {}", current.original);
                }
                Ok(())
            }
            VersionRelation::Downgrade => {
                if force {
                    if self.verbose {
                        eprintln!("[VERSION] Forced downgrade allowed: {} -> {}", current.original, target.original);
                    }
                    Ok(())
                } else {
                    Err(AppError::version(format!(
                        "Downgrade detected from {} to {}. Use --force to proceed with downgrade.",
                        current.original, target.original
                    )))
                }
            }
        }
    }

    /// Validate that a version string is properly formatted
    pub fn validate_version_string(&self, version_str: &str) -> Result<()> {
        if version_str.is_empty() {
            return Err(AppError::version("Version string cannot be empty"));
        }

        // Attempt to parse to validate format
        self.parse_version(version_str)?;
        Ok(())
    }

    /// Normalize a version string by stripping 'v' prefix and cleaning whitespace
    pub fn normalize_version_string(&self, version_str: &str) -> String {
        version_str
            .trim()
            .strip_prefix('v')
            .unwrap_or(version_str.trim())
            .to_string()
    }

    /// Check if a version is newer than another
    pub fn is_newer(&self, version: &Version, than: &Version) -> Result<bool> {
        let relation = self.compare_versions(than, version)?;
        Ok(matches!(relation, VersionRelation::Upgrade))
    }

    /// Check if a version is older than another
    pub fn is_older(&self, version: &Version, than: &Version) -> Result<bool> {
        let relation = self.compare_versions(than, version)?;
        Ok(matches!(relation, VersionRelation::Downgrade))
    }

    /// Check if two versions are the same
    pub fn is_same(&self, version1: &Version, version2: &Version) -> Result<bool> {
        let relation = self.compare_versions(version1, version2)?;
        Ok(matches!(relation, VersionRelation::Same))
    }

    /// Find the latest version from a list of versions
    pub fn find_latest_version(&self, versions: &[Version]) -> Result<Option<Version>> {
        if versions.is_empty() {
            return Ok(None);
        }

        let mut latest = &versions[0];
        
        for version in versions.iter().skip(1) {
            if self.is_newer(version, latest)? {
                latest = version;
            }
        }

        Ok(Some(latest.clone()))
    }

    /// Filter out pre-release versions if not allowed
    pub fn filter_prerelease_versions(&self, versions: Vec<Version>) -> Vec<Version> {
        if self.allow_prerelease {
            versions
        } else {
            versions.into_iter()
                .filter(|v| !v.is_prerelease())
                .collect()
        }
    }

    /// Check if version satisfies a requirement string (e.g., ">=1.0.0", "^2.1.0")
    pub fn satisfies_requirement(&self, version: &Version, requirement: &str) -> Result<bool> {
        let semver_version = self.to_semver(version)?;
        
        let req = semver::VersionReq::parse(requirement)
            .map_err(|e| AppError::version(format!("Invalid version requirement '{}': {}", requirement, e)))?;

        Ok(req.matches(&semver_version))
    }

    /// Get version increment suggestions (patch, minor, major)
    pub fn get_increment_suggestions(&self, current: &Version) -> Result<Vec<Version>> {
        let current_semver = self.to_semver(current)?;
        
        let mut suggestions = Vec::new();

        // Patch increment
        let patch_semver = SemVer::new(
            current_semver.major,
            current_semver.minor,
            current_semver.patch + 1,
        );
        suggestions.push(self.from_semver(&patch_semver)?);

        // Minor increment
        let minor_semver = SemVer::new(
            current_semver.major,
            current_semver.minor + 1,
            0,
        );
        suggestions.push(self.from_semver(&minor_semver)?);

        // Major increment
        let major_semver = SemVer::new(
            current_semver.major + 1,
            0,
            0,
        );
        suggestions.push(self.from_semver(&major_semver)?);

        Ok(suggestions)
    }

    /// Convert our Version struct to semver::Version for operations
    fn to_semver(&self, version: &Version) -> Result<SemVer> {
        let version_str = match &version.pre_release {
            Some(pre) => format!("{}.{}.{}-{}", version.major, version.minor, version.patch, pre),
            None => format!("{}.{}.{}", version.major, version.minor, version.patch),
        };

        SemVer::from_str(&version_str)
            .map_err(|e| AppError::version(format!("Failed to convert version '{}' to semver: {}", version_str, e)))
    }

    /// Convert semver::Version to our Version struct
    fn from_semver(&self, semver: &SemVer) -> Result<Version> {
        let pre_release = if semver.pre.is_empty() {
            None
        } else {
            Some(semver.pre.to_string())
        };

        let original = match &pre_release {
            Some(pre) => format!("{}.{}.{}-{}", semver.major, semver.minor, semver.patch, pre),
            None => format!("{}.{}.{}", semver.major, semver.minor, semver.patch),
        };

        Ok(Version::new(
            semver.major as u32,
            semver.minor as u32,
            semver.patch as u32,
            pre_release,
            original,
        ))
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Standalone utility functions for version operations
pub mod utils {
    use super::*;

    /// Quick version comparison without creating a VersionManager
    pub fn compare_version_strings(current: &str, target: &str) -> Result<VersionRelation> {
        let manager = VersionManager::new();
        let current_version = manager.parse_version(current)?;
        let target_version = manager.parse_version(target)?;
        manager.compare_versions(&current_version, &target_version)
    }

    /// Quick version validation without creating a VersionManager
    pub fn is_valid_version(version_str: &str) -> bool {
        let manager = VersionManager::new();
        manager.validate_version_string(version_str).is_ok()
    }

    /// Quick version normalization
    pub fn normalize_version(version_str: &str) -> String {
        let manager = VersionManager::new();
        manager.normalize_version_string(version_str)
    }

    /// Check if a version string represents a pre-release
    pub fn is_prerelease_version(version_str: &str) -> Result<bool> {
        let manager = VersionManager::new();
        let version = manager.parse_version(version_str)?;
        Ok(version.is_prerelease())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_manager_creation() {
        let manager = VersionManager::new();
        assert!(!manager.allow_prerelease);
        assert!(!manager.verbose);

        let manager = VersionManager::with_config(true, true);
        assert!(manager.allow_prerelease);
        assert!(manager.verbose);
    }

    #[test]
    fn test_version_parsing() {
        let manager = VersionManager::new();

        // Valid versions
        let version = manager.parse_version("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);

        let version = manager.parse_version("v2.0.0").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.original, "v2.0.0");

        // Invalid versions
        assert!(manager.parse_version("invalid").is_err());
        assert!(manager.parse_version("1.2").is_err());
        assert!(manager.parse_version("").is_err());
    }

    #[test]
    fn test_prerelease_handling() {
        let manager_no_prerelease = VersionManager::new();
        let manager_with_prerelease = VersionManager::with_config(true, false);

        // Pre-release should fail with default manager
        assert!(manager_no_prerelease.parse_version("1.0.0-alpha").is_err());

        // Pre-release should succeed with configured manager
        let version = manager_with_prerelease.parse_version("1.0.0-alpha.1").unwrap();
        assert_eq!(version.pre_release, Some("alpha.1".to_string()));
        assert!(version.is_prerelease());
    }

    #[test]
    fn test_version_comparison() {
        let manager = VersionManager::new();
        
        let v1_0_0 = manager.parse_version("1.0.0").unwrap();
        let v1_1_0 = manager.parse_version("1.1.0").unwrap();
        let v2_0_0 = manager.parse_version("2.0.0").unwrap();

        // Test upgrade detection
        let relation = manager.compare_versions(&v1_0_0, &v1_1_0).unwrap();
        assert_eq!(relation, VersionRelation::Upgrade);

        let relation = manager.compare_versions(&v1_0_0, &v2_0_0).unwrap();
        assert_eq!(relation, VersionRelation::Upgrade);

        // Test same version
        let relation = manager.compare_versions(&v1_0_0, &v1_0_0).unwrap();
        assert_eq!(relation, VersionRelation::Same);

        // Test downgrade detection
        let relation = manager.compare_versions(&v2_0_0, &v1_0_0).unwrap();
        assert_eq!(relation, VersionRelation::Downgrade);
    }

    #[test]
    fn test_downgrade_safety() {
        let manager = VersionManager::new();
        
        let current = manager.parse_version("2.0.0").unwrap();
        let target = manager.parse_version("1.0.0").unwrap();

        // Downgrade without force should fail
        assert!(manager.check_downgrade_safety(&current, &target, false).is_err());

        // Downgrade with force should succeed
        assert!(manager.check_downgrade_safety(&current, &target, true).is_ok());

        // Upgrade should always succeed
        assert!(manager.check_downgrade_safety(&target, &current, false).is_ok());
    }

    #[test]
    fn test_version_string_normalization() {
        let manager = VersionManager::new();

        assert_eq!(manager.normalize_version_string("v1.2.3"), "1.2.3");
        assert_eq!(manager.normalize_version_string("1.2.3"), "1.2.3");
        assert_eq!(manager.normalize_version_string("  v1.2.3  "), "1.2.3");
        assert_eq!(manager.normalize_version_string("  1.2.3  "), "1.2.3");
    }

    #[test]
    fn test_version_validation() {
        let manager = VersionManager::new();

        assert!(manager.validate_version_string("1.2.3").is_ok());
        assert!(manager.validate_version_string("v1.2.3").is_ok());
        assert!(manager.validate_version_string("").is_err());
        assert!(manager.validate_version_string("invalid").is_err());
    }

    #[test]
    fn test_version_utility_methods() {
        let manager = VersionManager::new();
        
        let v1 = manager.parse_version("1.0.0").unwrap();
        let v2 = manager.parse_version("2.0.0").unwrap();

        assert!(manager.is_newer(&v2, &v1).unwrap());
        assert!(!manager.is_newer(&v1, &v2).unwrap());

        assert!(manager.is_older(&v1, &v2).unwrap());
        assert!(!manager.is_older(&v2, &v1).unwrap());

        assert!(manager.is_same(&v1, &v1).unwrap());
        assert!(!manager.is_same(&v1, &v2).unwrap());
    }

    #[test]
    fn test_find_latest_version() {
        let manager = VersionManager::new();
        
        let versions = vec![
            manager.parse_version("1.0.0").unwrap(),
            manager.parse_version("2.0.0").unwrap(),
            manager.parse_version("1.5.0").unwrap(),
        ];

        let latest = manager.find_latest_version(&versions).unwrap().unwrap();
        assert_eq!(latest.original, "2.0.0");

        // Empty list
        assert!(manager.find_latest_version(&[]).unwrap().is_none());
    }

    #[test]
    fn test_prerelease_filtering() {
        let manager = VersionManager::with_config(true, false);
        
        let versions = vec![
            manager.parse_version("1.0.0").unwrap(),
            manager.parse_version("2.0.0-alpha").unwrap(),
            manager.parse_version("1.5.0").unwrap(),
        ];

        let filtered = manager.filter_prerelease_versions(versions.clone());
        assert_eq!(filtered.len(), 3); // All included when allowed

        let manager_no_prerelease = VersionManager::new();
        let filtered = manager_no_prerelease.filter_prerelease_versions(versions);
        assert_eq!(filtered.len(), 2); // Pre-release filtered out
        assert!(!filtered.iter().any(|v| v.is_prerelease()));
    }

    #[test]
    fn test_version_requirements() {
        let manager = VersionManager::new();
        let version = manager.parse_version("1.2.3").unwrap();

        assert!(manager.satisfies_requirement(&version, ">=1.0.0").unwrap());
        assert!(manager.satisfies_requirement(&version, "^1.2.0").unwrap());
        assert!(manager.satisfies_requirement(&version, "~1.2.0").unwrap());
        assert!(!manager.satisfies_requirement(&version, ">=2.0.0").unwrap());

        // Invalid requirement
        assert!(manager.satisfies_requirement(&version, "invalid").is_err());
    }

    #[test]
    fn test_increment_suggestions() {
        let manager = VersionManager::new();
        let current = manager.parse_version("1.2.3").unwrap();
        
        let suggestions = manager.get_increment_suggestions(&current).unwrap();
        assert_eq!(suggestions.len(), 3);
        
        // Patch increment: 1.2.4
        assert_eq!(suggestions[0].major, 1);
        assert_eq!(suggestions[0].minor, 2);
        assert_eq!(suggestions[0].patch, 4);
        
        // Minor increment: 1.3.0
        assert_eq!(suggestions[1].major, 1);
        assert_eq!(suggestions[1].minor, 3);
        assert_eq!(suggestions[1].patch, 0);
        
        // Major increment: 2.0.0
        assert_eq!(suggestions[2].major, 2);
        assert_eq!(suggestions[2].minor, 0);
        assert_eq!(suggestions[2].patch, 0);
    }

    #[test]
    fn test_utility_functions() {
        // Test standalone utility functions
        let relation = utils::compare_version_strings("1.0.0", "2.0.0").unwrap();
        assert_eq!(relation, VersionRelation::Upgrade);

        assert!(utils::is_valid_version("1.2.3"));
        assert!(!utils::is_valid_version("invalid"));

        assert_eq!(utils::normalize_version("v1.2.3"), "1.2.3");

        assert!(!utils::is_prerelease_version("1.2.3").unwrap());
        assert!(utils::is_prerelease_version("1.2.3-alpha").is_err()); // Default manager doesn't allow prerelease
    }

    #[test]
    fn test_edge_cases() {
        let manager = VersionManager::with_config(true, false);

        // Test zero versions
        let zero_version = manager.parse_version("0.0.0").unwrap();
        assert_eq!(zero_version.major, 0);

        // Test large version numbers
        let large_version = manager.parse_version("999.999.999").unwrap();
        assert_eq!(large_version.major, 999);

        // Test complex pre-release identifiers
        let complex_pre = manager.parse_version("1.0.0-alpha.beta.1").unwrap();
        assert_eq!(complex_pre.pre_release, Some("alpha.beta.1".to_string()));

        // Test version with build metadata (semver allows but strips it)
        let build_version = manager.parse_version("1.0.0+build.123").unwrap();
        assert_eq!(build_version.major, 1);
        assert_eq!(build_version.minor, 0);
        assert_eq!(build_version.patch, 0);
    }

    #[test]
    fn test_error_messages() {
        let manager = VersionManager::new();

        // Test descriptive error messages for empty string
        let err = manager.parse_version("").unwrap_err();
        // The semver library gives a specific error for empty strings, so check for "Invalid semantic version"
        assert!(err.to_string().contains("Invalid semantic version"));

        let err = manager.parse_version("1.0.0-alpha").unwrap_err();
        assert!(err.to_string().contains("Pre-release versions not allowed"));

        let current = manager.parse_version("2.0.0").unwrap();
        let target = manager.parse_version("1.0.0").unwrap();
        let err = manager.check_downgrade_safety(&current, &target, false).unwrap_err();
        assert!(err.to_string().contains("Downgrade detected"));
        assert!(err.to_string().contains("Use --force"));
    }
}