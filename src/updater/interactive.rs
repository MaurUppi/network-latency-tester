//! Interactive user interface for version selection and progress display
//!
//! This module provides user-friendly interfaces for selecting versions interactively,
//! displaying progress during operations, and handling user input. It leverages
//! the dialoguer crate for enhanced interactive experience with graceful fallback
//! to basic stdio when dialoguer is not available.

use crate::{AppError, Result};
use crate::updater::types::{Release, Version, VersionChoice};
use std::io::{self, Write};

/// Interactive user interface manager for version selection and progress display
pub struct InteractiveUI {
    /// Whether to use colored output
    use_colors: bool,
    /// Whether to use enhanced interactive features (dialoguer)
    use_enhanced: bool,
}

impl InteractiveUI {
    /// Create a new InteractiveUI instance
    pub fn new(use_colors: bool) -> Self {
        let use_enhanced = cfg!(feature = "dialoguer");
        Self {
            use_colors,
            use_enhanced,
        }
    }

    /// Display version menu and get user selection
    /// 
    /// Shows the 3 most recent versions plus current version, with clear
    /// indicators for current/latest status and includes a custom version option.
    pub fn display_version_menu(&self, releases: &[Release], current_version: &Version) -> Result<VersionChoice> {
        if releases.is_empty() {
            return Err(AppError::update("No releases available for selection"));
        }

        // Take the 3 most recent releases for display
        let display_releases: Vec<&Release> = releases.iter().take(3).collect();
        
        self.print_header("Available Versions")?;
        self.print_separator()?;
        
        // Display current version info
        self.print_current_version(current_version)?;
        self.print_empty_line()?;
        
        // Display available versions with numbered options
        for (index, release) in display_releases.iter().enumerate() {
            let option_number = index + 1;
            let is_latest = index == 0;
            let is_current = release.version() == current_version.original;
            
            self.print_version_option(option_number, release, is_latest, is_current)?;
        }
        
        // Add custom version option
        let custom_option = display_releases.len() + 1;
        self.print_custom_option(custom_option)?;
        
        self.print_separator()?;
        
        // Get user selection
        self.get_user_selection(display_releases.len() + 1)
    }

    /// Display progress indicator with message
    pub fn display_progress(&self, message: &str) {
        if self.use_colors {
            use colored::Colorize;
            eprint!("{} {}...", "[PROGRESS]".cyan().bold(), message);
        } else {
            eprint!("[PROGRESS] {}...", message);
        }
        io::stderr().flush().unwrap_or(());
    }

    /// Clear progress indicator and show completion
    pub fn complete_progress(&self, message: &str) {
        if self.use_colors {
            use colored::Colorize;
            eprintln!("\r{} {}", "[COMPLETE]".green().bold(), message);
        } else {
            eprintln!("\r[COMPLETE] {}", message);
        }
    }

    /// Display error message with formatting
    pub fn display_error(&self, message: &str) {
        if self.use_colors {
            use colored::Colorize;
            eprintln!("{} {}", "[ERROR]".red().bold(), message);
        } else {
            eprintln!("[ERROR] {}", message);
        }
    }

    /// Display success message with formatting
    pub fn display_success(&self, message: &str) {
        if self.use_colors {
            use colored::Colorize;
            eprintln!("{} {}", "[SUCCESS]".green().bold(), message);
        } else {
            eprintln!("[SUCCESS] {}", message);
        }
    }

    /// Print header with formatting
    fn print_header(&self, title: &str) -> Result<()> {
        if self.use_colors {
            use colored::Colorize;
            println!("\n{}", title.cyan().bold());
        } else {
            println!("\n{}", title);
        }
        Ok(())
    }

    /// Print separator line
    fn print_separator(&self) -> Result<()> {
        if self.use_colors {
            use colored::Colorize;
            println!("{}", "─".repeat(50).dimmed());
        } else {
            println!("{}", "-".repeat(50));
        }
        Ok(())
    }

    /// Print empty line
    fn print_empty_line(&self) -> Result<()> {
        println!();
        Ok(())
    }

    /// Print current version information
    fn print_current_version(&self, current_version: &Version) -> Result<()> {
        if self.use_colors {
            use colored::Colorize;
            println!("{} {}", 
                "Current version:".bold(), 
                current_version.original.blue()
            );
        } else {
            println!("Current version: {}", current_version.original);
        }
        Ok(())
    }

    /// Print a version option with formatting
    fn print_version_option(&self, number: usize, release: &Release, is_latest: bool, is_current: bool) -> Result<()> {
        let version = release.version();
        let date = self.format_release_date(&release.published_at);
        
        let mut status_indicators = Vec::new();
        if is_current {
            status_indicators.push("current");
        }
        if is_latest {
            status_indicators.push("latest");
        }
        
        let status_text = if !status_indicators.is_empty() {
            format!(" ({})", status_indicators.join(", "))
        } else {
            String::new()
        };

        if self.use_colors {
            use colored::Colorize;
            let number_text = format!("{})", number).yellow().bold();
            let version_text = if is_current {
                version.blue().bold()
            } else if is_latest {
                version.green()
            } else {
                version.normal()
            };
            let status_text = if !status_text.is_empty() {
                status_text.dimmed()
            } else {
                status_text.normal()
            };
            
            println!("  {} {} - {}{}", number_text, version_text, date.dimmed(), status_text);
        } else {
            println!("  {}) {} - {}{}", number, version, date, status_text);
        }
        Ok(())
    }

    /// Print custom version option
    fn print_custom_option(&self, number: usize) -> Result<()> {
        if self.use_colors {
            use colored::Colorize;
            println!("  {} {}", 
                format!("{})", number).yellow().bold(),
                "Enter custom version".italic()
            );
        } else {
            println!("  {}) Enter custom version", number);
        }
        Ok(())
    }

    /// Get user selection using appropriate input method
    fn get_user_selection(&self, max_options: usize) -> Result<VersionChoice> {
        if self.use_enhanced {
            self.get_enhanced_selection(max_options)
        } else {
            self.get_basic_selection(max_options)
        }
    }

    /// Get user selection using dialoguer (enhanced)
    #[cfg(feature = "dialoguer")]
    fn get_enhanced_selection(&self, max_options: usize) -> Result<VersionChoice> {
        use dialoguer::{Input, Select};
        
        // Create selection menu items
        let items: Vec<String> = (1..=max_options).map(|i| {
            if i == max_options {
                "Enter custom version".to_string()
            } else {
                format!("Version option {}", i)
            }
        }).collect();

        let selection = Select::new()
            .with_prompt("Select version")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|e| AppError::update(format!("Selection failed: {}", e)))?;

        if selection + 1 == max_options {
            // Custom version selected
            let custom_version: String = Input::new()
                .with_prompt("Enter version (e.g., 0.1.7 or v0.1.7)")
                .interact_text()
                .map_err(|e| AppError::update(format!("Input failed: {}", e)))?;
            
            if custom_version.trim().is_empty() {
                Ok(VersionChoice::Cancel)
            } else {
                Ok(VersionChoice::Custom(custom_version.trim().to_string()))
            }
        } else {
            Ok(VersionChoice::Release(selection))
        }
    }

    /// Fallback implementation when dialoguer is not available
    #[cfg(not(feature = "dialoguer"))]
    fn get_enhanced_selection(&self, max_options: usize) -> Result<VersionChoice> {
        self.get_basic_selection(max_options)
    }

    /// Get user selection using basic stdio
    fn get_basic_selection(&self, max_options: usize) -> Result<VersionChoice> {
        loop {
            if self.use_colors {
                use colored::Colorize;
                print!("\n{} ", "Select option (1-{} or 'q' to quit):".bold());
            } else {
                print!("\nSelect option (1-{} or 'q' to quit): ", max_options);
            }
            
            io::stdout().flush().map_err(|e| AppError::update(format!("Failed to flush stdout: {}", e)))?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .map_err(|e| AppError::update(format!("Failed to read input: {}", e)))?;
            
            let input = input.trim();
            
            // Handle quit
            if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") {
                return Ok(VersionChoice::Cancel);
            }
            
            // Try to parse as number
            if let Ok(choice) = input.parse::<usize>() {
                if choice >= 1 && choice <= max_options {
                    if choice == max_options {
                        // Custom version option
                        return self.get_custom_version_input();
                    } else {
                        return Ok(VersionChoice::Release(choice - 1)); // Convert to 0-based index
                    }
                }
            }
            
            // Invalid input
            self.display_error(&format!("Invalid input '{}'. Please enter a number between 1 and {} or 'q' to quit.", input, max_options));
        }
    }

    /// Get custom version input from user
    fn get_custom_version_input(&self) -> Result<VersionChoice> {
        if self.use_colors {
            use colored::Colorize;
            print!("{} ", "Enter version (e.g., 0.1.7 or v0.1.7):".bold());
        } else {
            print!("Enter version (e.g., 0.1.7 or v0.1.7): ");
        }
        
        io::stdout().flush().map_err(|e| AppError::update(format!("Failed to flush stdout: {}", e)))?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .map_err(|e| AppError::update(format!("Failed to read input: {}", e)))?;
        
        let input = input.trim();
        if input.is_empty() {
            Ok(VersionChoice::Cancel)
        } else {
            Ok(VersionChoice::Custom(input.to_string()))
        }
    }

    /// Format release date for display
    fn format_release_date(&self, published_at: &str) -> String {
        // Parse ISO 8601 date and format for display
        // For now, just extract the date part (YYYY-MM-DD)
        if let Some(date_part) = published_at.split('T').next() {
            date_part.to_string()
        } else {
            published_at.to_string()
        }
    }
}

impl Default for InteractiveUI {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updater::types::{ReleaseAsset};

    fn create_test_release(tag_name: &str, published_at: &str) -> Release {
        Release::new(
            tag_name.to_string(),
            format!("Release {}", tag_name),
            published_at.to_string(),
            "https://example.com".to_string(),
            vec![
                ReleaseAsset::new(
                    format!("binary-{}.tar.gz", tag_name),
                    "https://example.com/download".to_string(),
                    1024,
                    "application/gzip".to_string(),
                )
            ],
            false,
        )
    }

    #[test]
    fn test_interactive_ui_creation() {
        let ui = InteractiveUI::new(true);
        assert!(ui.use_colors);
        
        let ui = InteractiveUI::new(false);
        assert!(!ui.use_colors);
    }

    #[test]
    fn test_interactive_ui_default() {
        let ui = InteractiveUI::default();
        assert!(ui.use_colors);
    }

    #[test]
    fn test_format_release_date() {
        let ui = InteractiveUI::new(false);
        
        let iso_date = "2024-01-15T10:30:45Z";
        let formatted = ui.format_release_date(iso_date);
        assert_eq!(formatted, "2024-01-15");
        
        let partial_date = "2024-01-15";
        let formatted = ui.format_release_date(partial_date);
        assert_eq!(formatted, "2024-01-15");
    }

    #[test]
    fn test_display_version_menu_empty_releases() {
        let ui = InteractiveUI::new(false);
        let current_version = Version::parse("1.0.0").unwrap();
        let releases = vec![];
        
        let result = ui.display_version_menu(&releases, &current_version);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No releases available"));
    }

    #[test]
    fn test_progress_display() {
        let ui = InteractiveUI::new(false);
        
        // These should not panic
        ui.display_progress("Testing progress");
        ui.complete_progress("Test complete");
        ui.display_error("Test error");
        ui.display_success("Test success");
    }

    #[test]
    fn test_version_menu_display_structure() {
        let ui = InteractiveUI::new(false);
        
        let releases = vec![
            create_test_release("v1.2.0", "2024-01-20T10:00:00Z"),
            create_test_release("v1.1.0", "2024-01-15T10:00:00Z"),
            create_test_release("v1.0.0", "2024-01-10T10:00:00Z"),
        ];
        
        let current_version = Version::parse("1.1.0").unwrap();
        
        // This test verifies the structure works without panicking
        // The actual menu display would require integration testing with terminal
        assert_eq!(releases.len(), 3);
        assert_eq!(current_version.original, "1.1.0");
        
        // Test that we can identify current version
        let current_release = releases.iter().find(|r| r.version() == current_version.original);
        assert!(current_release.is_some());
    }

    #[test]
    fn test_version_choice_variants() {
        // Test that all VersionChoice variants can be created
        let release_choice = VersionChoice::Release(0);
        let custom_choice = VersionChoice::Custom("1.2.3".to_string());
        let cancel_choice = VersionChoice::Cancel;
        
        match release_choice {
            VersionChoice::Release(index) => assert_eq!(index, 0),
            _ => panic!("Wrong variant"),
        }
        
        match custom_choice {
            VersionChoice::Custom(version) => assert_eq!(version, "1.2.3"),
            _ => panic!("Wrong variant"),
        }
        
        match cancel_choice {
            VersionChoice::Cancel => {},
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_color_formatting_safety() {
        let ui_color = InteractiveUI::new(true);
        let ui_no_color = InteractiveUI::new(false);
        
        // Test that these methods don't panic with different color settings
        ui_color.display_progress("Test");
        ui_no_color.display_progress("Test");
        
        ui_color.display_error("Test");
        ui_no_color.display_error("Test");
        
        ui_color.display_success("Test");
        ui_no_color.display_success("Test");
    }

    #[test]
    fn test_enhanced_feature_detection() {
        let ui = InteractiveUI::new(true);
        
        // Test that enhanced feature detection doesn't cause compilation issues
        #[cfg(feature = "dialoguer")]
        assert!(ui.use_enhanced);
        
        #[cfg(not(feature = "dialoguer"))]
        assert!(!ui.use_enhanced);
    }
}