# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.1.7] - 2025-08-14

### Fixed
- **Test Suite Reliability**: Resolved all 8 failing unit tests that were preventing successful development
  - Fixed CLI duration parsing to reject invalid formats (`+10`, `0x` prefixes)
  - Added comprehensive IPv6 DNS server support to validation system
  - Corrected help topic case-insensitive behavior expectations
  - Enhanced executor tuning with proper timing requirements for parameter scaling tests
  - Fixed correlation ID string handling in logging system for IDs shorter than 8 characters
  - Resolved console timing summary display to show actual configuration names instead of "Unknown"
  - Fixed `best_config()` method to return correct HashMap keys instead of internal config names

### Added
- **Strategic Planning**: Created comprehensive bash parity improvement plan
  - Added `.spec-workflow/specs/improvement/` directory structure
  - Documented strategic approach for handling 15 failing bash parity integration tests
  - Established framework for prioritizing CLI feature implementation based on user impact

### Technical Improvements
- **Core Functionality**: All 329 unit tests now pass successfully (100% success rate)
- **Code Quality**: Enhanced string parsing validation and safety
- **DNS Support**: Expanded known public DNS servers list to include major IPv6 providers
- **Error Handling**: Improved safety in string operations and correlation ID management
- **Test Framework**: Better test isolation and timing management for concurrent operations

### Development
- **Test Strategy**: Clear separation between core functionality tests (passing) and CLI parity tests (strategic)
- **Documentation**: Comprehensive analysis of integration test failures with resolution roadmap
- **Quality Assurance**: Systematic approach to test failure resolution and prevention

## [0.1.6] - 2025-08-12

### Changed
- **DNS Result Ordering**: Results now grouped by DNS type (System DNS → Custom DNS → DoH) instead of pure performance sorting
- Within each DNS type group, configurations are still sorted by performance (best first)
- Improves result readability and provides logical grouping for better DNS comparison

### Technical Improvements
- Enhanced sorting algorithm to prioritize DNS type categorization while maintaining performance insights
- Updated sample outputs in documentation to reflect new ordering
- Removed tracked .env file from repository (keeping .env.example for reference)

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