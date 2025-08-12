# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.1.5] - 2025-01-08

### Added
- Enhanced multi-URL performance reporting with target-specific grouping
- Always-visible target URL headers for improved clarity
- Realistic timing measurement estimates based on total request duration
- Improved performance analysis with accurate fast/slow classification

### Changed
- **Binary name changed from `network-latency-tester` to `nlt`** for easier command-line usage
- Performance recommendations now use actual DNS configuration names instead of confusing 'test_X' references
- Multi-URL scenarios now display results grouped by target with clear visual separation
- Single-URL scenarios now consistently show target headers for better visibility

### Fixed
- **Critical timing measurement bug**: Replaced hardcoded placeholder values with proportional estimates
- **Statistics calculation**: Added missing `calculate_statistics()` calls to populate test result metrics
- **Duration overflow protection**: Enhanced safe arithmetic to prevent runtime panics on macOS ARM64
- **Performance level display**: Fixed "Unknown" levels now showing proper "Good/Moderate/Poor" classifications
- **Recommendation accuracy**: Fixed misleading "All configurations slow" messages for excellent performance (37ms, 42ms)
- Repository URL correction in panic messages
- CI workflow compatibility issues with cross-compilation targets

### Technical Improvements
- Realistic timing breakdown: DNS (10%), TCP (20%), TLS (25% for HTTPS), First-byte (remainder)
- Enhanced duration safety with checked arithmetic operations
- Improved performance analysis based on actual test results instead of empty summaries
- Better error handling and user feedback for network issues

## [0.1.0] - 2025-01-08 (Initial Release)

### Added
- Initial release of Network Latency Tester
- High-performance network latency testing with DNS configuration support
- Cross-platform compatibility (Windows, macOS, Linux)
- Advanced statistics and reporting capabilities
- Command-line interface with comprehensive options
- Support for multiple DNS resolution strategies
- Optimized execution with tuning capabilities

### Features
- **Core Testing**: Reliable network latency measurements
- **DNS Integration**: Advanced DNS configuration and testing
- **Cross-Platform**: Native builds for Windows, macOS (Intel & Apple Silicon), and Linux (x64 & ARM64)
- **Performance Optimization**: Adaptive timeout management and execution tuning
- **Rich Output**: Colored formatting and verbose reporting options
- **Configuration**: Environment file support and flexible parameter configuration
- **Statistics**: Comprehensive timing metrics and performance analysis

### Technical Improvements
- Rust-based implementation for maximum performance and safety
- Async/await architecture for efficient concurrent operations
- Comprehensive error handling with detailed diagnostics
- Memory-efficient data structures and algorithms
- Platform-specific optimizations for network operations

### Development
- Full CI/CD pipeline with automated testing and releases
- Cross-compilation support for major platforms
- Comprehensive test suite with property-based testing
- Performance benchmarking and profiling
- Documentation and examples for common use cases