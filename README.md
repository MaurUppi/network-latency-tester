# Network Latency Tester

A high-performance network latency testing tool built in Rust that measures connectivity to configurable target URLs using various DNS configurations including custom DNS servers and DNS-over-HTTPS providers.

## Features

- **Multiple DNS Configurations**: Test with system DNS, custom DNS servers, or DNS-over-HTTPS providers
- **Comprehensive Statistics**: Detailed timing metrics including DNS resolution, connection, and total response times
- **Network Diagnostics**: Built-in connectivity and health checks before running tests
- **Colored Output**: Rich terminal output with color-coded performance indicators
- **Flexible Configuration**: Environment variables, command-line arguments, and .env file support
- **Concurrent Testing**: Parallel execution across multiple DNS configurations
- **Cross-platform**: Works on Linux, macOS, and Windows

## Installation

### From Source

```bash
git clone https://github.com/MaurUppi/network-latency-tester
cd network-latency-tester
cargo build --release
```

The binary will be available at `target/release/network-latency-tester`.

### Using Cargo

```bash
cargo install network-latency-tester
```

## Quick Start

```bash
# Test default target with system DNS
./target/release/network-latency-tester

# Test a specific URL
./target/release/network-latency-tester --url https://example.com

# Test with custom DNS servers and 10 iterations
./target/release/network-latency-tester --count 10 --timeout 5

# Enable debug mode for detailed output
./target/release/network-latency-tester --debug --verbose

# Test the original ctok.ai URL
./target/release/network-latency-tester --test-original
```

## Configuration

### Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--url <URL>` | Target URL to test | `https://as.ctok.ai` |
| `--count <N>` | Number of test iterations | `5` |
| `--timeout <SECONDS>` | Request timeout in seconds | `10` |
| `--no-color` | Disable colored output | `false` |
| `--verbose` | Enable verbose output | `false` |
| `--debug` | Enable debug output | `false` |
| `--test-original` | Test original ctok.ai URL | `false` |
| `--help` | Show help information | - |

### Environment Variables

Create a `.env` file in your project directory (see `.env.example` for reference):

| Variable | Description | Example |
|----------|-------------|---------|
| `TARGET_URLS` | Comma-separated list of URLs to test | `https://example.com,https://google.com` |
| `DNS_SERVERS` | Comma-separated list of DNS server IPs | `8.8.8.8,1.1.1.1,208.67.222.222` |
| `DOH_PROVIDERS` | Comma-separated list of DoH URLs | `https://cloudflare-dns.com/dns-query` |
| `TEST_COUNT` | Number of test iterations (1-100) | `5` |
| `TIMEOUT_SECONDS` | Request timeout in seconds (1-300) | `10` |
| `ENABLE_COLOR` | Enable colored output | `true` |

### Configuration Priority

Configuration values are applied in the following order (highest to lowest priority):

1. Command-line arguments
2. Environment variables
3. `.env` file values
4. Default values

## Usage Examples

### Basic Usage

```bash
# Test with default configuration
./network-latency-tester

# Test specific URL with custom settings
./network-latency-tester --url https://api.github.com --count 10 --timeout 15
```

### Advanced Configuration

```bash
# Create .env file with custom configuration
cat > .env << EOF
TARGET_URLS=https://as.ctok.ai,https://api.openai.com,https://www.google.com
DNS_SERVERS=8.8.8.8,1.1.1.1,208.67.222.222,9.9.9.9
DOH_PROVIDERS=https://cloudflare-dns.com/dns-query,https://dns.google/dns-query
TEST_COUNT=10
TIMEOUT_SECONDS=5
ENABLE_COLOR=true
EOF

# Run tests with environment configuration
./network-latency-tester --verbose
```

### Performance Testing

```bash
# High-frequency testing for performance analysis
./network-latency-tester --count 20 --timeout 3 --verbose

# Compare different DNS providers
./network-latency-tester --debug --url https://example.com
```

## Output Format

The tool provides detailed output including:

- **DNS Validation**: Checks DNS configuration validity before testing
- **Test Progress**: Real-time progress updates during execution
- **Performance Tables**: Color-coded response times and success rates
- **Statistical Analysis**: Comprehensive statistics including percentiles and confidence intervals
- **Network Diagnostics**: System health and connectivity assessments
- **Recommendations**: Best performing DNS configurations

### Sample Output

```
🔍 Validating DNS configurations...
✓ DNS config System is valid
✓ DNS config Custom (8.8.8.8) is valid

📊 Planning 15 total tests across 3 DNS configs and 1 URLs
🧪 Testing https://example.com with System

📈 Generating statistical analysis...
🔧 Running network diagnostics...

🎯 Execution Results Summary
============================

📊 Overall: 15/15 tests successful (100.0%) in 2.34s
   • 0 failed, 0 timeout, 0 skipped

🏆 Recommended configuration: System

🚀 Top 3 Fastest Configurations:
   1. System - 45.2ms avg (100.0% success)
   2. Custom (8.8.8.8) - 52.1ms avg (100.0% success)
   3. DoH (Cloudflare) - 78.9ms avg (100.0% success)
```

## DNS Configuration

### System DNS

Uses your system's default DNS resolver configuration.

```bash
./network-latency-tester  # Uses system DNS
```

### Custom DNS Servers

Specify custom DNS servers via environment variables:

```bash
export DNS_SERVERS="8.8.8.8,1.1.1.1,208.67.222.222"
./network-latency-tester
```

### DNS-over-HTTPS (DoH)

Configure DoH providers for enhanced privacy:

```bash
export DOH_PROVIDERS="https://cloudflare-dns.com/dns-query,https://dns.google/dns-query"
./network-latency-tester
```

### Popular DNS Providers

| Provider | IP Address | DoH URL |
|----------|------------|---------|
| Google | `8.8.8.8`, `8.8.4.4` | `https://dns.google/dns-query` |
| Cloudflare | `1.1.1.1`, `1.0.0.1` | `https://cloudflare-dns.com/dns-query` |
| Quad9 | `9.9.9.9`, `149.112.112.112` | `https://dns.quad9.net/dns-query` |
| OpenDNS | `208.67.222.222`, `208.67.220.220` | - |
| Alibaba | `223.5.5.5`, `223.6.6.6` | - |

## Error Handling

The tool provides helpful error messages and suggestions:

### Configuration Errors
- Check .env file format
- Verify URL formats (must start with http:// or https://)
- Ensure DNS server IPs are valid
- DoH URLs must use HTTPS

### Network Errors
- Check internet connection
- Try different DNS servers
- Verify firewall settings
- Test with a different target URL

### DNS Resolution Errors
- Try using public DNS servers (8.8.8.8, 1.1.1.1)
- Check if the domain exists
- Test DNS resolution manually with `nslookup` or `dig`

## Development

### Prerequisites

- Rust 1.70+ (for async/await support)
- Cargo package manager

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- --debug
```

### Project Structure

```
src/
├── main.rs              # CLI application entry point
├── lib.rs               # Library exports and constants
├── cli.rs               # Command-line interface definition
├── app.rs               # Application core logic
├── error.rs             # Error handling system
├── types.rs             # Core type definitions
├── models/              # Data models and structures
│   ├── mod.rs
│   ├── config.rs        # Configuration models
│   └── metrics.rs       # Timing and measurement models
├── config/              # Configuration management
│   ├── mod.rs
│   ├── parser.rs        # Configuration parsing and merging
│   └── validation.rs    # Configuration validation
├── dns.rs               # DNS configuration and resolution
├── client.rs            # HTTP client with timing measurements
├── executor.rs          # Test execution engine
├── stats.rs             # Statistical analysis and calculations
├── diagnostics.rs       # Network diagnostics and health checks
└── output/              # Output formatting and display
    ├── mod.rs
    ├── formatter.rs     # Plain text formatting
    └── colored.rs       # Color-coded formatting
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test config::parser::tests

# Run integration tests
cargo test --test integration_tests
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature-name`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Run formatting (`cargo fmt`) and linting (`cargo clippy`)
7. Create a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [tokio](https://tokio.rs/) for async networking
- HTTP requests powered by [reqwest](https://github.com/seanmonstar/reqwest)
- CLI interface built with [clap](https://github.com/clap-rs/clap)
- Terminal colors via [colored](https://github.com/mackwic/colored)

## Migration Notes

This Rust implementation provides feature parity with the original bash script `check_ctok-v2.sh` while offering:

- **Better Performance**: Concurrent execution and optimized networking
- **Enhanced Reliability**: Comprehensive error handling and validation
- **Improved Usability**: Rich terminal output and configuration options
- **Cross-platform Support**: Works consistently across different operating systems
- **Maintainability**: Type-safe code with comprehensive test coverage

The tool maintains backward compatibility with the original script's output format while providing additional features and improvements.