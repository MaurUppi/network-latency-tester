//! Core data types for the update feature
//!
//! This module defines all the fundamental data structures used by the updater,
//! including GitHub release information, version handling, and configuration types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a GitHub release with all relevant metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Release {
    /// Release tag name (e.g., "v0.1.10")
    pub tag_name: String,
    /// Human-readable release name
    pub name: String,
    /// ISO 8601 publication date
    pub published_at: String,
    /// GitHub release page URL
    pub html_url: String,
    /// Available binary assets for download
    pub assets: Vec<ReleaseAsset>,
    /// Whether this is a pre-release version
    pub prerelease: bool,
}

impl Release {
    /// Create a new Release instance
    pub fn new(
        tag_name: String,
        name: String,
        published_at: String,
        html_url: String,
        assets: Vec<ReleaseAsset>,
        prerelease: bool,
    ) -> Self {
        Self {
            tag_name,
            name,
            published_at,
            html_url,
            assets,
            prerelease,
        }
    }

    /// Get the version from tag_name, stripping 'v' prefix if present
    pub fn version(&self) -> String {
        self.tag_name.strip_prefix('v').unwrap_or(&self.tag_name).to_string()
    }

    /// Check if this release has assets for download
    pub fn has_assets(&self) -> bool {
        !self.assets.is_empty()
    }
}

/// Represents a downloadable asset from a GitHub release
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReleaseAsset {
    /// Asset filename (e.g., "nlt-v0.1.10-x86_64-apple-darwin")
    pub name: String,
    /// Direct download URL
    pub browser_download_url: String,
    /// File size in bytes
    pub size: u64,
    /// MIME content type
    pub content_type: String,
}

impl ReleaseAsset {
    /// Create a new ReleaseAsset instance
    pub fn new(name: String, browser_download_url: String, size: u64, content_type: String) -> Self {
        Self {
            name,
            browser_download_url,
            size,
            content_type,
        }
    }

    /// Format size in human-readable format
    pub fn formatted_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = self.size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Semantic version representation with comparison capabilities
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Patch version number
    pub patch: u32,
    /// Pre-release identifier (e.g., "alpha", "beta", "rc1")
    pub pre_release: Option<String>,
    /// Original string representation
    pub original: String,
}

impl Version {
    /// Create a new Version instance
    pub fn new(major: u32, minor: u32, patch: u32, pre_release: Option<String>, original: String) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release,
            original,
        }
    }

    /// Parse a version string into a Version struct
    pub fn parse(version_str: &str) -> Result<Self, String> {
        // Strip 'v' prefix if present
        let clean_version = version_str.strip_prefix('v').unwrap_or(version_str);
        
        // Split on '-' to separate version from pre-release
        let parts: Vec<&str> = clean_version.split('-').collect();
        let version_part = parts[0];
        let pre_release = if parts.len() > 1 {
            Some(parts[1..].join("-"))
        } else {
            None
        };

        // Parse major.minor.patch
        let version_numbers: Vec<&str> = version_part.split('.').collect();
        if version_numbers.len() < 3 {
            return Err(format!("Invalid version format: '{}'. Expected format: 'x.y.z' or 'vx.y.z'", version_str));
        }

        let major = version_numbers[0].parse::<u32>()
            .map_err(|_| format!("Invalid major version: '{}'", version_numbers[0]))?;
        let minor = version_numbers[1].parse::<u32>()
            .map_err(|_| format!("Invalid minor version: '{}'", version_numbers[1]))?;
        let patch = version_numbers[2].parse::<u32>()
            .map_err(|_| format!("Invalid patch version: '{}'", version_numbers[2]))?;

        Ok(Version::new(major, minor, patch, pre_release, version_str.to_string()))
    }

    /// Check if this is a pre-release version
    pub fn is_prerelease(&self) -> bool {
        self.pre_release.is_some()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.pre_release {
            Some(pre) => write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre),
            None => write!(f, "{}.{}.{}", self.major, self.minor, self.patch),
        }
    }
}

/// Command-line arguments specific to update functionality
#[derive(Debug, Clone)]
pub struct UpdateArgs {
    /// Whether update mode is activated (--update flag)
    pub update: bool,
    /// Target version to install (--version argument)
    pub target_version: Option<String>,
    /// Force downgrade without confirmation (--force flag)
    pub force_downgrade: bool,
    /// Whether to run in interactive mode (derived from presence of target_version)
    pub interactive: bool,
}

impl UpdateArgs {
    /// Create a new UpdateArgs instance
    pub fn new(update: bool, target_version: Option<String>, force_downgrade: bool) -> Self {
        let interactive = update && target_version.is_none();
        Self {
            update,
            target_version,
            force_downgrade,
            interactive,
        }
    }

    /// Check if update arguments are valid
    pub fn validate(&self) -> Result<(), String> {
        if !self.update && (self.target_version.is_some() || self.force_downgrade) {
            return Err("--version and --force require --update to be specified".to_string());
        }
        Ok(())
    }
}

/// Geographic regions for download acceleration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeographicRegion {
    /// China mainland - requires download acceleration
    ChinaMainland,
    /// Global region - direct GitHub downloads
    Global,
    /// Unknown region - default to global
    Unknown,
}

impl GeographicRegion {
    /// Check if this region requires download acceleration
    pub fn needs_acceleration(&self) -> bool {
        matches!(self, GeographicRegion::ChinaMainland)
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            GeographicRegion::ChinaMainland => "China mainland",
            GeographicRegion::Global => "Global",
            GeographicRegion::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for GeographicRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Relationship between two versions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionRelation {
    /// Target version is newer than current
    Upgrade,
    /// Target version is the same as current
    Same,
    /// Target version is older than current
    Downgrade,
}

impl VersionRelation {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            VersionRelation::Upgrade => "upgrade",
            VersionRelation::Same => "same version",
            VersionRelation::Downgrade => "downgrade",
        }
    }
}

impl fmt::Display for VersionRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Choice made by user during interactive version selection
#[derive(Debug, Clone)]
pub enum VersionChoice {
    /// User selected a specific release by index
    Release(usize),
    /// User chose to enter a custom version string
    Custom(String),
    /// User cancelled the operation
    Cancel,
}

/// Platform information for automatic OS/architecture detection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system (e.g., "macos", "linux", "windows")
    pub os: String,
    /// Architecture (e.g., "x86_64", "aarch64", "arm64")
    pub arch: String,
    /// Target triple (e.g., "x86_64-apple-darwin")
    pub target_triple: String,
}

impl PlatformInfo {
    /// Create PlatformInfo for the current system
    pub fn current() -> Self {
        let os = match std::env::consts::OS {
            "macos" => "macos",
            "linux" => "linux", 
            "windows" => "windows",
            other => other,
        }.to_string();
        
        let arch = std::env::consts::ARCH.to_string();
        
        // Create more specific target triple
        let target_triple = match std::env::consts::OS {
            "macos" => match std::env::consts::ARCH {
                "x86_64" => "x86_64-apple-darwin".to_string(),
                "aarch64" => "aarch64-apple-darwin".to_string(),
                _ => format!("{}-apple-darwin", std::env::consts::ARCH),
            },
            "linux" => match std::env::consts::ARCH {
                "x86_64" => "x86_64-unknown-linux-gnu".to_string(),
                "aarch64" => "aarch64-unknown-linux-gnu".to_string(),
                _ => format!("{}-unknown-linux-gnu", std::env::consts::ARCH),
            },
            "windows" => match std::env::consts::ARCH {
                "x86_64" => "x86_64-pc-windows-msvc".to_string(),
                "aarch64" => "aarch64-pc-windows-msvc".to_string(),
                _ => format!("{}-pc-windows-msvc", std::env::consts::ARCH),
            },
            _ => format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        };
        
        Self {
            os,
            arch,
            target_triple,
        }
    }
    
    /// Check if an asset name matches this platform
    pub fn matches_asset_name(&self, asset_name: &str) -> bool {
        let asset_lower = asset_name.to_lowercase();
        
        // Map current platform to CI archive naming convention
        let expected_patterns = self.get_ci_archive_patterns();
        
        // Check if asset name contains any of the expected patterns
        expected_patterns.iter().any(|pattern| asset_lower.contains(pattern))
    }
    
    /// Get CI archive naming patterns for this platform
    pub fn get_ci_archive_patterns(&self) -> Vec<String> {
        match (self.os.as_str(), self.arch.as_str()) {
            ("windows", "x86_64") => vec!["windows-x64".to_string()],
            ("macos", "x86_64") => vec!["darwin-x64".to_string()], 
            ("macos", "aarch64") => vec!["darwin-arm64".to_string()],
            ("linux", "x86_64") => vec!["linux-x64-ubuntu".to_string(), "linux-x64".to_string()],
            // Fallback patterns for edge cases
            ("windows", arch) => vec![format!("windows-{}", arch)],
            ("macos", arch) => {
                let arch_name = if arch == "aarch64" { "arm64" } else { arch };
                vec![format!("darwin-{}", arch_name)]
            },
            ("linux", arch) => {
                let arch_name = if arch == "aarch64" { "arm64" } else { arch };
                vec![format!("linux-{}", arch_name)]
            },
            _ => vec![format!("{}-{}", self.os, self.arch)],
        }
    }
    
    /// Get preferred file extension for this platform
    pub fn preferred_extension(&self) -> &'static str {
        match self.os.as_str() {
            "windows" => ".zip",
            _ => ".tar.gz",
        }
    }
    
    /// Format platform information for display
    pub fn display_name(&self) -> String {
        format!("{} {}", 
            match self.os.as_str() {
                "macos" => "macOS",
                "linux" => "Linux", 
                "windows" => "Windows",
                other => other,
            },
            match self.arch.as_str() {
                "x86_64" => "x64",
                "aarch64" => "ARM64",
                "arm64" => "ARM64",
                other => other,
            }
        )
    }
}

impl Default for PlatformInfo {
    fn default() -> Self {
        Self::current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = Version::parse("v1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);
        assert_eq!(version.original, "v1.2.3");
    }

    #[test]
    fn test_platform_info_creation() {
        let platform = PlatformInfo::current();
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
        assert!(!platform.target_triple.is_empty());
    }

    #[test]
    fn test_platform_asset_matching() {
        let macos_x64_platform = PlatformInfo {
            os: "macos".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-apple-darwin".to_string(),
        };
        
        // Test actual CI release filename patterns
        assert!(macos_x64_platform.matches_asset_name("network-latency-tester-v0.1.9-darwin-x64.tar.gz"));
        assert!(!macos_x64_platform.matches_asset_name("network-latency-tester-v0.1.9-darwin-arm64.tar.gz"));
        assert!(!macos_x64_platform.matches_asset_name("network-latency-tester-v0.1.9-linux-x64-ubuntu.tar.gz"));
        assert!(!macos_x64_platform.matches_asset_name("network-latency-tester-v0.1.9-windows-x64.zip"));
        
        let macos_arm64_platform = PlatformInfo {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            target_triple: "aarch64-apple-darwin".to_string(),
        };
        
        assert!(macos_arm64_platform.matches_asset_name("network-latency-tester-v0.1.9-darwin-arm64.tar.gz"));
        assert!(!macos_arm64_platform.matches_asset_name("network-latency-tester-v0.1.9-darwin-x64.tar.gz"));
        
        let linux_platform = PlatformInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        };
        
        assert!(linux_platform.matches_asset_name("network-latency-tester-v0.1.9-linux-x64-ubuntu.tar.gz"));
        assert!(!linux_platform.matches_asset_name("network-latency-tester-v0.1.9-windows-x64.zip"));
        
        let windows_platform = PlatformInfo {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-pc-windows-msvc".to_string(),
        };
        
        assert!(windows_platform.matches_asset_name("network-latency-tester-v0.1.9-windows-x64.zip"));
        assert!(!windows_platform.matches_asset_name("network-latency-tester-v0.1.9-darwin-x64.tar.gz"));
    }

    #[test]
    fn test_platform_preferred_extension() {
        let unix_platform = PlatformInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        };
        assert_eq!(unix_platform.preferred_extension(), ".tar.gz");
        
        let windows_platform = PlatformInfo {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-pc-windows-msvc".to_string(),
        };
        assert_eq!(windows_platform.preferred_extension(), ".zip");
    }

    #[test]
    fn test_platform_display_name() {
        let platform = PlatformInfo {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            target_triple: "aarch64-apple-darwin".to_string(),
        };
        assert_eq!(platform.display_name(), "macOS ARM64");
        
        let platform = PlatformInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        };
        assert_eq!(platform.display_name(), "Linux x64");
    }

    #[test]
    fn test_platform_arm64_matching() {
        let platform = PlatformInfo {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            target_triple: "aarch64-apple-darwin".to_string(),
        };
        
        // Should match CI naming pattern for ARM64 
        assert!(platform.matches_asset_name("network-latency-tester-v0.1.9-darwin-arm64.tar.gz"));
        assert!(!platform.matches_asset_name("network-latency-tester-v0.1.9-darwin-x64.tar.gz"));
    }
    
    #[test]
    fn test_ci_archive_patterns() {
        let macos_x64 = PlatformInfo {
            os: "macos".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-apple-darwin".to_string(),
        };
        assert_eq!(macos_x64.get_ci_archive_patterns(), vec!["darwin-x64"]);
        
        let macos_arm64 = PlatformInfo {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            target_triple: "aarch64-apple-darwin".to_string(),
        };
        assert_eq!(macos_arm64.get_ci_archive_patterns(), vec!["darwin-arm64"]);
        
        let linux_x64 = PlatformInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        };
        assert_eq!(linux_x64.get_ci_archive_patterns(), vec!["linux-x64-ubuntu", "linux-x64"]);
        
        let windows_x64 = PlatformInfo {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-pc-windows-msvc".to_string(),
        };
        assert_eq!(windows_x64.get_ci_archive_patterns(), vec!["windows-x64"]);
    }

    #[test]
    fn test_version_parsing_no_prefix() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_parsing_prerelease() {
        let version = Version::parse("v1.2.3-beta.1").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, Some("beta.1".to_string()));
        assert!(version.is_prerelease());
    }

    #[test]
    fn test_version_parsing_invalid() {
        assert!(Version::parse("invalid").is_err());
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("1.2.x").is_err());
    }

    #[test]
    fn test_release_version_extraction() {
        let release = Release::new(
            "v1.2.3".to_string(),
            "Release 1.2.3".to_string(),
            "2024-01-01T00:00:00Z".to_string(),
            "https://example.com".to_string(),
            vec![],
            false,
        );
        assert_eq!(release.version(), "1.2.3");
    }

    #[test]
    fn test_asset_size_formatting() {
        let asset = ReleaseAsset::new(
            "test".to_string(),
            "https://example.com".to_string(),
            1536, // 1.5 KB
            "application/octet-stream".to_string(),
        );
        assert_eq!(asset.formatted_size(), "1.5 KB");
    }

    #[test]
    fn test_update_args_validation() {
        let args = UpdateArgs::new(false, Some("1.2.3".to_string()), false);
        assert!(args.validate().is_err());

        let args = UpdateArgs::new(true, Some("1.2.3".to_string()), false);
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_geographic_region_acceleration() {
        assert!(GeographicRegion::ChinaMainland.needs_acceleration());
        assert!(!GeographicRegion::Global.needs_acceleration());
        assert!(!GeographicRegion::Unknown.needs_acceleration());
    }
}